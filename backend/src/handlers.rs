use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    db, models::{ApiResponse, Game, GameSummary, Stats}, scanner, steam, AppState
};

pub async fn health() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse::success("OK"))
}

pub async fn list_games(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<Vec<GameSummary>>> {
    match db::get_all_games(&state.db).await {
        Ok(games) => {
            let summaries: Vec<GameSummary> = games.into_iter().map(|g| g.into()).collect();
            Json(ApiResponse::success(summaries))
        }
        Err(e) => {
            tracing::error!("Failed to list games: {}", e);
            Json(ApiResponse::error("Internal server error"))
        }
    }
}

pub async fn get_game(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<ApiResponse<Game>> {
    match db::get_game_by_id(&state.db, id).await {
        Ok(Some(game)) => Json(ApiResponse::success(game)),
        Ok(None) => Json(ApiResponse::error("Game not found")),
        Err(e) => {
            tracing::error!("Failed to get game {}: {}", id, e);
            Json(ApiResponse::error("Internal server error"))
        }
    }
}

/// SECURITY: Search query constraints
const MAX_SEARCH_QUERY_LENGTH: usize = 200;
const MIN_SEARCH_QUERY_LENGTH: usize = 1;

/// Enrichment configuration
const ENRICHMENT_BATCH_SIZE: usize = 20;
const STEAM_API_RATE_LIMIT_MS: u64 = 500;

#[derive(Deserialize)]
pub struct SearchQuery {
    q: String,
}

pub async fn search_games(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Json<ApiResponse<Vec<GameSummary>>> {
    // SECURITY: Validate search query length to prevent abuse
    let query_trimmed = query.q.trim();
    if query_trimmed.len() < MIN_SEARCH_QUERY_LENGTH {
        return Json(ApiResponse::error("Search query too short"));
    }
    if query_trimmed.len() > MAX_SEARCH_QUERY_LENGTH {
        return Json(ApiResponse::error("Search query too long"));
    }

    match db::search_games(&state.db, query_trimmed).await {
        Ok(games) => {
            let summaries: Vec<GameSummary> = games.into_iter().map(|g| g.into()).collect();
            Json(ApiResponse::success(summaries))
        }
        Err(e) => {
            tracing::error!("Failed to search games: {}", e);
            Json(ApiResponse::error("Internal server error"))
        }
    }
}

pub async fn scan_games(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<ScanResult>> {
    tracing::info!("Starting game scan of {}", state.games_path);

    let games = scanner::scan_games_directory(&state.games_path);
    let total = games.len();
    let mut added = 0;

    for game in games {
        match db::upsert_game(
            &state.db,
            &game.folder_path,
            &game.folder_name,
            &game.clean_title,
            game.size_bytes,
        )
        .await
        {
            Ok(_) => added += 1,
            Err(e) => {
                tracing::warn!("Failed to upsert game '{}': {}", game.clean_title, e);
            }
        }
    }

    tracing::info!("Scan complete: {} games found, {} added/updated", total, added);

    Json(ApiResponse::success(ScanResult {
        total_found: total,
        added_or_updated: added,
    }))
}

#[derive(serde::Serialize)]
pub struct ScanResult {
    total_found: usize,
    added_or_updated: usize,
}

pub async fn enrich_games(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<EnrichResult>> {
    tracing::info!("Starting Steam enrichment");

    let games = match db::get_pending_games(&state.db).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to get pending games: {}", e);
            return Json(ApiResponse::error("Internal server error"));
        }
    };

    let client = reqwest::Client::new();
    let mut enriched = 0;
    let mut failed = 0;

    // Process up to ENRICHMENT_BATCH_SIZE games per request to avoid timeouts
    for game in games.iter().take(ENRICHMENT_BATCH_SIZE) {
        tracing::info!("Enriching: {}", game.title);

        // Search for Steam App ID
        let (app_id, confidence) = match steam::search_steam_app(&client, &game.title).await {
            Some((id, conf)) => (id, conf),
            None => {
                failed += 1;
                continue;
            }
        };

        // Rate limit
        tokio::time::sleep(tokio::time::Duration::from_millis(STEAM_API_RATE_LIMIT_MS)).await;

        // Fetch details
        let details = steam::fetch_steam_details(&client, app_id).await;

        // Rate limit
        tokio::time::sleep(tokio::time::Duration::from_millis(STEAM_API_RATE_LIMIT_MS)).await;

        // Fetch reviews
        let reviews = steam::fetch_steam_reviews(&client, app_id).await;

        // Update database
        if let Some(d) = details {
            let genres_json = d.genres.map(|g| serde_json::to_string(&g).unwrap_or_default());
            let devs_json = d.developers.map(|g| serde_json::to_string(&g).unwrap_or_default());
            let pubs_json = d.publishers.map(|g| serde_json::to_string(&g).unwrap_or_default());

            if let Err(e) = db::update_game_steam_data(
                &state.db,
                game.id,
                app_id,
                d.description.as_deref(),
                d.header_image.as_deref(),
                d.background.as_deref(),
                genres_json.as_deref(),
                devs_json.as_deref(),
                pubs_json.as_deref(),
                d.release_date.as_deref(),
                confidence,
            )
            .await
            {
                tracing::warn!("Failed to update game {}: {}", game.id, e);
                failed += 1;
                continue;
            }
        }

        if let Some(r) = reviews {
            if let Err(e) = db::update_game_reviews(
                &state.db,
                game.id,
                r.score,
                r.count,
                &r.summary,
            )
            .await
            {
                tracing::warn!("Failed to update reviews for game {}: {}", game.id, e);
            }
        }

        enriched += 1;
        tracing::info!("Enriched: {} (Steam App ID: {})", game.title, app_id);
    }

    tracing::info!("Enrichment complete: {} enriched, {} failed", enriched, failed);

    Json(ApiResponse::success(EnrichResult {
        enriched,
        failed,
        remaining: games.len().saturating_sub(ENRICHMENT_BATCH_SIZE),
    }))
}

#[derive(serde::Serialize)]
pub struct EnrichResult {
    enriched: usize,
    failed: usize,
    remaining: usize,
}

pub async fn get_stats(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<Stats>> {
    match db::get_stats(&state.db).await {
        Ok(stats) => Json(ApiResponse::success(stats)),
        Err(e) => {
            tracing::error!("Failed to get stats: {}", e);
            Json(ApiResponse::error("Internal server error"))
        }
    }
}
