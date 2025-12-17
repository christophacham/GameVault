use sqlx::{SqlitePool, Row};
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

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_games_title ON games(title);
CREATE INDEX IF NOT EXISTS idx_games_match_status ON games(match_status);
CREATE INDEX IF NOT EXISTS idx_games_steam_app_id ON games(steam_app_id);
"#;

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Enable WAL mode for better concurrent access
    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(pool)
        .await?;

    sqlx::query(SCHEMA).execute(pool).await?;

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
    sqlx::query_as::<_, Game>(
        "SELECT * FROM games WHERE title LIKE ? ORDER BY title LIMIT 50"
    )
    .bind(pattern)
    .fetch_all(pool)
    .await
}

pub async fn get_pending_games(pool: &SqlitePool) -> Result<Vec<Game>, sqlx::Error> {
    sqlx::query_as::<_, Game>(
        "SELECT * FROM games WHERE match_status = 'pending' OR steam_app_id IS NULL ORDER BY title"
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

pub async fn get_stats(pool: &SqlitePool) -> Result<Stats, sqlx::Error> {
    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM games")
        .fetch_one(pool)
        .await?;

    let matched: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM games WHERE match_status = 'matched'")
        .fetch_one(pool)
        .await?;

    let pending: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM games WHERE match_status = 'pending'")
        .fetch_one(pool)
        .await?;

    let enriched: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM games WHERE steam_app_id IS NOT NULL")
        .fetch_one(pool)
        .await?;

    Ok(Stats {
        total_games: total.0,
        matched_games: matched.0,
        pending_games: pending.0,
        enriched_games: enriched.0,
    })
}
