---
sidebar_position: 1
---

# Backend Modules

The GameVault backend is organized into focused modules, each handling a specific concern.

## Module Map

```
backend/src/
├── main.rs          # Application entry and server setup
├── config.rs        # TOML configuration
├── db.rs            # Database operations
├── handlers.rs      # HTTP endpoint handlers
├── scanner.rs       # Directory scanning
├── steam.rs         # Steam API client
├── local_storage.rs # File I/O operations
├── embedded.rs      # Static asset serving
├── models.rs        # Data structures
└── tray.rs          # System tray (Windows)
```

## Module Dependencies

```
                    ┌──────────┐
                    │  main.rs │
                    └────┬─────┘
          ┌──────────────┼──────────────┐
          │              │              │
          ▼              ▼              ▼
    ┌──────────┐  ┌──────────┐  ┌──────────┐
    │ config   │  │ handlers │  │ embedded │
    └──────────┘  └────┬─────┘  └──────────┘
                       │
         ┌─────────────┼─────────────┐
         │             │             │
         ▼             ▼             ▼
   ┌──────────┐  ┌──────────┐  ┌──────────┐
   │ scanner  │  │    db    │  │  steam   │
   └──────────┘  └──────────┘  └──────────┘
                       │
                       ▼
               ┌──────────────┐
               │local_storage │
               └──────────────┘
```

## Module Responsibilities

### main.rs
- Tokio async runtime initialization
- Logging (tracing) setup
- Database connection pool
- Router configuration
- CORS and authentication middleware
- System tray initialization

### config.rs
- TOML file parsing
- Environment variable overrides
- Path resolution (relative to executable)
- Atomic config file updates

### db.rs
- SQLite connection management
- Schema migrations
- CRUD operations
- Transaction handling
- Query builders

### handlers.rs
- HTTP request parsing
- Input validation
- Business logic orchestration
- Response formatting
- Error handling

### scanner.rs
- Directory traversal
- Folder name cleanup (regex)
- Non-game content filtering
- Size estimation

### steam.rs
- Steam Store API client
- Title fuzzy matching
- Rate limiting
- Response parsing

### local_storage.rs
- Metadata file I/O
- Image caching
- Backup management
- Dual-write coordination

### embedded.rs
- rust-embed asset serving
- MIME type detection
- Cache headers
- SPA fallback routing

### models.rs
- Data structures (Game, Stats, etc.)
- Serde serialization
- SQLx row mapping
- Type conversions

### tray.rs
- Windows system tray icon
- Menu commands
- Event handling
- Process lifecycle

## Data Flow Example

### Edit Game Metadata

```
1. PUT /api/games/:id
   └─▶ handlers::update_game()

2. Parse and validate request body
   └─▶ UpdateGameRequest struct

3. Update database
   └─▶ db::update_game_metadata()
       └─▶ BEGIN TRANSACTION
       └─▶ UPDATE games SET ...
       └─▶ SELECT * FROM games WHERE id = ?
       └─▶ COMMIT

4. Write to local file
   └─▶ local_storage::save_game_metadata()
       └─▶ Write .gamevault/metadata.json

5. Return updated game
   └─▶ Json<ApiResponse<Game>>
```

## Error Handling

Each module follows consistent patterns:

```rust
// Return Result for fallible operations
pub async fn get_game(pool: &SqlitePool, id: i64) -> Result<Option<Game>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM games WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

// Log errors, return sanitized messages to clients
match db::get_game(&state.db, id).await {
    Ok(Some(game)) => Json(ApiResponse::success(game)),
    Ok(None) => Json(ApiResponse::error("Game not found")),
    Err(e) => {
        tracing::error!("Database error: {}", e);
        Json(ApiResponse::error("Internal server error"))
    }
}
```

## Configuration

Modules access configuration via:

```rust
// Environment variables (legacy)
std::env::var("DATABASE_URL")
std::env::var("GAMES_PATH")

// Config struct (preferred)
let config = AppConfig::load()?;
config.games_path()
config.database_url()
config.server.port
```

## Testing

Each module has inline tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_title() {
        assert_eq!(
            clean_title("Cyberpunk 2077 [FitGirl Repack]"),
            "Cyberpunk 2077"
        );
    }
}
```

Run with:

```bash
cargo test
```
