---
sidebar_position: 3
---

# db.rs

Database operations using SQLx with SQLite.

## Connection Pool

```rust
let pool = SqlitePoolOptions::new()
    .max_connections(5)
    .connect(&database_url)
    .await?;
```

## Migrations

### Schema Creation

```rust
pub async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    // Create games table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS games (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            folder_path TEXT NOT NULL UNIQUE,
            folder_name TEXT NOT NULL,
            title TEXT NOT NULL,
            -- ... all columns
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )
    "#).execute(pool).await?;

    // Create indexes
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_games_title ON games(title)")
        .execute(pool).await?;

    // Version tracking
    sqlx::query("CREATE TABLE IF NOT EXISTS schema_version (version INTEGER PRIMARY KEY)")
        .execute(pool).await?;

    // Incremental migrations
    migrate_to_v2(pool).await?;
    migrate_to_v3(pool).await?;

    Ok(())
}
```

### Version Tracking

```rust
async fn get_schema_version(pool: &SqlitePool) -> i32 {
    sqlx::query_scalar("SELECT version FROM schema_version")
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .unwrap_or(0)
}

async fn set_schema_version(pool: &SqlitePool, version: i32) {
    sqlx::query("INSERT OR REPLACE INTO schema_version (version) VALUES (?)")
        .bind(version)
        .execute(pool)
        .await
        .ok();
}
```

## Query Functions

### Get All Games

```rust
pub async fn get_all_games(pool: &SqlitePool) -> Result<Vec<Game>, sqlx::Error> {
    sqlx::query_as::<_, Game>("SELECT * FROM games ORDER BY title ASC")
        .fetch_all(pool)
        .await
}
```

### Get Game by ID

```rust
pub async fn get_game_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Game>, sqlx::Error> {
    sqlx::query_as::<_, Game>("SELECT * FROM games WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}
```

### Search Games

```rust
pub async fn search_games(pool: &SqlitePool, query: &str) -> Result<Vec<Game>, sqlx::Error> {
    let pattern = format!("%{}%", query);
    sqlx::query_as::<_, Game>(
        "SELECT * FROM games WHERE title LIKE ? ORDER BY title ASC"
    )
        .bind(pattern)
        .fetch_all(pool)
        .await
}
```

### Upsert Game

```rust
pub async fn upsert_game(
    pool: &SqlitePool,
    folder_path: &str,
    folder_name: &str,
    title: &str,
    size_bytes: Option<i64>,
) -> Result<i64, sqlx::Error> {
    sqlx::query(r#"
        INSERT INTO games (folder_path, folder_name, title, size_bytes, match_status)
        VALUES (?, ?, ?, ?, 'pending')
        ON CONFLICT(folder_path) DO UPDATE SET
            folder_name = excluded.folder_name,
            title = CASE
                WHEN manually_edited = 1 THEN games.title
                ELSE excluded.title
            END,
            size_bytes = excluded.size_bytes,
            updated_at = datetime('now')
    "#)
        .bind(folder_path)
        .bind(folder_name)
        .bind(title)
        .bind(size_bytes)
        .execute(pool)
        .await
        .map(|r| r.last_insert_rowid())
}
```

### Update with Transaction

```rust
pub async fn update_game_metadata(
    pool: &SqlitePool,
    id: i64,
    title: Option<&str>,
    summary: Option<&str>,
    genres: Option<&str>,
    developers: Option<&str>,
    publishers: Option<&str>,
    release_date: Option<&str>,
    review_score: Option<i64>,
) -> Result<Game, sqlx::Error> {
    // Use transaction for atomic update + select
    let mut tx = pool.begin().await?;

    sqlx::query(r#"
        UPDATE games SET
            title = COALESCE(?, title),
            summary = COALESCE(?, summary),
            genres = COALESCE(?, genres),
            developers = COALESCE(?, developers),
            publishers = COALESCE(?, publishers),
            release_date = COALESCE(?, release_date),
            review_score = COALESCE(?, review_score),
            manually_edited = 1,
            updated_at = datetime('now')
        WHERE id = ?
    "#)
        .bind(title)
        .bind(summary)
        .bind(genres)
        .bind(developers)
        .bind(publishers)
        .bind(release_date)
        .bind(review_score)
        .bind(id)
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

### Get Games Needing Enrichment

```rust
pub async fn get_games_needing_enrichment(pool: &SqlitePool) -> Result<Vec<Game>, sqlx::Error> {
    sqlx::query_as::<_, Game>(r#"
        SELECT * FROM games
        WHERE steam_app_id IS NULL
          AND match_status = 'pending'
        ORDER BY title ASC
    "#)
        .fetch_all(pool)
        .await
}
```

### Update Steam Data

```rust
pub async fn update_game_steam_data(
    pool: &SqlitePool,
    id: i64,
    steam_app_id: i64,
    summary: Option<&str>,
    cover_url: Option<&str>,
    background_url: Option<&str>,
    genres: Option<&str>,
    developers: Option<&str>,
    publishers: Option<&str>,
    release_date: Option<&str>,
    match_confidence: f64,
) -> Result<(), sqlx::Error> {
    sqlx::query(r#"
        UPDATE games SET
            steam_app_id = ?,
            summary = ?,
            cover_url = ?,
            background_url = ?,
            genres = ?,
            developers = ?,
            publishers = ?,
            release_date = ?,
            match_confidence = ?,
            match_status = 'matched',
            updated_at = datetime('now')
        WHERE id = ?
    "#)
        .bind(steam_app_id)
        .bind(summary)
        .bind(cover_url)
        .bind(background_url)
        .bind(genres)
        .bind(developers)
        .bind(publishers)
        .bind(release_date)
        .bind(match_confidence)
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}
```

### Get Statistics

```rust
pub async fn get_stats(pool: &SqlitePool) -> Result<Stats, sqlx::Error> {
    let row = sqlx::query_as::<_, (i64, i64, i64, i64)>(r#"
        SELECT
            COUNT(*) as total_games,
            SUM(CASE WHEN match_status = 'matched' THEN 1 ELSE 0 END) as matched_games,
            SUM(CASE WHEN match_status = 'pending' THEN 1 ELSE 0 END) as pending_games,
            SUM(CASE WHEN steam_app_id IS NOT NULL THEN 1 ELSE 0 END) as enriched_games
        FROM games
    "#)
        .fetch_one(pool)
        .await?;

    Ok(Stats {
        total_games: row.0,
        matched_games: row.1,
        pending_games: row.2,
        enriched_games: row.3,
    })
}
```

## Error Handling

```rust
// Propagate errors to caller
pub async fn get_game(pool: &SqlitePool, id: i64) -> Result<Option<Game>, sqlx::Error> {
    sqlx::query_as("SELECT * FROM games WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

// Usage in handlers
match db::get_game(&state.db, id).await {
    Ok(Some(game)) => Json(ApiResponse::success(game)),
    Ok(None) => Json(ApiResponse::error("Game not found")),
    Err(e) => {
        tracing::error!("DB error: {}", e);
        Json(ApiResponse::error("Internal server error"))
    }
}
```

## Type Mapping

| Rust Type | SQLite Type | Notes |
|-----------|-------------|-------|
| `i64` | INTEGER | IDs, counts, scores |
| `String` | TEXT | All text fields |
| `Option<T>` | NULL | Nullable columns |
| `f64` | REAL | Confidence scores |
| `Vec<String>` | TEXT | JSON serialized |
