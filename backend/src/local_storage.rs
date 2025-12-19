use reqwest::Client;
use chrono::Utc;
use crate::models::Game;
use std::path::{Path, PathBuf};
use std::fs;
use std::io::Write;

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
                        filename: path.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default(),
                        path: path.to_string_lossy().to_string(),
                        size_bytes: metadata.len() as i64,
                        created_at: metadata.created()
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
pub fn read_game_metadata(game_folder: &str) -> Result<ImportedMetadata, Box<dyn std::error::Error + Send + Sync>> {
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
    Path::new(game_folder).join(GAMEVAULT_DIR).join("metadata.json")
}

/// Export game metadata to JSON file in .gamevault folder
pub fn export_game_metadata(game: &Game) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let folder_path = &game.folder_path;
    
    // Check if folder is writable
    if !is_folder_writable(folder_path) {
        return Err(format!("Game folder not writable: {}", folder_path).into());
    }
    
    // Ensure .gamevault directory exists
    ensure_gamevault_dir(folder_path)?;
    
    // Parse JSON string fields into Vec<String>
    let genres: Option<Vec<String>> = game.genres.as_ref()
        .and_then(|s| serde_json::from_str(s).ok());
    let developers: Option<Vec<String>> = game.developers.as_ref()
        .and_then(|s| serde_json::from_str(s).ok());
    let publishers: Option<Vec<String>> = game.publishers.as_ref()
        .and_then(|s| serde_json::from_str(s).ok());
    
    // Build HLTB data if any field is present
    let hltb = if game.hltb_main_mins.is_some() || game.hltb_extra_mins.is_some() || game.hltb_completionist_mins.is_some() {
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
    };
    
    // Serialize to pretty JSON
    let json = serde_json::to_string_pretty(&metadata)?;
    
    // Write to file
    let metadata_path = get_metadata_path(folder_path);
    fs::write(&metadata_path, &json)?;
    
    tracing::info!("Exported metadata: {:?} ({} bytes)", metadata_path, json.len());
    Ok(metadata_path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
