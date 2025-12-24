---
sidebar_position: 2
---

# Backend Architecture

The GameVault backend is built with Rust using the Axum web framework, providing a fast, memory-safe server with async I/O.

## Module Structure

```
backend/src/
├── main.rs          # Entry point, router setup, middleware
├── config.rs        # TOML configuration loading
├── db.rs            # SQLite database operations
├── handlers.rs      # API endpoint handlers
├── scanner.rs       # Game folder scanning
├── steam.rs         # Steam API client
├── local_storage.rs # Local file operations
├── embedded.rs      # Static asset serving
├── models.rs        # Data structures
└── tray.rs          # Windows system tray
```

## main.rs - Server Entry Point

The main module bootstraps the application:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize logging (tracing)
    // 2. Load configuration (TOML + env vars)
    // 3. Connect to SQLite database
    // 4. Run migrations
    // 5. Configure CORS and authentication
    // 6. Build router with API routes
    // 7. Start system tray (Windows)
    // 8. Listen on configured port
}
```

### Router Structure

```rust
let api_routes = Router::new()
    // Public endpoints
    .route("/health", get(health))
    .route("/games", get(list_games))
    .route("/games/search", get(search_games))
    .route("/games/:id", get(get_game))
    .route("/stats", get(get_stats))

    // Protected endpoints (require API_KEY if set)
    .merge(protected_routes)

    // Config endpoints
    .merge(config_routes);

let app = Router::new()
    .nest("/api", api_routes)
    .fallback(serve_static)  // Embedded frontend
    .layer(cors)
    .layer(TraceLayer::new_for_http());
```

### Authentication Middleware

Optional API key authentication for sensitive endpoints:

```rust
async fn auth_middleware(request: Request, next: Next) -> Response {
    let api_key = match std::env::var("API_KEY") {
        Ok(key) if !key.is_empty() => key,
        _ => return next.run(request).await,  // No auth required
    };

    // Check Authorization header
    match auth_header {
        Some(h) if h == format!("Bearer {}", api_key) => next.run(request).await,
        Some(h) if h == api_key => next.run(request).await,
        _ => (StatusCode::UNAUTHORIZED, "Invalid API key").into_response(),
    }
}
```

## handlers.rs - API Endpoints

### Endpoint Categories

| Category | Endpoints | Auth Required |
|----------|-----------|---------------|
| **Health** | `GET /health` | No |
| **Games** | `GET /games`, `/games/:id`, `/games/search` | No |
| **Scan** | `POST /scan` | Yes* |
| **Enrich** | `POST /enrich` | Yes* |
| **Edit** | `PUT /games/:id` | Yes* |
| **Rematch** | `POST /games/:id/match`, `/match/confirm` | Yes* |
| **Import/Export** | `POST /import`, `/export` | Yes* |
| **Config** | `GET/PUT /config`, `/shutdown`, `/restart` | No |

*Only when `API_KEY` environment variable is set.

### Key Handler Patterns

**List with Transform**
```rust
pub async fn list_games(State(state): State<Arc<AppState>>)
    -> Json<ApiResponse<Vec<GameSummary>>>
{
    match db::get_all_games(&state.db).await {
        Ok(games) => {
            let summaries: Vec<GameSummary> = games.into_iter()
                .map(|g| g.into())
                .collect();
            Json(ApiResponse::success(summaries))
        }
        Err(e) => Json(ApiResponse::error("Internal server error"))
    }
}
```

**Input Validation**
```rust
const MAX_SEARCH_QUERY_LENGTH: usize = 200;
const MIN_SEARCH_QUERY_LENGTH: usize = 1;

pub async fn search_games(...) {
    let query = query.q.trim();
    if query.len() < MIN_SEARCH_QUERY_LENGTH {
        return Json(ApiResponse::error("Search query too short"));
    }
    // ...
}
```

## db.rs - Database Operations

### Connection Pool

```rust
let pool = SqlitePoolOptions::new()
    .max_connections(5)
    .connect(&database_url)
    .await?;
```

### Transaction Pattern

Atomic updates with transaction:

```rust
pub async fn update_game_metadata(
    pool: &SqlitePool,
    id: i64,
    // ... fields
) -> Result<Game, sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query("UPDATE games SET ... WHERE id = ?")
        .execute(&mut *tx)
        .await?;

    let game = sqlx::query_as::<_, Game>("SELECT * FROM games WHERE id = ?")
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(game)
}
```

### Migration System

```rust
pub async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS games (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            folder_path TEXT NOT NULL UNIQUE,
            folder_name TEXT NOT NULL,
            title TEXT NOT NULL,
            -- ... 25+ columns
        )
    "#).execute(pool).await?;

    // Schema version tracking
    // ALTER TABLE migrations for new columns
}
```

## scanner.rs - Directory Scanning

### Folder Name Cleanup

Regex patterns remove common suffixes:

```rust
const CLEANUP_PATTERNS: &[&str] = &[
    r"\[FitGirl.*?\]",
    r"\[DODI.*?\]",
    r"\[.*?Repack.*?\]",
    r"\s*v\d+(\.\d+)*\w*",
    r"\s*\(.*?\)",
    // ... more patterns
];
```

### Exclusion Patterns

Skip non-game content:

```rust
const EXCLUSION_PATTERNS: &[&str] = &[
    r"(?i)\[BluRay\]",
    r"(?i)\[720p\]",
    r"(?i)\.mkv$",
    r"(?i)S\d{2}E\d{2}",  // TV shows
    // ...
];
```

## steam.rs - Steam API Client

### API Endpoints Used

| Endpoint | Purpose |
|----------|---------|
| `store.steampowered.com/api/storesearch` | Search by title |
| `store.steampowered.com/api/appdetails` | Game metadata |
| `store.steampowered.com/appreviews` | Review scores |

### Rate Limiting

```rust
const ENRICHMENT_BATCH_SIZE: usize = 20;
const STEAM_API_RATE_LIMIT_MS: u64 = 500;

// Between each API call
tokio::time::sleep(Duration::from_millis(STEAM_API_RATE_LIMIT_MS)).await;
```

### Fuzzy Matching

Uses `strsim` crate for Jaro-Winkler similarity:

```rust
let similarity = strsim::jaro_winkler(&clean_title, &steam_name);
if similarity > 0.85 {
    // Auto-match
} else if similarity > 0.60 {
    // Manual review recommended
}
```

## local_storage.rs - File Operations

### Dual-Write System

```rust
pub fn save_game_metadata(game: &Game) -> Result<()> {
    let metadata_path = get_metadata_path(&game.folder_path);

    let metadata = ExportedMetadata {
        schema_version: 1,
        exported_at: Utc::now().to_rfc3339(),
        steam_app_id: game.steam_app_id,
        // ... all fields
    };

    // Write atomically
    let tmp_path = metadata_path.with_extension("json.tmp");
    std::fs::write(&tmp_path, serde_json::to_string_pretty(&metadata)?)?;
    std::fs::rename(tmp_path, metadata_path)?;

    Ok(())
}
```

### Image Caching

```rust
pub async fn cache_game_images(
    client: &reqwest::Client,
    folder_path: &str,
    cover_url: Option<&str>,
    background_url: Option<&str>,
) -> (Option<String>, Option<String>) {
    // Download to .gamevault/cover.jpg and background.jpg
    // Returns local paths for DB storage
}
```

## embedded.rs - Static Assets

### rust-embed Integration

```rust
#[derive(RustEmbed)]
#[folder = "../frontend/out/"]
#[prefix = ""]
pub struct StaticAssets;
```

### Static File Serving

```rust
pub async fn serve_static(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    // Try exact path
    if let Some(content) = StaticAssets::get(path) {
        return Response::builder()
            .header(CONTENT_TYPE, mime_guess::from_path(path))
            .header(CACHE_CONTROL, get_cache_control(path))
            .body(Body::from(content.data.into_owned()));
    }

    // Try .html extension (Next.js static export)
    // Try /index.html for directories
    // Fallback to index.html for SPA routing
}
```

## config.rs - Configuration

### TOML Structure

```toml
[paths]
game_library = "D:\\Games"
database = "sqlite:./data/gamevault.db?mode=rwc"
cache = "./cache"

[server]
port = 3000
auto_open_browser = true
bind_address = "127.0.0.1"
```

### Environment Variable Overrides

```rust
// Legacy (backwards compatible)
std::env::var("DATABASE_URL")
std::env::var("GAMES_PATH")
std::env::var("PORT")

// New format
std::env::var("GAMEVAULT_PATHS__GAME_LIBRARY")
std::env::var("GAMEVAULT_SERVER__PORT")
```

## Error Handling

### API Response Pattern

```rust
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self { /* ... */ }
    pub fn error(msg: impl Into<String>) -> Self { /* ... */ }
}
```

### Error Sanitization

Database errors are logged but not exposed to clients:

```rust
Err(e) => {
    tracing::error!("Database error: {}", e);
    Json(ApiResponse::error("Internal server error"))
}
```
