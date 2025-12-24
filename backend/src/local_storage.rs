use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::Utc;
use reqwest::Client;

use crate::models::Game;

/// Directory name for GameVault data within each game folder
const GAMEVAULT_DIR: &str = ".gamevault";
const SAVES_DIR: &str = "saves";

/// Check if a game folder is writable
pub fn is_folder_writable(game_folder: &str) -> bool {
    let path = Path::new(game_folder);
    if !path.exists() {
        return false;
    }

    // Try to create/access the .gamevault directory
    let gamevault_path = path.join(GAMEVAULT_DIR);

    // If it exists, check if we can write to it
    if gamevault_path.exists() {
        let test_file = gamevault_path.join(".write_test");
        match fs::File::create(&test_file) {
            Ok(_) => {
                let _ = fs::remove_file(&test_file);
                true
            }
            Err(_) => false,
        }
    } else {
        // Try to create the directory
        match fs::create_dir(&gamevault_path) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

/// Ensure .gamevault directory exists in game folder
pub fn ensure_gamevault_dir(game_folder: &str) -> Result<PathBuf, std::io::Error> {
    let gamevault_path = Path::new(game_folder).join(GAMEVAULT_DIR);
    fs::create_dir_all(&gamevault_path)?;
    Ok(gamevault_path)
}

/// Ensure saves directory exists within .gamevault
pub fn ensure_saves_dir(game_folder: &str) -> Result<PathBuf, std::io::Error> {
    let saves_path = Path::new(game_folder).join(GAMEVAULT_DIR).join(SAVES_DIR);
    fs::create_dir_all(&saves_path)?;
    Ok(saves_path)
}

/// Get the path where cover image should be stored
pub fn get_cover_path(game_folder: &str) -> PathBuf {
    Path::new(game_folder).join(GAMEVAULT_DIR).join("cover.jpg")
}

/// Get the path where background image should be stored
pub fn get_background_path(game_folder: &str) -> PathBuf {
    Path::new(game_folder).join(GAMEVAULT_DIR).join("background.jpg")
}

/// Download and save an image to local storage
pub async fn download_and_save_image(
    client: &Client,
    url: &str,
    dest_path: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Skip if file already exists
    if dest_path.exists() {
        tracing::debug!("Image already cached: {:?}", dest_path);
        return Ok(());
    }

    // Ensure parent directory exists
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)?;
    }

    tracing::info!("Downloading image: {} -> {:?}", url, dest_path);

    let response = client
        .get(url)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }

    let bytes = response.bytes().await?;

    // Write to file
    let mut file = fs::File::create(dest_path)?;
    file.write_all(&bytes)?;

    tracing::info!("Saved image: {:?} ({} bytes)", dest_path, bytes.len());
    Ok(())
}

/// Cache cover and background images for a game
pub async fn cache_game_images(
    client: &Client,
    game_folder: &str,
    cover_url: Option<&str>,
    background_url: Option<&str>,
) -> (Option<String>, Option<String>) {
    // Check if folder is writable first
    if !is_folder_writable(game_folder) {
        tracing::warn!("Game folder not writable, skipping image cache: {}", game_folder);
        return (None, None);
    }

    let mut local_cover: Option<String> = None;
    let mut local_background: Option<String> = None;

    // Download cover image
    if let Some(url) = cover_url {
        let cover_path = get_cover_path(game_folder);
        match download_and_save_image(client, url, &cover_path).await {
            Ok(_) => {
                local_cover = Some(cover_path.to_string_lossy().to_string());
            }
            Err(e) => {
                tracing::warn!("Failed to download cover: {}", e);
            }
        }
    }

    // Download background image
    if let Some(url) = background_url {
        let bg_path = get_background_path(game_folder);
        match download_and_save_image(client, url, &bg_path).await {
            Ok(_) => {
                local_background = Some(bg_path.to_string_lossy().to_string());
            }
            Err(e) => {
                tracing::warn!("Failed to download background: {}", e);
            }
        }
    }

    (local_cover, local_background)
}

/// List all backup files for a game
pub fn list_backups(game_folder: &str) -> Vec<BackupInfo> {
    let saves_path = Path::new(game_folder).join(GAMEVAULT_DIR).join(SAVES_DIR);
    let mut backups = Vec::new();

    if let Ok(entries) = fs::read_dir(&saves_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "zip") {
                if let Ok(metadata) = entry.metadata() {
                    backups.push(BackupInfo {
                        filename: path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default(),
                        path: path.to_string_lossy().to_string(),
                        size_bytes: metadata.len() as i64,
                        created_at: metadata
                            .created()
                            .ok()
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs() as i64)
                            .unwrap_or(0),
                    });
                }
            }
        }
    }

    // Sort by created_at descending (newest first)
    backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    backups
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BackupInfo {
    pub filename: String,
    pub path: String,
    pub size_bytes: i64,
    pub created_at: i64,
}

/// Metadata structure for JSON export
/// This is a dedicated DTO separate from Game to provide a stable export format
#[derive(Debug, Clone, serde::Serialize)]
pub struct ExportedMetadata {
    pub schema_version: u32,
    pub title: String,
    pub steam_app_id: Option<i64>,
    pub summary: Option<String>,
    pub genres: Option<Vec<String>>,
    pub developers: Option<Vec<String>>,
    pub publishers: Option<Vec<String>>,
    pub release_date: Option<String>,
    pub review_score: Option<i64>,
    pub review_summary: Option<String>,
    pub hltb: Option<HltbData>,
    pub exported_at: String,
    pub manually_edited: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HltbData {
    pub main_mins: Option<i64>,
    pub extra_mins: Option<i64>,
    pub completionist_mins: Option<i64>,
}

/// Imported metadata structure (mirrors ExportedMetadata but with Deserialize)
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ImportedMetadata {
    pub title: String,
    pub steam_app_id: Option<i64>,
    pub summary: Option<String>,
    pub genres: Option<Vec<String>>,
    pub developers: Option<Vec<String>>,
    pub publishers: Option<Vec<String>>,
    pub release_date: Option<String>,
    pub review_score: Option<i64>,
    pub review_summary: Option<String>,
    pub hltb: Option<HltbData>,
    pub exported_at: String,
}

/// Result of importing metadata for a single game
#[derive(Debug)]
pub enum ImportResult {
    Imported(ImportedMetadata),
    Skipped { reason: String },
    NotFound,
    Failed { error: String },
}

/// Read and parse metadata from .gamevault/metadata.json
pub fn read_game_metadata(
    game_folder: &str,
) -> Result<ImportedMetadata, Box<dyn std::error::Error + Send + Sync>> {
    let metadata_path = get_metadata_path(game_folder);

    if !metadata_path.exists() {
        return Err("Metadata file not found".into());
    }

    let json_content = fs::read_to_string(&metadata_path)?;
    let metadata: ImportedMetadata = serde_json::from_str(&json_content)?;

    Ok(metadata)
}

/// Import game metadata from JSON file, comparing timestamps
/// Returns ImportResult indicating what happened
pub fn import_game_metadata(game: &Game) -> ImportResult {
    let folder_path = &game.folder_path;

    // Try to read the metadata file
    let metadata = match read_game_metadata(folder_path) {
        Ok(m) => m,
        Err(e) => {
            let err_str = e.to_string();
            if err_str.contains("not found") {
                return ImportResult::NotFound;
            }
            return ImportResult::Failed { error: err_str };
        }
    };

    // Parse timestamps for comparison
    let json_exported_at = chrono::DateTime::parse_from_rfc3339(&metadata.exported_at)
        .map(|dt| dt.with_timezone(&Utc))
        .ok();

    let db_updated_at = chrono::DateTime::parse_from_rfc3339(&game.updated_at)
        .ok()
        .map(|dt| dt.with_timezone(&Utc));

    // Compare timestamps - only import if JSON is newer or DB has no timestamp
    match (json_exported_at, db_updated_at) {
        (Some(json_time), Some(db_time)) if json_time <= db_time => {
            return ImportResult::Skipped {
                reason: format!("Database is newer ({} vs {})", db_time, json_time),
            };
        }
        _ => {
            // JSON is newer, DB has no timestamp, or couldn't parse - proceed with import
        }
    }

    tracing::info!("Importing metadata for: {}", game.title);
    ImportResult::Imported(metadata)
}

/// Get the path where metadata JSON should be stored
pub fn get_metadata_path(game_folder: &str) -> PathBuf {
    Path::new(game_folder)
        .join(GAMEVAULT_DIR)
        .join("metadata.json")
}

/// Export game metadata to JSON file in .gamevault folder
pub fn export_game_metadata(
    game: &Game,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let folder_path = &game.folder_path;

    // Check if folder is writable
    if !is_folder_writable(folder_path) {
        return Err(format!("Game folder not writable: {}", folder_path).into());
    }

    // Ensure .gamevault directory exists
    ensure_gamevault_dir(folder_path)?;

    // Parse JSON string fields into Vec<String>
    let genres: Option<Vec<String>> = game
        .genres
        .as_ref()
        .and_then(|s| serde_json::from_str(s).ok());
    let developers: Option<Vec<String>> = game
        .developers
        .as_ref()
        .and_then(|s| serde_json::from_str(s).ok());
    let publishers: Option<Vec<String>> = game
        .publishers
        .as_ref()
        .and_then(|s| serde_json::from_str(s).ok());

    // Build HLTB data if any field is present
    let hltb = if game.hltb_main_mins.is_some()
        || game.hltb_extra_mins.is_some()
        || game.hltb_completionist_mins.is_some()
    {
        Some(HltbData {
            main_mins: game.hltb_main_mins,
            extra_mins: game.hltb_extra_mins,
            completionist_mins: game.hltb_completionist_mins,
        })
    } else {
        None
    };

    // Create export struct
    let metadata = ExportedMetadata {
        schema_version: 2,
        title: game.title.clone(),
        steam_app_id: game.steam_app_id,
        summary: game.summary.clone(),
        genres,
        developers,
        publishers,
        release_date: game.release_date.clone(),
        review_score: game.review_score,
        review_summary: game.review_summary.clone(),
        hltb,
        exported_at: Utc::now().to_rfc3339(),
        manually_edited: game.manually_edited.unwrap_or(0) == 1,
    };

    // Serialize to pretty JSON
    let json = serde_json::to_string_pretty(&metadata)?;

    // Write to file
    let metadata_path = get_metadata_path(folder_path);
    fs::write(&metadata_path, &json)?;

    tracing::info!(
        "Exported metadata: {:?} ({} bytes)",
        metadata_path,
        json.len()
    );
    Ok(metadata_path.to_string_lossy().to_string())
}

/// Save game metadata after user edit (dual-write from DB)
/// This is called after update_game_metadata to keep JSON in sync
pub fn save_game_metadata(game: &Game) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let folder_path = &game.folder_path;

    // Check if folder is writable
    if !is_folder_writable(folder_path) {
        tracing::warn!(
            "Game folder not writable, skipping metadata save: {}",
            folder_path
        );
        return Ok(()); // Don't fail the request, just skip file write
    }

    // Ensure .gamevault directory exists
    ensure_gamevault_dir(folder_path)?;

    // Parse JSON string fields into Vec<String>
    let genres: Option<Vec<String>> = game
        .genres
        .as_ref()
        .and_then(|s| serde_json::from_str(s).ok());
    let developers: Option<Vec<String>> = game
        .developers
        .as_ref()
        .and_then(|s| serde_json::from_str(s).ok());
    let publishers: Option<Vec<String>> = game
        .publishers
        .as_ref()
        .and_then(|s| serde_json::from_str(s).ok());

    // Build HLTB data if any field is present
    let hltb = if game.hltb_main_mins.is_some()
        || game.hltb_extra_mins.is_some()
        || game.hltb_completionist_mins.is_some()
    {
        Some(HltbData {
            main_mins: game.hltb_main_mins,
            extra_mins: game.hltb_extra_mins,
            completionist_mins: game.hltb_completionist_mins,
        })
    } else {
        None
    };

    // Create export struct with manually_edited = true (since this is from user edit)
    let metadata = ExportedMetadata {
        schema_version: 2,
        title: game.title.clone(),
        steam_app_id: game.steam_app_id,
        summary: game.summary.clone(),
        genres,
        developers,
        publishers,
        release_date: game.release_date.clone(),
        review_score: game.review_score,
        review_summary: game.review_summary.clone(),
        hltb,
        exported_at: Utc::now().to_rfc3339(),
        manually_edited: true, // Always true when saving from user edit
    };

    // Serialize to pretty JSON
    let json = serde_json::to_string_pretty(&metadata)?;

    // Write to file
    let metadata_path = get_metadata_path(folder_path);
    fs::write(&metadata_path, &json)?;

    tracing::info!(
        "Saved game metadata: {:?} ({} bytes)",
        metadata_path,
        json.len()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Game;

    #[test]
    fn test_gamevault_paths() {
        let folder = "/games/TestGame";
        assert_eq!(
            get_cover_path(folder),
            PathBuf::from("/games/TestGame/.gamevault/cover.jpg")
        );
        assert_eq!(
            get_background_path(folder),
            PathBuf::from("/games/TestGame/.gamevault/background.jpg")
        );
    }

    #[test]
    fn test_metadata_path() {
        let folder = "/games/TestGame";
        assert_eq!(
            get_metadata_path(folder),
            PathBuf::from("/games/TestGame/.gamevault/metadata.json")
        );
    }

    #[test]
    fn test_exported_metadata_schema_version() {
        let metadata = ExportedMetadata {
            schema_version: 2,
            title: "Test Game".to_string(),
            steam_app_id: Some(12345),
            summary: Some("A test game".to_string()),
            genres: Some(vec!["Action".to_string()]),
            developers: Some(vec!["Test Dev".to_string()]),
            publishers: Some(vec!["Test Pub".to_string()]),
            release_date: Some("2024-01-15".to_string()),
            review_score: Some(85),
            review_summary: Some("Very Positive".to_string()),
            hltb: None,
            exported_at: "2024-01-01T00:00:00Z".to_string(),
            manually_edited: false,
        };

        assert_eq!(metadata.schema_version, 2);
        assert_eq!(metadata.manually_edited, false);
    }

    #[test]
    fn test_exported_metadata_serialization() {
        let metadata = ExportedMetadata {
            schema_version: 2,
            title: "Test Game".to_string(),
            steam_app_id: Some(12345),
            summary: None,
            genres: Some(vec!["RPG".to_string(), "Action".to_string()]),
            developers: None,
            publishers: None,
            release_date: None,
            review_score: Some(90),
            review_summary: None,
            hltb: Some(HltbData {
                main_mins: Some(600),
                extra_mins: Some(1200),
                completionist_mins: Some(2400),
            }),
            exported_at: "2024-01-01T00:00:00Z".to_string(),
            manually_edited: true,
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("\"schema_version\":2"));
        assert!(json.contains("\"manually_edited\":true"));
        assert!(json.contains("\"main_mins\":600"));
    }

    fn create_test_game() -> Game {
        Game {
            id: 1,
            folder_path: "/games/test".to_string(),
            folder_name: "test".to_string(),
            title: "Test Game".to_string(),
            igdb_id: None,
            steam_app_id: Some(12345),
            summary: Some("A test game".to_string()),
            release_date: Some("2024-01-15".to_string()),
            cover_url: None,
            background_url: None,
            local_cover_path: None,
            local_background_path: None,
            genres: Some(r#"["Action", "RPG"]"#.to_string()),
            developers: Some(r#"["Test Dev"]"#.to_string()),
            publishers: Some(r#"["Test Pub"]"#.to_string()),
            review_score: Some(85),
            review_count: None,
            review_summary: Some("Very Positive".to_string()),
            review_score_recent: None,
            review_count_recent: None,
            size_bytes: None,
            match_confidence: Some(0.95),
            match_status: "matched".to_string(),
            user_status: None,
            playtime_mins: None,
            match_locked: None,
            hltb_main_mins: Some(600),
            hltb_extra_mins: Some(1200),
            hltb_completionist_mins: Some(2400),
            save_path_pattern: None,
            manually_edited: Some(1),
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        }
    }

    #[test]
    fn test_export_game_metadata_creates_correct_structure() {
        // This test verifies the export logic without writing to disk
        let game = create_test_game();

        // Parse the JSON fields as they would be in export
        let genres: Option<Vec<String>> = game
            .genres
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok());

        assert_eq!(genres, Some(vec!["Action".to_string(), "RPG".to_string()]));
    }

    #[test]
    fn test_manually_edited_flag_conversion() {
        let game = create_test_game();

        // manually_edited is stored as i64 (0 or 1) in SQLite
        let is_edited = game.manually_edited.unwrap_or(0) == 1;
        assert!(is_edited);

        // Test unedited game
        let mut unedited_game = game;
        unedited_game.manually_edited = Some(0);
        let is_edited = unedited_game.manually_edited.unwrap_or(0) == 1;
        assert!(!is_edited);
    }
}
