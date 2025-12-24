use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

use crate::{
    config::{self, AppConfig},
    db, local_storage,
    models::{ApiResponse, Game, GameSummary, Stats},
    scanner, steam, AppState,
};

pub async fn health() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse::success("OK"))
}

pub async fn list_games(State(state): State<Arc<AppState>>) -> Json<ApiResponse<Vec<GameSummary>>> {
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

pub async fn scan_games(State(state): State<Arc<AppState>>) -> Json<ApiResponse<ScanResult>> {
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

    tracing::info!(
        "Scan complete: {} games found, {} added/updated",
        total,
        added
    );

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

pub async fn enrich_games(State(state): State<Arc<AppState>>) -> Json<ApiResponse<EnrichResult>> {
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
            let genres_json = d
                .genres
                .map(|g| serde_json::to_string(&g).unwrap_or_default());
            let devs_json = d
                .developers
                .map(|g| serde_json::to_string(&g).unwrap_or_default());
            let pubs_json = d
                .publishers
                .map(|g| serde_json::to_string(&g).unwrap_or_default());

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
            )
            .await;

            // Update database with local image paths
            if local_cover.is_some() || local_bg.is_some() {
                if let Err(e) = db::update_game_local_images(
                    &state.db,
                    game.id,
                    local_cover.as_deref(),
                    local_bg.as_deref(),
                )
                .await
                {
                    tracing::warn!(
                        "Failed to update local image paths for game {}: {}",
                        game.id,
                        e
                    );
                }
            }
        }

        if let Some(r) = reviews {
            if let Err(e) =
                db::update_game_reviews(&state.db, game.id, r.score, r.count, &r.summary).await
            {
                tracing::warn!("Failed to update reviews for game {}: {}", game.id, e);
            }
        }

        enriched += 1;
        tracing::info!("Enriched: {} (Steam App ID: {})", game.title, app_id);
    }

    tracing::info!(
        "Enrichment complete: {} enriched, {} failed",
        enriched,
        failed
    );

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

pub async fn get_stats(State(state): State<Arc<AppState>>) -> Json<ApiResponse<Stats>> {
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

/// SECURITY: Validate that a path is within the allowed games directory
/// Returns the canonicalized path if valid, None if path traversal detected
fn validate_path_within_games(
    games_path: &str,
    file_path: &std::path::Path,
) -> Option<std::path::PathBuf> {
    // Canonicalize the games directory (resolve symlinks, normalize)
    let games_canonical = match std::fs::canonicalize(games_path) {
        Ok(p) => p,
        Err(_) => return None,
    };

    // Canonicalize the target file path
    let file_canonical = match std::fs::canonicalize(file_path) {
        Ok(p) => p,
        Err(_) => return None,
    };

    // SECURITY: Verify the file is within the games directory
    if file_canonical.starts_with(&games_canonical) {
        Some(file_canonical)
    } else {
        tracing::warn!(
            "Path traversal attempt blocked: {:?} is not within {:?}",
            file_path,
            games_canonical
        );
        None
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

    // SECURITY: Validate path is within games directory
    let validated_path = match validate_path_within_games(&state.games_path, &cover_path) {
        Some(p) => p,
        None => {
            return (StatusCode::FORBIDDEN, "Access denied").into_response();
        }
    };

    // Read and serve the image
    match std::fs::read(&validated_path) {
        Ok(bytes) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "image/jpeg")],
            bytes,
        )
            .into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read image").into_response(),
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

    // SECURITY: Validate path is within games directory
    let validated_path = match validate_path_within_games(&state.games_path, &bg_path) {
        Some(p) => p,
        None => {
            return (StatusCode::FORBIDDEN, "Access denied").into_response();
        }
    };

    // Read and serve the image
    match std::fs::read(&validated_path) {
        Ok(bytes) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "image/jpeg")],
            bytes,
        )
            .into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read image").into_response(),
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

    tracing::info!(
        "Export complete: {} exported, {} skipped, {} failed",
        exported,
        skipped,
        failed
    );

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
                let genres_json = metadata
                    .genres
                    .as_ref()
                    .map(|g| serde_json::to_string(g).unwrap_or_default());
                let devs_json = metadata
                    .developers
                    .as_ref()
                    .map(|d| serde_json::to_string(d).unwrap_or_default());
                let pubs_json = metadata
                    .publishers
                    .as_ref()
                    .map(|p| serde_json::to_string(p).unwrap_or_default());

                // Extract HLTB data
                let (hltb_main, hltb_extra, hltb_comp) = metadata
                    .hltb
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
                )
                .await
                {
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

    tracing::info!(
        "Import complete: {} imported, {} skipped, {} not found, {} failed",
        imported,
        skipped,
        not_found,
        failed
    );

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

/// Request body for re-matching a game to a different Steam entry
#[derive(Deserialize)]
pub struct RematchGameRequest {
    /// Steam URL or App ID (e.g., "https://store.steampowered.com/app/292030/..." or "292030")
    pub steam_input: String,
}

/// Response for rematch operation
#[derive(serde::Serialize)]
pub struct RematchResult {
    pub steam_app_id: i64,
    pub title: String,
    pub summary: Option<String>,
    pub genres: Option<Vec<String>>,
    pub developers: Option<Vec<String>>,
    pub publishers: Option<Vec<String>>,
    pub release_date: Option<String>,
    pub cover_url: Option<String>,
    pub review_score: Option<i64>,
    pub review_summary: Option<String>,
}

/// Extract Steam App ID from URL or raw ID string
fn parse_steam_input(input: &str) -> Option<i64> {
    let input = input.trim();

    // Try parsing as raw App ID first
    if let Ok(id) = input.parse::<i64>() {
        return Some(id);
    }

    // Try extracting from URL patterns
    // Matches: store.steampowered.com/app/292030 or /app/292030/
    let re = regex::Regex::new(r"(?:store\.steampowered\.com/app/|^/app/|/app/)(\d+)").ok()?;
    re.captures(input)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

/// Re-match a game to a different Steam entry (POST /games/{id}/match)
/// Fetches Steam data and returns preview for confirmation
pub async fn rematch_game(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(payload): Json<RematchGameRequest>,
) -> Json<ApiResponse<RematchResult>> {
    tracing::info!("Rematch game {} with input: {}", id, payload.steam_input);

    // Parse Steam App ID from input
    let steam_app_id = match parse_steam_input(&payload.steam_input) {
        Some(id) => id,
        None => {
            return Json(ApiResponse::error("Invalid Steam URL or App ID. Please enter a valid Steam store URL or numeric App ID."));
        }
    };

    // Verify the game exists
    match db::get_game_by_id(&state.db, id).await {
        Ok(Some(_)) => {}
        Ok(None) => return Json(ApiResponse::error("Game not found")),
        Err(e) => {
            tracing::error!("Failed to get game {}: {}", id, e);
            return Json(ApiResponse::error("Database error"));
        }
    }

    // Fetch Steam details
    let client = reqwest::Client::new();
    let details = steam::fetch_steam_details(&client, steam_app_id).await;

    if details.is_none() {
        return Json(ApiResponse::error(
            "Could not fetch Steam game details. Please verify the App ID is correct.",
        ));
    }

    let d = details.unwrap();

    // Fetch reviews
    let reviews = steam::fetch_steam_reviews(&client, steam_app_id).await;

    // Build preview response
    let result = RematchResult {
        steam_app_id,
        title: d.name,
        summary: d.description,
        genres: d.genres,
        developers: d.developers,
        publishers: d.publishers,
        release_date: d.release_date,
        cover_url: d.header_image,
        review_score: reviews.as_ref().map(|r| r.score),
        review_summary: reviews.as_ref().map(|r| r.summary.clone()),
    };

    Json(ApiResponse::success(result))
}

/// Confirm and apply a rematch (POST /games/{id}/match/confirm)
/// Actually updates the game with the new Steam data
pub async fn confirm_rematch(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(payload): Json<RematchGameRequest>,
) -> Json<ApiResponse<Game>> {
    tracing::info!(
        "Confirming rematch for game {} with input: {}",
        id,
        payload.steam_input
    );

    // Parse Steam App ID from input
    let steam_app_id = match parse_steam_input(&payload.steam_input) {
        Some(id) => id,
        None => {
            return Json(ApiResponse::error("Invalid Steam URL or App ID"));
        }
    };

    // Get the game
    let game = match db::get_game_by_id(&state.db, id).await {
        Ok(Some(g)) => g,
        Ok(None) => return Json(ApiResponse::error("Game not found")),
        Err(e) => {
            tracing::error!("Failed to get game {}: {}", id, e);
            return Json(ApiResponse::error("Database error"));
        }
    };

    // Fetch Steam details
    let client = reqwest::Client::new();
    let details = steam::fetch_steam_details(&client, steam_app_id).await;

    if details.is_none() {
        return Json(ApiResponse::error("Could not fetch Steam game details"));
    }

    let d = details.unwrap();

    // Fetch reviews
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    let reviews = steam::fetch_steam_reviews(&client, steam_app_id).await;

    // Update database with new Steam data
    let genres_json = d
        .genres
        .map(|g| serde_json::to_string(&g).unwrap_or_default());
    let devs_json = d
        .developers
        .map(|g| serde_json::to_string(&g).unwrap_or_default());
    let pubs_json = d
        .publishers
        .map(|g| serde_json::to_string(&g).unwrap_or_default());

    if let Err(e) = db::update_game_steam_data(
        &state.db,
        id,
        steam_app_id,
        d.description.as_deref(),
        d.header_image.as_deref(),
        d.background.as_deref(),
        genres_json.as_deref(),
        devs_json.as_deref(),
        pubs_json.as_deref(),
        d.release_date.as_deref(),
        1.0, // Manual match has full confidence
    )
    .await
    {
        tracing::error!("Failed to update game steam data: {}", e);
        return Json(ApiResponse::error("Failed to update game"));
    }

    // Update reviews if available
    if let Some(r) = reviews {
        if let Err(e) = db::update_game_reviews(&state.db, id, r.score, r.count, &r.summary).await {
            tracing::warn!("Failed to update reviews: {}", e);
        }
    }

    // Cache images locally
    let (local_cover, local_bg) = local_storage::cache_game_images(
        &client,
        &game.folder_path,
        d.header_image.as_deref(),
        d.background.as_deref(),
    )
    .await;

    if local_cover.is_some() || local_bg.is_some() {
        if let Err(e) =
            db::update_game_local_images(&state.db, id, local_cover.as_deref(), local_bg.as_deref())
                .await
        {
            tracing::warn!("Failed to update local image paths: {}", e);
        }
    }

    // Fetch updated game
    let updated_game = match db::get_game_by_id(&state.db, id).await {
        Ok(Some(g)) => g,
        Ok(None) => return Json(ApiResponse::error("Game not found after update")),
        Err(e) => {
            tracing::error!("Failed to get updated game: {}", e);
            return Json(ApiResponse::error("Database error"));
        }
    };

    // Dual-write to metadata.json
    if let Err(e) = local_storage::save_game_metadata(&updated_game) {
        tracing::warn!("Failed to save metadata.json: {}", e);
    }

    tracing::info!("Rematched game {} to Steam App ID {}", id, steam_app_id);
    Json(ApiResponse::success(updated_game))
}

/// Request body for updating game metadata
#[derive(Deserialize)]
pub struct UpdateGameRequest {
    pub title: Option<String>,
    pub summary: Option<String>,
    pub genres: Option<Vec<String>>,
    pub developers: Option<Vec<String>>,
    pub publishers: Option<Vec<String>>,
    pub release_date: Option<String>,
    pub review_score: Option<i64>,
}

/// Update game metadata (PUT /games/{id})
/// Dual-writes to DB and metadata.json
pub async fn update_game(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateGameRequest>,
) -> Json<ApiResponse<Game>> {
    tracing::info!("Updating game {}", id);

    // Convert Vec<String> to JSON strings for DB storage
    let genres_json = payload
        .genres
        .map(|g| serde_json::to_string(&g).unwrap_or_default());
    let devs_json = payload
        .developers
        .map(|d| serde_json::to_string(&d).unwrap_or_default());
    let pubs_json = payload
        .publishers
        .map(|p| serde_json::to_string(&p).unwrap_or_default());

    // Update database and get updated game
    let game = match db::update_game_metadata(
        &state.db,
        id,
        payload.title.as_deref(),
        payload.summary.as_deref(),
        genres_json.as_deref(),
        devs_json.as_deref(),
        pubs_json.as_deref(),
        payload.release_date.as_deref(),
        payload.review_score,
    )
    .await
    {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to update game {}: {}", id, e);
            return Json(ApiResponse::error("Failed to update game"));
        }
    };

    // Dual-write to metadata.json (don't fail if file write fails)
    if let Err(e) = local_storage::save_game_metadata(&game) {
        tracing::warn!("Failed to save metadata.json for game {}: {}", id, e);
        // Continue - DB update succeeded, which is the primary storage
    }

    tracing::info!("Updated game: {} (id={})", game.title, id);
    Json(ApiResponse::success(game))
}

// ============================================================================
// Configuration API
// ============================================================================

/// Response structure for GET /api/config
#[derive(serde::Serialize)]
pub struct ConfigResponse {
    pub paths: ConfigPathsResponse,
    pub server: ConfigServerResponse,
}

#[derive(serde::Serialize)]
pub struct ConfigPathsResponse {
    pub game_library: String,
    pub cache: String,
    pub game_library_exists: bool,
    pub cache_exists: bool,
}

#[derive(serde::Serialize)]
pub struct ConfigServerResponse {
    pub port: u16,
    pub auto_open_browser: bool,
    pub bind_address: String,
}

/// Get current configuration (GET /api/config)
pub async fn get_config() -> Json<ApiResponse<ConfigResponse>> {
    match AppConfig::load() {
        Ok(cfg) => {
            // Get original config values (for display)
            let original_game_library = cfg.paths.game_library.to_string_lossy().to_string();
            let original_cache = cfg.paths.cache.to_string_lossy().to_string();

            // Get resolved paths (for validation)
            let games_path = cfg.games_path();
            let cache_path = cfg.cache_path();

            let response = ConfigResponse {
                paths: ConfigPathsResponse {
                    // Show original path if set, otherwise show placeholder
                    game_library: if original_game_library.is_empty() {
                        String::new()
                    } else {
                        original_game_library
                    },
                    cache: if original_cache.is_empty() {
                        "./cache".to_string()
                    } else {
                        original_cache
                    },
                    game_library_exists: !cfg.paths.game_library.to_string_lossy().is_empty()
                        && games_path.is_dir(),
                    cache_exists: cache_path.is_dir(),
                },
                server: ConfigServerResponse {
                    port: cfg.server.port,
                    auto_open_browser: cfg.server.auto_open_browser,
                    bind_address: cfg.server.bind_address.clone(),
                },
            };
            Json(ApiResponse::success(response))
        }
        Err(e) => {
            tracing::error!("Failed to load config: {}", e);
            Json(ApiResponse::error("Failed to load configuration"))
        }
    }
}

/// Request structure for PUT /api/config
#[derive(Deserialize)]
pub struct ConfigUpdateRequest {
    pub game_library: String,
    pub cache: String,
    pub port: u16,
    pub auto_open_browser: bool,
}

/// Response structure for PUT /api/config
#[derive(serde::Serialize)]
pub struct ConfigUpdateResponse {
    pub success: bool,
    pub restart_required: bool,
    pub message: String,
}

/// Update configuration (PUT /api/config)
pub async fn update_config(
    Json(payload): Json<ConfigUpdateRequest>,
) -> Json<ApiResponse<ConfigUpdateResponse>> {
    // Validate game library path
    let game_path = std::path::PathBuf::from(&payload.game_library);
    if !game_path.is_dir() {
        return Json(ApiResponse::error(
            "Game library path does not exist or is not a directory",
        ));
    }

    // SECURITY: Canonicalize path to resolve symlinks and prevent symlink attacks
    let game_path = match std::fs::canonicalize(&game_path) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("Failed to canonicalize game library path: {}", e);
            return Json(ApiResponse::error("Invalid game library path"));
        }
    };

    // SECURITY: Verify it's still a directory after canonicalization
    if !game_path.is_dir() {
        return Json(ApiResponse::error(
            "Game library path is not a valid directory",
        ));
    }

    // Validate port range
    if payload.port < 1024 || payload.port > 65535 {
        return Json(ApiResponse::error("Port must be between 1024 and 65535"));
    }

    // Load current config to check for restart-requiring changes
    let current_config = AppConfig::load().ok();
    let restart_required = current_config
        .as_ref()
        .map(|c| c.server.port != payload.port)
        .unwrap_or(false);

    // Build new config
    let new_config = AppConfig {
        paths: config::PathsConfig {
            game_library: game_path,
            database: current_config
                .as_ref()
                .map(|c| c.paths.database.clone())
                .unwrap_or_else(|| "sqlite:./data/gamevault.db?mode=rwc".to_string()),
            cache: std::path::PathBuf::from(&payload.cache),
        },
        server: config::ServerConfig {
            port: payload.port,
            auto_open_browser: payload.auto_open_browser,
            bind_address: current_config
                .as_ref()
                .map(|c| c.server.bind_address.clone())
                .unwrap_or_else(|| "127.0.0.1".to_string()),
        },
    };

    // Write config atomically
    match config::write_config(&new_config) {
        Ok(_) => {
            let message = if restart_required {
                "Configuration saved. Restart required for port change.".to_string()
            } else {
                "Configuration saved successfully.".to_string()
            };

            Json(ApiResponse::success(ConfigUpdateResponse {
                success: true,
                restart_required,
                message,
            }))
        }
        Err(e) => {
            tracing::error!("Failed to save config: {}", e);
            Json(ApiResponse::error("Failed to save configuration"))
        }
    }
}

/// Shutdown the server (POST /api/shutdown)
pub async fn shutdown_server() -> Json<ApiResponse<&'static str>> {
    tracing::info!("Shutdown requested via API");

    // Spawn a task to shutdown after response is sent
    tokio::spawn(async {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        std::process::exit(0);
    });

    Json(ApiResponse::success("Shutting down..."))
}

/// Restart the server (POST /api/restart)
pub async fn restart_server() -> Json<ApiResponse<&'static str>> {
    tracing::info!("Restart requested via API");

    // Get the current executable path
    let exe_path = match std::env::current_exe() {
        Ok(path) => path,
        Err(e) => {
            tracing::error!("Failed to get executable path: {}", e);
            return Json(ApiResponse::error("Failed to get executable path"));
        }
    };

    // Spawn a task to restart after response is sent
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Spawn new process
        match std::process::Command::new(&exe_path).spawn() {
            Ok(_) => {
                tracing::info!("New instance spawned, exiting current process");
            }
            Err(e) => {
                tracing::error!("Failed to spawn new process: {}", e);
            }
        }

        // Exit current process
        std::process::exit(0);
    });

    Json(ApiResponse::success("Restarting..."))
}

/// Check if game library path is configured (GET /api/config/status)
pub async fn get_config_status() -> Json<ApiResponse<ConfigStatusResponse>> {
    match AppConfig::load() {
        Ok(cfg) => {
            // Get original config value (not resolved)
            let original_path = cfg.paths.game_library.to_string_lossy().to_string();
            let resolved_path = cfg.games_path();
            let resolved_str = resolved_path.to_string_lossy().to_string();

            // Check if original path is empty, ".", or resolved path doesn't exist
            let needs_setup =
                original_path.is_empty() || original_path == "." || !resolved_path.is_dir();

            Json(ApiResponse::success(ConfigStatusResponse {
                needs_setup,
                game_library_configured: !needs_setup,
                game_library_path: resolved_str,
            }))
        }
        Err(_) => Json(ApiResponse::success(ConfigStatusResponse {
            needs_setup: true,
            game_library_configured: false,
            game_library_path: String::new(),
        })),
    }
}

#[derive(serde::Serialize)]
pub struct ConfigStatusResponse {
    pub needs_setup: bool,
    pub game_library_configured: bool,
    pub game_library_path: String,
}
