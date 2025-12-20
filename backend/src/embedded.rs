//! Embedded static assets for portable executable
//!
//! Uses rust-embed to bundle the Next.js static export into the binary.
//! Falls back to filesystem serving in development mode.

use axum::{
    body::Body,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use rust_embed::RustEmbed;

/// Embedded static assets from Next.js build
/// The folder path is relative to the Cargo.toml (backend/) directory
#[derive(RustEmbed)]
#[folder = "../frontend/out/"]
#[prefix = ""]
pub struct StaticAssets;

/// Serve embedded static files
pub async fn serve_static(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    // Handle root path
    let path = if path.is_empty() { "index.html" } else { path };

    tracing::debug!("Serving static file: {}", path);

    // Try exact path first
    if let Some(content) = StaticAssets::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime.as_ref())
            .header(header::CACHE_CONTROL, get_cache_control(path))
            .body(Body::from(content.data.into_owned()))
            .unwrap();
    }

    // Try with .html extension (Next.js static export format)
    let html_path = format!("{}.html", path.trim_end_matches('/'));
    if let Some(content) = StaticAssets::get(&html_path) {
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .header(header::CACHE_CONTROL, "no-cache")
            .body(Body::from(content.data.into_owned()))
            .unwrap();
    }

    // Try with /index.html for directory paths
    let index_path = format!("{}/index.html", path.trim_end_matches('/'));
    if let Some(content) = StaticAssets::get(&index_path) {
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .header(header::CACHE_CONTROL, "no-cache")
            .body(Body::from(content.data.into_owned()))
            .unwrap();
    }

    // Fallback to root index.html for SPA client-side routing
    if let Some(content) = StaticAssets::get("index.html") {
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .header(header::CACHE_CONTROL, "no-cache")
            .body(Body::from(content.data.into_owned()))
            .unwrap();
    }

    // Nothing found
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(Body::from("Not Found"))
        .unwrap()
}

/// Get appropriate cache control header based on file type
fn get_cache_control(path: &str) -> &'static str {
    // Static assets with hash in filename can be cached forever
    if path.contains("/_next/static/") {
        "public, max-age=31536000, immutable"
    }
    // Fonts can be cached
    else if path.ends_with(".woff2") || path.ends_with(".woff") || path.ends_with(".ttf") {
        "public, max-age=31536000, immutable"
    }
    // Images can be cached
    else if path.ends_with(".png")
        || path.ends_with(".jpg")
        || path.ends_with(".jpeg")
        || path.ends_with(".gif")
        || path.ends_with(".svg")
        || path.ends_with(".ico")
    {
        "public, max-age=86400"
    }
    // HTML and other files should not be cached
    else {
        "no-cache"
    }
}

/// Check if embedded assets are available (for conditional compilation)
pub fn has_embedded_assets() -> bool {
    StaticAssets::get("index.html").is_some()
}

/// List all embedded files (for debugging)
#[allow(dead_code)]
pub fn list_embedded_files() -> Vec<String> {
    StaticAssets::iter().map(|f| f.to_string()).collect()
}
