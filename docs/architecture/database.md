---
sidebar_position: 4
---

# Database Schema

GameVault uses SQLite for data persistence, with the database file stored at `./data/gamevault.db`.

## Tables

### games

The main table storing all game information.

```sql
CREATE TABLE games (
    id                    INTEGER PRIMARY KEY AUTOINCREMENT,

    -- Folder information (hidden from API)
    folder_path           TEXT NOT NULL UNIQUE,
    folder_name           TEXT NOT NULL,
    title                 TEXT NOT NULL,

    -- External IDs
    igdb_id               INTEGER,
    steam_app_id          INTEGER,

    -- Basic metadata
    summary               TEXT,
    release_date          TEXT,

    -- Image URLs (CDN fallback)
    cover_url             TEXT,
    background_url        TEXT,

    -- Local cached images
    local_cover_path      TEXT,
    local_background_path TEXT,

    -- JSON metadata
    genres                TEXT,  -- JSON array of strings
    developers            TEXT,  -- JSON array of strings
    publishers            TEXT,  -- JSON array of strings

    -- Review data
    review_score          INTEGER,
    review_count          INTEGER,
    review_summary        TEXT,
    review_score_recent   INTEGER,
    review_count_recent   INTEGER,

    -- Technical info
    size_bytes            INTEGER,

    -- Matching
    match_confidence      REAL,
    match_status          TEXT NOT NULL DEFAULT 'pending',

    -- User state
    user_status           TEXT,
    playtime_mins         INTEGER,
    match_locked          INTEGER,

    -- HowLongToBeat data
    hltb_main_mins        INTEGER,
    hltb_extra_mins       INTEGER,
    hltb_completionist_mins INTEGER,

    -- Save backup
    save_path_pattern     TEXT,

    -- Manual edit tracking
    manually_edited       INTEGER DEFAULT 0,

    -- Timestamps
    created_at            TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at            TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### schema_version

Tracks database migration state:

```sql
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY
);
```

## Column Details

### Match Status Values

| Status | Description |
|--------|-------------|
| `pending` | Not yet matched to Steam |
| `matched` | Successfully matched |
| `failed` | Matching attempted but failed |
| `manual` | Manually matched by user |

### JSON Fields

Genres, developers, and publishers are stored as JSON arrays:

```json
// genres column
["Action", "RPG", "Open World"]

// developers column
["CD Projekt Red"]

// publishers column
["CD Projekt"]
```

### Image Paths

| Column | Purpose | Example |
|--------|---------|---------|
| `cover_url` | Steam CDN URL | `https://steamcdn-a.akamaihd.net/...` |
| `background_url` | Steam background | `https://steamcdn-a.akamaihd.net/...` |
| `local_cover_path` | Cached locally | `.gamevault/cover.jpg` |
| `local_background_path` | Cached locally | `.gamevault/background.jpg` |

## Indexes

```sql
CREATE INDEX idx_games_title ON games(title);
CREATE INDEX idx_games_match_status ON games(match_status);
CREATE INDEX idx_games_steam_app_id ON games(steam_app_id);
```

## Common Queries

### List All Games

```sql
SELECT * FROM games ORDER BY title ASC;
```

### Search by Title

```sql
SELECT * FROM games
WHERE title LIKE '%' || ? || '%'
ORDER BY title ASC;
```

### Games Needing Enrichment

```sql
SELECT * FROM games
WHERE steam_app_id IS NULL
  AND match_status = 'pending'
ORDER BY title ASC;
```

### Get Statistics

```sql
SELECT
    COUNT(*) as total_games,
    SUM(CASE WHEN match_status = 'matched' THEN 1 ELSE 0 END) as matched_games,
    SUM(CASE WHEN match_status = 'pending' THEN 1 ELSE 0 END) as pending_games,
    SUM(CASE WHEN steam_app_id IS NOT NULL THEN 1 ELSE 0 END) as enriched_games
FROM games;
```

### Update with Transaction

```sql
BEGIN TRANSACTION;

UPDATE games SET
    title = ?,
    summary = ?,
    genres = ?,
    developers = ?,
    publishers = ?,
    release_date = ?,
    review_score = ?,
    manually_edited = 1,
    updated_at = datetime('now')
WHERE id = ?;

SELECT * FROM games WHERE id = ?;

COMMIT;
```

## Data Types

### Rust ↔ SQLite Mapping

| Rust Type | SQLite Type | Notes |
|-----------|-------------|-------|
| `i64` | INTEGER | Primary keys, IDs |
| `String` | TEXT | All text fields |
| `Option<T>` | NULL or T | Nullable columns |
| `f64` | REAL | Confidence scores |
| `Vec<String>` | TEXT (JSON) | Serialized as JSON |

### SQLx Query Mapping

```rust
#[derive(sqlx::FromRow)]
pub struct Game {
    pub id: i64,
    pub title: String,
    pub steam_app_id: Option<i64>,
    pub match_confidence: Option<f64>,
    // ...
}
```

## Migrations

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

### Adding Columns

```rust
if version < 2 {
    sqlx::query("ALTER TABLE games ADD COLUMN manually_edited INTEGER DEFAULT 0")
        .execute(pool)
        .await?;
    set_schema_version(pool, 2).await;
}
```

## Security Considerations

### Hidden Columns

`folder_path` and `folder_name` are excluded from API responses:

```rust
#[derive(Serialize)]
pub struct Game {
    // ...
    #[serde(skip_serializing)]
    pub folder_path: String,
    #[serde(skip_serializing)]
    pub folder_name: String,
    // ...
}
```

### SQL Injection Prevention

All queries use parameterized statements:

```rust
// Safe - parameterized
sqlx::query("SELECT * FROM games WHERE id = ?")
    .bind(id)
    .fetch_one(pool)
    .await?;

// Never do this:
// format!("SELECT * FROM games WHERE id = {}", user_input)
```

## Backup Strategy

### Database Backup

Copy the database file when GameVault is not running:

```powershell
Copy-Item .\data\gamevault.db .\backup\gamevault-backup.db
```

### Metadata Backup

Use the Export feature to write `.gamevault/metadata.json` files to each game folder, providing redundant storage independent of the database.
