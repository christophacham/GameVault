//! Configuration management for GameVault portable executable
//!
//! Loads configuration from:
//! 1. Default values (built-in)
//! 2. config.toml next to executable
//! 3. Environment variables (GAMEVAULT_* prefix)

use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub paths: PathsConfig,
    pub server: ServerConfig,
}

/// Path configuration for data storage
#[derive(Debug, Deserialize, Clone)]
pub struct PathsConfig {
    /// Root directory containing games to scan
    pub game_library: PathBuf,
    /// SQLite database file path
    pub database: String,
    /// Cache directory for cover images
    pub cache: PathBuf,
}

/// Server configuration
#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    /// Port to listen on
    pub port: u16,
    /// Whether to auto-open browser on startup
    pub auto_open_browser: bool,
    /// Address to bind to
    pub bind_address: String,
}

impl AppConfig {
    /// Load configuration from file and environment
    pub fn load() -> Result<Self, ConfigError> {
        let exe_dir = get_exe_directory();
        let config_path = exe_dir.join("config.toml");

        tracing::info!("Looking for config at: {:?}", config_path);

        let config = Config::builder()
            // Default values
            .set_default("paths.game_library", ".")?
            .set_default("paths.database", "sqlite:./data/gamevault.db?mode=rwc")?
            .set_default("paths.cache", "./cache")?
            .set_default("server.port", 3000)?
            .set_default("server.auto_open_browser", true)?
            .set_default("server.bind_address", "127.0.0.1")?
            // Load from config file if it exists
            .add_source(File::from(config_path).required(false))
            // Environment variable overrides (GAMEVAULT_PATHS__GAME_LIBRARY, etc.)
            .add_source(
                config::Environment::with_prefix("GAMEVAULT")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;

        config.try_deserialize()
    }

    /// Get the database URL, resolving relative paths
    pub fn database_url(&self) -> String {
        let db_path = &self.paths.database;

        // If it's already a SQLite URL, use as-is
        if db_path.starts_with("sqlite:") {
            return db_path.clone();
        }

        // Otherwise, construct the URL from path
        let path = resolve_path(db_path);
        format!("sqlite:{}?mode=rwc", path.display())
    }

    /// Get the games path, resolving relative paths
    pub fn games_path(&self) -> PathBuf {
        resolve_path(&self.paths.game_library.to_string_lossy())
    }

    /// Get the cache path, resolving relative paths
    pub fn cache_path(&self) -> PathBuf {
        resolve_path(&self.paths.cache.to_string_lossy())
    }
}

/// Get the directory containing the executable
pub fn get_exe_directory() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Resolve a path relative to the executable directory
pub fn resolve_path(path: &str) -> PathBuf {
    let path_buf = PathBuf::from(path);

    // If absolute, use as-is
    if path_buf.is_absolute() {
        return path_buf;
    }

    // Resolve relative to exe directory
    get_exe_directory().join(path_buf)
}

/// Ensure required directories exist
pub fn ensure_directories(config: &AppConfig) -> anyhow::Result<()> {
    let exe_dir = get_exe_directory();

    // Create data directory for database
    let data_dir = exe_dir.join("data");
    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir)?;
        tracing::info!("Created data directory: {:?}", data_dir);
    }

    // Create cache directory
    let cache_dir = config.cache_path();
    if !cache_dir.exists() {
        std::fs::create_dir_all(&cache_dir)?;
        tracing::info!("Created cache directory: {:?}", cache_dir);
    }

    // Create logs directory
    let logs_dir = exe_dir.join("logs");
    if !logs_dir.exists() {
        std::fs::create_dir_all(&logs_dir)?;
        tracing::info!("Created logs directory: {:?}", logs_dir);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        // This should work with defaults even without a config file
        let config = AppConfig::load();
        assert!(config.is_ok());
    }
}
