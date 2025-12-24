---
sidebar_position: 2
---

# handlers.rs

HTTP endpoint handlers implementing the GameVault REST API.

## Endpoint Summary

### Public Endpoints

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| GET | `/api/health` | `health` | Health check |
| GET | `/api/games` | `list_games` | List all games |
| GET | `/api/games/:id` | `get_game` | Get single game |
| GET | `/api/games/search` | `search_games` | Search by title |
| GET | `/api/games/recent` | `get_recent_games` | Recently added |
| GET | `/api/stats` | `get_stats` | Library statistics |
| GET | `/api/games/:id/cover` | `serve_game_cover` | Serve cover image |
| GET | `/api/games/:id/background` | `serve_game_background` | Serve background |

### Protected Endpoints

Require `API_KEY` if set:

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| POST | `/api/scan` | `scan_games` | Scan for games |
| POST | `/api/enrich` | `enrich_games` | Fetch Steam data |
| PUT | `/api/games/:id` | `update_game` | Edit metadata |
| POST | `/api/games/:id/match` | `rematch_game` | Preview rematch |
| POST | `/api/games/:id/match/confirm` | `confirm_rematch` | Apply rematch |
| POST | `/api/export` | `export_all_metadata` | Export to files |
| POST | `/api/import` | `import_all_metadata` | Import from files |

### Config Endpoints

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| GET | `/api/config` | `get_config` | Get settings |
| PUT | `/api/config` | `update_config` | Update settings |
| GET | `/api/config/status` | `get_config_status` | Check setup state |
| POST | `/api/shutdown` | `shutdown_server` | Stop server |
| POST | `/api/restart` | `restart_server` | Restart server |

## Request Types

### SearchQuery

```rust
#[derive(Deserialize)]
pub struct SearchQuery {
    q: String,  // Query parameter: ?q=witcher
}
```

### UpdateGameRequest

```rust
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
```

### RematchGameRequest

```rust
#[derive(Deserialize)]
pub struct RematchGameRequest {
    /// Steam URL or App ID
    pub steam_input: String,
}
```

### ConfigUpdateRequest

```rust
#[derive(Deserialize)]
pub struct ConfigUpdateRequest {
    pub game_library: String,
    pub cache: String,
    pub port: u16,
    pub auto_open_browser: bool,
}
```

## Response Types

### ScanResult

```rust
#[derive(Serialize)]
pub struct ScanResult {
    total_found: usize,
    added_or_updated: usize,
}
```

### EnrichResult

```rust
#[derive(Serialize)]
pub struct EnrichResult {
    enriched: usize,
    failed: usize,
    remaining: usize,
    total: usize,
}
```

### RematchResult

```rust
#[derive(Serialize)]
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
```

## Handler Patterns

### Basic CRUD

```rust
pub async fn get_game(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Json<ApiResponse<Game>> {
    match db::get_game_by_id(&state.db, id).await {
        Ok(Some(game)) => Json(ApiResponse::success(game)),
        Ok(None) => Json(ApiResponse::error("Game not found")),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            Json(ApiResponse::error("Internal server error"))
        }
    }
}
```

### Input Validation

```rust
const MAX_SEARCH_QUERY_LENGTH: usize = 200;
const MIN_SEARCH_QUERY_LENGTH: usize = 1;

pub async fn search_games(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Json<ApiResponse<Vec<GameSummary>>> {
    let q = query.q.trim();

    if q.len() < MIN_SEARCH_QUERY_LENGTH {
        return Json(ApiResponse::error("Search query too short"));
    }
    if q.len() > MAX_SEARCH_QUERY_LENGTH {
        return Json(ApiResponse::error("Search query too long"));
    }

    // ... proceed with search
}
```

### Batch Processing

```rust
const ENRICHMENT_BATCH_SIZE: usize = 20;
const STEAM_API_RATE_LIMIT_MS: u64 = 500;

pub async fn enrich_games(State(state): State<Arc<AppState>>) {
    let games = db::get_games_needing_enrichment(&state.db).await?;

    for game in games.iter().take(ENRICHMENT_BATCH_SIZE) {
        // Search Steam
        let (app_id, confidence) = steam::search_steam_app(&client, &game.title).await?;

        // Rate limit
        tokio::time::sleep(Duration::from_millis(STEAM_API_RATE_LIMIT_MS)).await;

        // Fetch details
        let details = steam::fetch_steam_details(&client, app_id).await;

        // Update database
        db::update_game_steam_data(&state.db, game.id, ...).await?;
    }
}
```

### Binary Response

```rust
pub async fn serve_game_cover(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> axum::response::Response {
    use axum::http::{header, StatusCode};
    use axum::response::IntoResponse;

    let folder_path = db::get_game_folder_path(&state.db, id).await?;
    let cover_path = local_storage::get_cover_path(&folder_path);

    if !cover_path.exists() {
        return (StatusCode::NOT_FOUND, "Not found").into_response();
    }

    let bytes = std::fs::read(&cover_path)?;

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/jpeg")],
        bytes,
    ).into_response()
}
```

### Dual-Write

```rust
pub async fn update_game(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateGameRequest>,
) -> Json<ApiResponse<Game>> {
    // 1. Update database (primary)
    let game = db::update_game_metadata(&state.db, id, ...).await?;

    // 2. Write to local file (backup)
    if let Err(e) = local_storage::save_game_metadata(&game) {
        tracing::warn!("Failed to save metadata.json: {}", e);
        // Don't fail - DB update succeeded
    }

    Json(ApiResponse::success(game))
}
```

## Error Handling

All handlers follow this pattern:

```rust
match operation().await {
    Ok(result) => Json(ApiResponse::success(result)),
    Err(e) => {
        // Log the actual error (for debugging)
        tracing::error!("Operation failed: {}", e);
        // Return sanitized error to client
        Json(ApiResponse::error("Operation failed"))
    }
}
```

## Shutdown/Restart

```rust
pub async fn shutdown_server() -> Json<ApiResponse<&'static str>> {
    tokio::spawn(async {
        // Wait for response to be sent
        tokio::time::sleep(Duration::from_millis(500)).await;
        std::process::exit(0);
    });

    Json(ApiResponse::success("Shutting down..."))
}

pub async fn restart_server() -> Json<ApiResponse<&'static str>> {
    let exe_path = std::env::current_exe()?;

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(500)).await;
        std::process::Command::new(&exe_path).spawn()?;
        std::process::exit(0);
    });

    Json(ApiResponse::success("Restarting..."))
}
```
