use std::path::Path;

use regex::Regex;
use walkdir::WalkDir;

/// Patterns to remove from folder names to get clean game titles
const CLEANUP_PATTERNS: &[&str] = &[
    r"\[FitGirl.*?\]",
    r"\[DODI.*?\]",
    r"\[.*?Repack.*?\]",
    r"\[.*?Monkey.*?\]",
    r"\[BluRay\]",
    r"\[720p\]",
    r"\[1080p\]",
    r"\[YTS.*?\]",
    r"\[YIFY\]",
    r"Portable\s+by\s+\w+",
    r"by\s+\w+$",
    r"\s*v\d+(\.\d+)*\w*",
    r"\s*-\s*(HRTP|EE|NG|MCE|CGC)$",
    r"\s*NG\s*-\s*HRTP$",
    r"\s*-\s*Dilogy$",
    r"\s*\(.*?\)",
    r"\s+$",
    r"^\s+",
];

/// Patterns that indicate non-game content (movies, etc.)
const EXCLUSION_PATTERNS: &[&str] = &[
    r"(?i)\[BluRay\]",
    r"(?i)\[720p\]",
    r"(?i)\[1080p\]",
    r"(?i)\[2160p\]",
    r"(?i)\[4K\]",
    r"(?i)\[YTS",
    r"(?i)\[YIFY",
    r"(?i)\[RARBG\]",
    r"(?i)\[WEB-?DL\]",
    r"(?i)\[HDRip\]",
    r"(?i)\[BRRip\]",
    r"(?i)\[DVDRip\]",
    r"(?i)\.mkv$",
    r"(?i)\.avi$",
    r"(?i)\.mp4$",
    r"(?i)S\d{2}E\d{2}",  // TV show pattern like S01E05
];

/// Check if a folder name matches exclusion patterns (non-game content)
fn is_excluded(folder_name: &str) -> bool {
    for pattern in EXCLUSION_PATTERNS {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(folder_name) {
                return true;
            }
        }
    }
    false
}

pub struct ScannedGame {
    pub folder_path: String,
    pub folder_name: String,
    pub clean_title: String,
    pub size_bytes: Option<i64>,
}

/// Clean a folder name to extract the game title
pub fn clean_title(folder_name: &str) -> String {
    let mut title = folder_name.to_string();

    for pattern in CLEANUP_PATTERNS {
        if let Ok(re) = Regex::new(pattern) {
            title = re.replace_all(&title, "").to_string();
        }
    }

    // Clean up multiple spaces and dashes
    let re_spaces = Regex::new(r"\s+").unwrap();
    title = re_spaces.replace_all(&title, " ").to_string();

    let re_dashes = Regex::new(r"\s*-\s*$").unwrap();
    title = re_dashes.replace_all(&title, "").to_string();

    title.trim().to_string()
}

/// Scan a directory for game folders
pub fn scan_games_directory(path: &str) -> Vec<ScannedGame> {
    let mut games = Vec::new();

    let base_path = Path::new(path);
    if !base_path.exists() {
        tracing::error!("Games path does not exist: {}", path);
        return games;
    }

    for entry in WalkDir::new(path).min_depth(1).max_depth(1) {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!("Error reading directory entry: {}", e);
                continue;
            }
        };

        if !entry.file_type().is_dir() {
            continue;
        }

        let folder_name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden folders and known non-game folders
        if folder_name.starts_with('.')
            || folder_name == "game-library-app"
            || folder_name == "GameVault"
            || folder_name == "Adult"
            || folder_name.ends_with(".rar")
            || folder_name.ends_with(".zip")
        {
            continue;
        }

        // Skip non-game content (movies, TV shows, etc.) - check raw name before cleanup
        if is_excluded(&folder_name) {
            tracing::info!("Excluding non-game content: {}", folder_name);
            continue;
        }

        let folder_path = entry.path().to_string_lossy().to_string();
        let clean_title = clean_title(&folder_name);

        // Try to get folder size (just count immediate contents for speed)
        let size_bytes = get_folder_size_estimate(entry.path());

        if !clean_title.is_empty() {
            games.push(ScannedGame {
                folder_path,
                folder_name,
                clean_title,
                size_bytes,
            });
        }
    }

    tracing::info!("Scanned {} game folders", games.len());
    games
}

/// Get an estimate of folder size (for performance, only counts top-level files)
fn get_folder_size_estimate(path: &Path) -> Option<i64> {
    let mut total: u64 = 0;

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    total += metadata.len();
                }
            }
        }
    }

    if total > 0 {
        Some(total as i64)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_title() {
        assert_eq!(
            clean_title("Cyberpunk 2077 [FitGirl Repack]"),
            "Cyberpunk 2077"
        );
        // Note: HRTP suffix not stripped as it may be a valid game variant identifier
        assert_eq!(
            clean_title("Fallout 4 NG - HRTP [FitGirl Repack]"),
            "Fallout 4 NG - HRTP"
        );
        assert_eq!(
            clean_title("STALKER 2 Heart of Chornobyl - Ultimate Edition Portable by Ksenia"),
            "STALKER 2 Heart of Chornobyl - Ultimate Edition"
        );
        assert_eq!(
            clean_title("Age of Empires II - Definitive Edition [FitGirl Repack]"),
            "Age of Empires II - Definitive Edition"
        );
        assert_eq!(
            clean_title("C&C - Remastered Collection [FitGirl Repack]"),
            "C&C - Remastered Collection"
        );
    }
}
