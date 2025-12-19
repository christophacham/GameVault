use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    db, local_storage, models::{ApiResponse, Game, GameSummary, Stats}, scanner, steam, AppState
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

    let games = match db::get_games_needing_enrichment(&state.db).await {
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

            // Cache images locally in the game folder
            let (local_cover, local_bg) = local_storage::cache_game_images(
                &client,
                &game.folder_path,
                d.header_image.as_deref(),
                d.background.as_deref(),
            ).await;

            // Update database with local image paths
            if local_cover.is_some() || local_bg.is_some() {
                if let Err(e) = db::update_game_local_images(
                    &state.db,
                    game.id,
                    local_cover.as_deref(),
                    local_bg.as_deref(),
                ).await {
                    tracing::warn!("Failed to update local image paths for game {}: {}", game.id, e);
                }
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
        total: games.len(),
    }))
}

#[derive(serde::Serialize)]
pub struct EnrichResult {
    enriched: usize,
    failed: usize,
    remaining: usize,
    total: usize,
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

/// Get recently added games
pub async fn get_recent_games(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<Vec<GameSummary>>> {
    match db::get_recent_games(&state.db, 10).await {
        Ok(games) => {
            let summaries: Vec<GameSummary> = games.into_iter().map(|g| g.into()).collect();
            Json(ApiResponse::success(summaries))
        }
        Err(e) => {
            tracing::error!("Failed to get recent games: {}", e);
            Json(ApiResponse::error("Internal server error"))
        }
    }
}

/// Serve a game's cover image from local storage
pub async fn serve_game_cover(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> axum::response::Response {
    use axum::http::{header, StatusCode};
    use axum::response::IntoResponse;

    // Get game folder path
    let folder_path = match db::get_game_folder_path(&state.db, id).await {
        Ok(Some(path)) => path,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, "Game not found").into_response();
        }
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    // Get cover image path
    let cover_path = local_storage::get_cover_path(&folder_path);

    if !cover_path.exists() {
        return (StatusCode::NOT_FOUND, "Cover image not found").into_response();
    }

    // Read and serve the image
    match std::fs::read(&cover_path) {
        Ok(bytes) => {
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "image/jpeg")],
                bytes,
            ).into_response()
        }
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read image").into_response()
        }
    }
}

/// Serve a game's background image from local storage
pub async fn serve_game_background(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> axum::response::Response {
    use axum::http::{header, StatusCode};
    use axum::response::IntoResponse;

    // Get game folder path
    let folder_path = match db::get_game_folder_path(&state.db, id).await {
        Ok(Some(path)) => path,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, "Game not found").into_response();
        }
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    // Get background image path
    let bg_path = local_storage::get_background_path(&folder_path);

    if !bg_path.exists() {
        return (StatusCode::NOT_FOUND, "Background image not found").into_response();
    }

    // Read and serve the image
    match std::fs::read(&bg_path) {
        Ok(bytes) => {
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "image/jpeg")],
                bytes,
            ).into_response()
        }
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read image").into_response()
        }
    }
}

/// Check if a game folder is writable (for backup functionality)
pub async fn check_folder_writable(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<ApiResponse<FolderStatus>> {
    let folder_path = match db::get_game_folder_path(&state.db, id).await {
        Ok(Some(path)) => path,
        Ok(None) => {
            return Json(ApiResponse::error("Game not found"));
        }
        Err(e) => {
            tracing::error!("Failed to get game folder: {}", e);
            return Json(ApiResponse::error("Internal server error"));
        }
    };

    let writable = local_storage::is_folder_writable(&folder_path);
    let backups = if writable {
        local_storage::list_backups(&folder_path)
    } else {
        vec![]
    };

    Json(ApiResponse::success(FolderStatus {
        writable,
        backup_count: backups.len(),
        backups,
    }))
}

#[derive(serde::Serialize)]
pub struct FolderStatus {
    pub writable: bool,
    pub backup_count: usize,
    pub backups: Vec<local_storage::BackupInfo>,
}

/// Export metadata for all matched games to their .gamevault folders
pub async fn export_all_metadata(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<ExportResult>> {
    tracing::info!("Starting metadata export");

    // Get all matched games
    let games = match db::get_all_games(&state.db).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to get games: {}", e);
            return Json(ApiResponse::error("Internal server error"));
        }
    };

    let mut exported = 0;
    let mut skipped = 0;
    let mut failed = 0;

    for game in &games {
        // Skip games without Steam data
        if game.steam_app_id.is_none() {
            skipped += 1;
            continue;
        }

        match local_storage::export_game_metadata(game) {
            Ok(_) => {
                exported += 1;
            }
            Err(e) => {
                tracing::warn!("Failed to export metadata for '{}': {}", game.title, e);
                failed += 1;
            }
        }
    }

    tracing::info!("Export complete: {} exported, {} skipped, {} failed", exported, skipped, failed);

    Json(ApiResponse::success(ExportResult {
        exported,
        skipped,
        failed,
        total: games.len(),
    }))
}

#[derive(serde::Serialize)]
pub struct ExportResult {
    pub exported: usize,
    pub skipped: usize,
    pub failed: usize,
    pub total: usize,
}
/// Import metadata from .gamevault/metadata.json files into database
pub async fn import_all_metadata(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<ImportResult>> {
    tracing::info!("Starting metadata import");

    // Get all games
    let games = match db::get_all_games(&state.db).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to get games: {}", e);
            return Json(ApiResponse::error("Internal server error"));
        }
    };

    let mut imported = 0;
    let mut skipped = 0;
    let mut not_found = 0;
    let mut failed = 0;

    for game in &games {
        match local_storage::import_game_metadata(game) {
            local_storage::ImportResult::Imported(metadata) => {
                // Convert Vec<String> to JSON strings for database
                let genres_json = metadata.genres.as_ref()
                    .map(|g| serde_json::to_string(g).unwrap_or_default());
                let devs_json = metadata.developers.as_ref()
                    .map(|d| serde_json::to_string(d).unwrap_or_default());
                let pubs_json = metadata.publishers.as_ref()
                    .map(|p| serde_json::to_string(p).unwrap_or_default());

                // Extract HLTB data
                let (hltb_main, hltb_extra, hltb_comp) = metadata.hltb
                    .map(|h| (h.main_mins, h.extra_mins, h.completionist_mins))
                    .unwrap_or((None, None, None));

                // Update database
                if let Err(e) = db::update_game_from_import(
                    &state.db,
                    game.id,
                    metadata.steam_app_id,
                    metadata.summary.as_deref(),
                    genres_json.as_deref(),
                    devs_json.as_deref(),
                    pubs_json.as_deref(),
                    metadata.release_date.as_deref(),
                    metadata.review_score,
                    metadata.review_summary.as_deref(),
                    hltb_main,
                    hltb_extra,
                    hltb_comp,
                ).await {
                    tracing::warn!("Failed to import metadata for '{}': {}", game.title, e);
                    failed += 1;
                } else {
                    imported += 1;
                    tracing::info!("Imported metadata for: {}", game.title);
                }
            }
            local_storage::ImportResult::Skipped { reason } => {
                tracing::debug!("Skipped '{}': {}", game.title, reason);
                skipped += 1;
            }
            local_storage::ImportResult::NotFound => {
                not_found += 1;
            }
            local_storage::ImportResult::Failed { error } => {
                tracing::warn!("Failed to read metadata for '{}': {}", game.title, error);
                failed += 1;
            }
        }
    }

    tracing::info!("Import complete: {} imported, {} skipped, {} not found, {} failed", 
                   imported, skipped, not_found, failed);

    Json(ApiResponse::success(ImportResult {
        imported,
        skipped,
        not_found,
        failed,
        total: games.len(),
    }))
}

#[derive(serde::Serialize)]
pub struct ImportResult {
    pub imported: usize,
    pub skipped: usize,
    pub not_found: usize,
    pub failed: usize,
    pub total: usize,
}

