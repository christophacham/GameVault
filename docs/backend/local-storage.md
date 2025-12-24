---
sidebar_position: 6
---

# local_storage.rs

Local file operations for metadata persistence and image caching.

## Directory Structure

Each game folder contains a `.gamevault` subdirectory:

```
D:\Games\Cyberpunk 2077\
├── .gamevault\
│   ├── metadata.json     # Exported game data
│   ├── cover.jpg         # Cached cover image
│   └── background.jpg    # Cached background
└── ... (game files)
```

## Metadata Export

### ExportedMetadata

```rust
#[derive(Serialize, Deserialize)]
pub struct ExportedMetadata {
    pub schema_version: i32,
    pub exported_at: String,
    pub steam_app_id: Option<i64>,
    pub title: String,
    pub summary: Option<String>,
    pub genres: Option<Vec<String>>,
    pub developers: Option<Vec<String>>,
    pub publishers: Option<Vec<String>>,
    pub release_date: Option<String>,
    pub review_score: Option<i64>,
    pub review_count: Option<i64>,
    pub review_summary: Option<String>,
    pub hltb: Option<HltbData>,
    pub manually_edited: bool,
}
```

### save_game_metadata

Dual-write function called after game updates:

```rust
pub fn save_game_metadata(game: &Game) -> Result<()> {
    let gamevault_dir = get_gamevault_dir(&game.folder_path);
    std::fs::create_dir_all(&gamevault_dir)?;

    let metadata = ExportedMetadata {
        schema_version: 1,
        exported_at: chrono::Utc::now().to_rfc3339(),
        steam_app_id: game.steam_app_id,
        title: game.title.clone(),
        summary: game.summary.clone(),
        genres: parse_json_array(&game.genres),
        developers: parse_json_array(&game.developers),
        publishers: parse_json_array(&game.publishers),
        release_date: game.release_date.clone(),
        review_score: game.review_score,
        review_count: game.review_count,
        review_summary: game.review_summary.clone(),
        hltb: game.hltb_main_mins.map(|main| HltbData {
            main_mins: Some(main),
            extra_mins: game.hltb_extra_mins,
            completionist_mins: game.hltb_completionist_mins,
        }),
        manually_edited: game.manually_edited.unwrap_or(0) == 1,
    };

    // Atomic write: write to temp file, then rename
    let metadata_path = gamevault_dir.join("metadata.json");
    let tmp_path = metadata_path.with_extension("json.tmp");

    let json = serde_json::to_string_pretty(&metadata)?;
    std::fs::write(&tmp_path, json)?;
    std::fs::rename(tmp_path, metadata_path)?;

    Ok(())
}
```

### export_game_metadata

Called during bulk export:

```rust
pub fn export_game_metadata(game: &Game) -> Result<()> {
    // Only export games with Steam data
    if game.steam_app_id.is_none() {
        return Ok(());
    }

    save_game_metadata(game)
}
```

## Metadata Import

### ImportResult

```rust
pub enum ImportResult {
    Imported(ExportedMetadata),
    Skipped { reason: String },
    NotFound,
    Failed { error: String },
}
```

### import_game_metadata

```rust
pub fn import_game_metadata(game: &Game) -> ImportResult {
    let metadata_path = get_metadata_path(&game.folder_path);

    if !metadata_path.exists() {
        return ImportResult::NotFound;
    }

    let content = match std::fs::read_to_string(&metadata_path) {
        Ok(c) => c,
        Err(e) => return ImportResult::Failed { error: e.to_string() },
    };

    let metadata: ExportedMetadata = match serde_json::from_str(&content) {
        Ok(m) => m,
        Err(e) => return ImportResult::Failed { error: e.to_string() },
    };

    // Skip if game already has Steam data and wasn't manually edited
    if game.steam_app_id.is_some() && !metadata.manually_edited {
        return ImportResult::Skipped {
            reason: "Already enriched".to_string()
        };
    }

    ImportResult::Imported(metadata)
}
```

## Image Caching

### cache_game_images

Downloads and caches cover and background images:

```rust
pub async fn cache_game_images(
    client: &reqwest::Client,
    folder_path: &str,
    cover_url: Option<&str>,
    background_url: Option<&str>,
) -> (Option<String>, Option<String>) {
    let gamevault_dir = get_gamevault_dir(folder_path);

    // Create directory if needed
    if std::fs::create_dir_all(&gamevault_dir).is_err() {
        return (None, None);
    }

    let local_cover = if let Some(url) = cover_url {
        download_image(client, url, &gamevault_dir.join("cover.jpg")).await
    } else {
        None
    };

    let local_bg = if let Some(url) = background_url {
        download_image(client, url, &gamevault_dir.join("background.jpg")).await
    } else {
        None
    };

    (local_cover, local_bg)
}
```

### download_image

```rust
async fn download_image(
    client: &reqwest::Client,
    url: &str,
    path: &Path,
) -> Option<String> {
    let response = client.get(url).send().await.ok()?;
    let bytes = response.bytes().await.ok()?;

    std::fs::write(path, &bytes).ok()?;

    Some(path.to_string_lossy().to_string())
}
```

### Path Helpers

```rust
pub fn get_gamevault_dir(folder_path: &str) -> PathBuf {
    Path::new(folder_path).join(".gamevault")
}

pub fn get_metadata_path(folder_path: &str) -> PathBuf {
    get_gamevault_dir(folder_path).join("metadata.json")
}

pub fn get_cover_path(folder_path: &str) -> PathBuf {
    get_gamevault_dir(folder_path).join("cover.jpg")
}

pub fn get_background_path(folder_path: &str) -> PathBuf {
    get_gamevault_dir(folder_path).join("background.jpg")
}
```

## Folder Utilities

### is_folder_writable

```rust
pub fn is_folder_writable(folder_path: &str) -> bool {
    let test_path = Path::new(folder_path).join(".gamevault_write_test");

    // Try to create a test file
    let result = std::fs::write(&test_path, "test");

    // Clean up
    let _ = std::fs::remove_file(&test_path);

    result.is_ok()
}
```

### list_backups

```rust
pub fn list_backups(folder_path: &str) -> Vec<BackupInfo> {
    let gamevault_dir = get_gamevault_dir(folder_path);
    let backups_dir = gamevault_dir.join("backups");

    if !backups_dir.exists() {
        return vec![];
    }

    std::fs::read_dir(&backups_dir)
        .ok()
        .map(|entries| {
            entries
                .flatten()
                .filter_map(|e| {
                    let path = e.path();
                    if path.extension()?.to_str()? == "zip" {
                        Some(BackupInfo {
                            filename: e.file_name().to_string_lossy().to_string(),
                            created_at: e.metadata().ok()?.modified().ok()?,
                        })
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}
```

## Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exported_metadata_serialization() {
        let metadata = ExportedMetadata {
            schema_version: 1,
            exported_at: "2024-01-01T00:00:00Z".to_string(),
            steam_app_id: Some(292030),
            title: "The Witcher 3".to_string(),
            // ...
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let parsed: ExportedMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.title, metadata.title);
    }

    #[test]
    fn test_manually_edited_flag() {
        let game = create_test_game();
        let metadata = ExportedMetadata::from_game(&game);

        assert_eq!(metadata.manually_edited, game.manually_edited == Some(1));
    }
}
```
