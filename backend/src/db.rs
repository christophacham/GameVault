use sqlx::{Row, SqlitePool};

use crate::models::{Game, Stats};

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS games (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    folder_path TEXT NOT NULL UNIQUE,
    folder_name TEXT NOT NULL,
    title TEXT NOT NULL,

    igdb_id INTEGER,
    steam_app_id INTEGER,

    summary TEXT,
    release_date TEXT,

    cover_url TEXT,
    background_url TEXT,

    -- Local cached images (stored in .gamevault/ within game folder)
    local_cover_path TEXT,
    local_background_path TEXT,

    genres TEXT,
    developers TEXT,
    publishers TEXT,

    review_score INTEGER,
    review_count INTEGER,
    review_summary TEXT,

    review_score_recent INTEGER,
    review_count_recent INTEGER,

    size_bytes INTEGER,

    match_confidence REAL,
    match_status TEXT NOT NULL DEFAULT 'pending',

    -- User state
    user_status TEXT DEFAULT 'unplayed',
    playtime_mins INTEGER DEFAULT 0,
    match_locked INTEGER DEFAULT 0,

    -- HLTB data
    hltb_main_mins INTEGER,
    hltb_extra_mins INTEGER,
    hltb_completionist_mins INTEGER,

    -- Save backup pattern
    save_path_pattern TEXT,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_games_title ON games(title);
CREATE INDEX IF NOT EXISTS idx_games_match_status ON games(match_status);
CREATE INDEX IF NOT EXISTS idx_games_steam_app_id ON games(steam_app_id);
"#;

/// Migration to add new columns to existing databases
const MIGRATIONS: &[&str] = &[
    "ALTER TABLE games ADD COLUMN local_cover_path TEXT",
    "ALTER TABLE games ADD COLUMN local_background_path TEXT",
    "ALTER TABLE games ADD COLUMN user_status TEXT DEFAULT 'unplayed'",
    "ALTER TABLE games ADD COLUMN playtime_mins INTEGER DEFAULT 0",
    "ALTER TABLE games ADD COLUMN match_locked INTEGER DEFAULT 0",
    "ALTER TABLE games ADD COLUMN hltb_main_mins INTEGER",
    "ALTER TABLE games ADD COLUMN hltb_extra_mins INTEGER",
    "ALTER TABLE games ADD COLUMN hltb_completionist_mins INTEGER",
    "ALTER TABLE games ADD COLUMN save_path_pattern TEXT",
    "ALTER TABLE games ADD COLUMN manually_edited INTEGER DEFAULT 0",
];

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Enable WAL mode for better concurrent access
    sqlx::query("PRAGMA journal_mode=WAL").execute(pool).await?;

    sqlx::query(SCHEMA).execute(pool).await?;

    // Run migrations for existing databases (ignore errors for already-existing columns)
    for migration in MIGRATIONS {
        let _ = sqlx::query(migration).execute(pool).await;
    }

    Ok(())
}

pub async fn upsert_game(
    pool: &SqlitePool,
    folder_path: &str,
    folder_name: &str,
    title: &str,
    size_bytes: Option<i64>,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO games (folder_path, folder_name, title, size_bytes, match_status)
        VALUES (?, ?, ?, ?, 'pending')
        ON CONFLICT(folder_path) DO UPDATE SET
            folder_name = excluded.folder_name,
            title = excluded.title,
            size_bytes = COALESCE(excluded.size_bytes, games.size_bytes),
            updated_at = datetime('now')
        RETURNING id
        "#,
    )
    .bind(folder_path)
    .bind(folder_name)
    .bind(title)
    .bind(size_bytes)
    .fetch_one(pool)
    .await?;

    Ok(result.get("id"))
}

pub async fn get_all_games(pool: &SqlitePool) -> Result<Vec<Game>, sqlx::Error> {
    sqlx::query_as::<_, Game>("SELECT * FROM games ORDER BY title")
        .fetch_all(pool)
        .await
}

pub async fn get_game_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Game>, sqlx::Error> {
    sqlx::query_as::<_, Game>("SELECT * FROM games WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn search_games(pool: &SqlitePool, query: &str) -> Result<Vec<Game>, sqlx::Error> {
    let pattern = format!("%{}%", query);
    sqlx::query_as::<_, Game>("SELECT * FROM games WHERE title LIKE ? ORDER BY title LIMIT 50")
        .bind(pattern)
        .fetch_all(pool)
        .await
}

/// Get games that need enrichment:
/// - Pending games (not yet matched to Steam)
/// - Games missing local images (matched but image caching failed)
pub async fn get_games_needing_enrichment(pool: &SqlitePool) -> Result<Vec<Game>, sqlx::Error> {
    sqlx::query_as::<_, Game>(
        "SELECT * FROM games WHERE (match_status = 'pending' OR steam_app_id IS NULL) OR (match_status = 'matched' AND (local_cover_path IS NULL OR local_background_path IS NULL)) ORDER BY title"
    )
    .fetch_all(pool)
    .await
}

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
    sqlx::query(
        r#"
        UPDATE games SET
            steam_app_id = ?,
            summary = COALESCE(?, summary),
            cover_url = COALESCE(?, cover_url),
            background_url = COALESCE(?, background_url),
            genres = COALESCE(?, genres),
            developers = COALESCE(?, developers),
            publishers = COALESCE(?, publishers),
            release_date = COALESCE(?, release_date),
            match_confidence = ?,
            match_status = 'matched',
            updated_at = datetime('now')
        WHERE id = ?
        "#,
    )
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

pub async fn update_game_reviews(
    pool: &SqlitePool,
    id: i64,
    review_score: i64,
    review_count: i64,
    review_summary: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE games SET
            review_score = ?,
            review_count = ?,
            review_summary = ?,
            updated_at = datetime('now')
        WHERE id = ?
        "#,
    )
    .bind(review_score)
    .bind(review_count)
    .bind(review_summary)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Update game metadata from imported JSON file
pub async fn update_game_from_import(
    pool: &SqlitePool,
    id: i64,
    steam_app_id: Option<i64>,
    summary: Option<&str>,
    genres: Option<&str>,
    developers: Option<&str>,
    publishers: Option<&str>,
    release_date: Option<&str>,
    review_score: Option<i64>,
    review_summary: Option<&str>,
    hltb_main: Option<i64>,
    hltb_extra: Option<i64>,
    hltb_completionist: Option<i64>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE games SET
            steam_app_id = COALESCE(?, steam_app_id),
            summary = COALESCE(?, summary),
            genres = COALESCE(?, genres),
            developers = COALESCE(?, developers),
            publishers = COALESCE(?, publishers),
            release_date = COALESCE(?, release_date),
            review_score = COALESCE(?, review_score),
            review_summary = COALESCE(?, review_summary),
            hltb_main_mins = COALESCE(?, hltb_main_mins),
            hltb_extra_mins = COALESCE(?, hltb_extra_mins),
            hltb_completionist_mins = COALESCE(?, hltb_completionist_mins),
            match_status = CASE WHEN ? IS NOT NULL THEN 'matched' ELSE match_status END,
            updated_at = datetime('now')
        WHERE id = ?
        "#,
    )
    .bind(steam_app_id)
    .bind(summary)
    .bind(genres)
    .bind(developers)
    .bind(publishers)
    .bind(release_date)
    .bind(review_score)
    .bind(review_summary)
    .bind(hltb_main)
    .bind(hltb_extra)
    .bind(hltb_completionist)
    .bind(steam_app_id)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_stats(pool: &SqlitePool) -> Result<Stats, sqlx::Error> {
    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM games")
        .fetch_one(pool)
        .await?;

    let matched: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM games WHERE match_status = 'matched'")
            .fetch_one(pool)
            .await?;

    let pending: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM games WHERE match_status = 'pending'")
            .fetch_one(pool)
            .await?;

    let enriched: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM games WHERE steam_app_id IS NOT NULL")
            .fetch_one(pool)
            .await?;

    Ok(Stats {
        total_games: total.0,
        matched_games: matched.0,
        pending_games: pending.0,
        enriched_games: enriched.0,
    })
}

/// Update local image paths for a game
pub async fn update_game_local_images(
    pool: &SqlitePool,
    id: i64,
    local_cover_path: Option<&str>,
    local_background_path: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE games SET
            local_cover_path = COALESCE(?, local_cover_path),
            local_background_path = COALESCE(?, local_background_path),
            updated_at = datetime('now')
        WHERE id = ?
        "#,
    )
    .bind(local_cover_path)
    .bind(local_background_path)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get recently added games
pub async fn get_recent_games(pool: &SqlitePool, limit: i64) -> Result<Vec<Game>, sqlx::Error> {
    sqlx::query_as::<_, Game>("SELECT * FROM games ORDER BY created_at DESC LIMIT ?")
        .bind(limit)
        .fetch_all(pool)
        .await
}

/// Get game by ID with folder path (for internal use)
pub async fn get_game_folder_path(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<String>, sqlx::Error> {
    let result: Option<(String,)> = sqlx::query_as("SELECT folder_path FROM games WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(result.map(|r| r.0))
}

/// Update game metadata from user edits
/// Returns the updated Game for dual-write to metadata.json
/// Uses a transaction to ensure atomicity of UPDATE + SELECT
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
    let mut tx = pool.begin().await?;

    sqlx::query(
        r#"
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
        "#,
    )
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

    // Fetch the updated game within the same transaction
    let game = sqlx::query_as::<_, Game>("SELECT * FROM games WHERE id = ?")
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(game)
}
