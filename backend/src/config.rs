//! Configuration management for GameVault portable executable
//!
//! Loads configuration from:
//! 1. Default values (built-in)
//! 2. config.toml next to executable
//! 3. Environment variables (GAMEVAULT_* prefix)

use config::{Config, ConfigError, File};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    pub paths: PathsConfig,
    pub server: ServerConfig,
}

/// Path configuration for data storage
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PathsConfig {
    /// Root directory containing games to scan
    pub game_library: PathBuf,
    /// SQLite database file path
    pub database: String,
    /// Cache directory for cover images
    pub cache: PathBuf,
}

/// Server configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
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

/// Get the config file path
pub fn get_config_path() -> PathBuf {
    get_exe_directory().join("config.toml")
}

/// Write configuration to config.toml atomically
pub fn write_config(config: &AppConfig) -> anyhow::Result<()> {
    let config_path = get_config_path();
    let temp_path = get_exe_directory().join("config.toml.tmp");

    // Serialize to TOML
    let toml_string = toml::to_string_pretty(config)?;

    // Write to temp file first
    std::fs::write(&temp_path, &toml_string)?;

    // Atomic rename
    std::fs::rename(&temp_path, &config_path)?;

    tracing::info!("Configuration saved to {:?}", config_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_default_config() {
        // This should work with defaults even without a config file
        let config = AppConfig::load();
        assert!(config.is_ok());
    }

    #[test]
    fn test_default_port() {
        let config = AppConfig::load().unwrap();
        assert_eq!(config.server.port, 3000);
    }

    #[test]
    fn test_default_bind_address() {
        let config = AppConfig::load().unwrap();
        assert_eq!(config.server.bind_address, "127.0.0.1");
    }

    #[test]
    fn test_default_auto_open_browser() {
        let config = AppConfig::load().unwrap();
        assert!(config.server.auto_open_browser);
    }

    #[test]
    fn test_resolve_absolute_path() {
        let path = if cfg!(windows) {
            "C:\\Games"
        } else {
            "/home/user/games"
        };
        let resolved = resolve_path(path);
        assert!(resolved.is_absolute());
        assert_eq!(resolved.to_string_lossy(), path);
    }

    #[test]
    fn test_resolve_relative_path() {
        let resolved = resolve_path("./games");
        // Relative paths get resolved to exe directory
        assert!(resolved.is_absolute());
        assert!(resolved.to_string_lossy().contains("games"));
    }

    #[test]
    fn test_database_url_format() {
        let config = AppConfig::load().unwrap();
        let db_url = config.database_url();
        assert!(db_url.starts_with("sqlite:"));
        assert!(db_url.contains("?mode=rwc"));
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig {
            paths: PathsConfig {
                game_library: PathBuf::from("D:\\Games"),
                database: "sqlite:./data/test.db?mode=rwc".to_string(),
                cache: PathBuf::from("./cache"),
            },
            server: ServerConfig {
                port: 8080,
                auto_open_browser: false,
                bind_address: "0.0.0.0".to_string(),
            },
        };

        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("port = 8080"));
        assert!(toml_str.contains("auto_open_browser = false"));
        assert!(toml_str.contains("bind_address = \"0.0.0.0\""));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
[paths]
game_library = "D:\\TestGames"
database = "sqlite:./data/test.db?mode=rwc"
cache = "./cache"

[server]
port = 9000
auto_open_browser = true
bind_address = "127.0.0.1"
"#;

        let config: AppConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.server.port, 9000);
        assert!(config.server.auto_open_browser);
        assert_eq!(config.paths.game_library.to_string_lossy(), "D:\\TestGames");
    }

    #[test]
    fn test_empty_game_library_is_valid() {
        let toml_str = r#"
[paths]
game_library = ""
database = "sqlite:./data/test.db?mode=rwc"
cache = "./cache"

[server]
port = 3000
auto_open_browser = true
bind_address = "127.0.0.1"
"#;

        let config: Result<AppConfig, _> = toml::from_str(toml_str);
        assert!(config.is_ok());
    }
}
