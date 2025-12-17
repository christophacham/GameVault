mod db;
mod handlers;
mod models;
mod scanner;
mod steam;

use axum::{
    routing::{get, post},
    Router,
    middleware,
    extract::Request,
    response::{Response, IntoResponse},
    http::StatusCode,
};
use axum::body::Body;
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use axum::http::{HeaderValue, Method, header::CONTENT_TYPE};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub struct AppState {
    pub db: sqlx::SqlitePool,
    pub games_path: String,
}

/// SECURITY: Optional API key authentication middleware
/// Set API_KEY env var to enable authentication on sensitive endpoints
async fn auth_middleware(request: Request, next: axum::middleware::Next) -> Response {
    // If no API_KEY is configured, allow all requests (backwards compatible)
    let api_key = match std::env::var("API_KEY") {
        Ok(key) if !key.is_empty() => key,
        _ => return next.run(request).await,
    };

    // Check Authorization header
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    match auth_header {
        Some(header) if header == format!("Bearer {}", api_key) => {
            next.run(request).await
        }
        Some(header) if header == api_key => {
            // Also accept raw API key without Bearer prefix
            next.run(request).await
        }
        _ => {
            tracing::warn!("Unauthorized API request - invalid or missing API key");
            (StatusCode::UNAUTHORIZED, "Unauthorized: Invalid or missing API key").into_response()
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting GameVault server...");

    // Get configuration from environment
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./data/games.db?mode=rwc".to_string());
    let games_path = std::env::var("GAMES_PATH")
        .unwrap_or_else(|_| "/games".to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);
    // SECURITY: Default to localhost only - use HOST=0.0.0.0 to expose to network
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

    tracing::info!("Database URL: {}", database_url);
    tracing::info!("Games path: {}", games_path);

    // Create database pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    tracing::info!("Database connected");

    // Run migrations
    db::run_migrations(&pool).await?;
    tracing::info!("Migrations complete");

    // Create app state
    let state = Arc::new(AppState {
        db: pool,
        games_path,
    });

    // SECURITY: CORS configuration - restrict to localhost by default
    // Set CORS_ORIGINS env var to allow additional origins (comma-separated)
    let cors = {
        let default_origins = vec![
            "http://localhost:3000".parse::<HeaderValue>().unwrap(),
            "http://127.0.0.1:3000".parse::<HeaderValue>().unwrap(),
            "http://localhost:5173".parse::<HeaderValue>().unwrap(),  // Vite dev server
            "http://127.0.0.1:5173".parse::<HeaderValue>().unwrap(),
        ];

        let origins: Vec<HeaderValue> = std::env::var("CORS_ORIGINS")
            .map(|s| {
                s.split(',')
                    .filter_map(|origin| origin.trim().parse().ok())
                    .collect()
            })
            .unwrap_or(default_origins);

        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers([CONTENT_TYPE])
    };

    // Build API routes (order matters - specific routes before parameterized)
    // SECURITY: /scan and /enrich require API_KEY if configured
    let protected_routes = Router::new()
        .route("/scan", post(handlers::scan_games))
        .route("/enrich", post(handlers::enrich_games))
        .layer(middleware::from_fn(auth_middleware));

    let api_routes = Router::new()
        .route("/health", get(handlers::health))
        .route("/games", get(handlers::list_games))
        .route("/games/search", get(handlers::search_games))
        .route("/games/:id", get(handlers::get_game))
        .route("/stats", get(handlers::get_stats))
        .merge(protected_routes)
        .with_state(state);

    // Build main router - serve static files and API
    let app = Router::new()
        .nest("/api", api_routes)
        .fallback_service(
            ServeDir::new("public")
                .append_index_html_on_directories(true)
                .not_found_service(ServeFile::new("public/index.html"))
        )
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let addr = format!("{}:{}", host, port);
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
