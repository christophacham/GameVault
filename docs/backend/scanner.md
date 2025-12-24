---
sidebar_position: 4
---

# scanner.rs

Directory scanning with regex-based folder name cleanup.

## Core Functions

### scan_games_directory

Scans a directory for game folders:

```rust
pub fn scan_games_directory(path: &str) -> Vec<ScannedGame> {
    let mut games = Vec::new();

    for entry in WalkDir::new(path).min_depth(1).max_depth(1) {
        let entry = entry?;

        if !entry.file_type().is_dir() {
            continue;
        }

        let folder_name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden and known non-game folders
        if should_skip(&folder_name) {
            continue;
        }

        // Skip non-game content (movies, TV shows)
        if is_excluded(&folder_name) {
            tracing::info!("Excluding: {}", folder_name);
            continue;
        }

        let clean_title = clean_title(&folder_name);
        let size_bytes = get_folder_size_estimate(entry.path());

        if !clean_title.is_empty() {
            games.push(ScannedGame {
                folder_path: entry.path().to_string_lossy().to_string(),
                folder_name,
                clean_title,
                size_bytes,
            });
        }
    }

    games
}
```

### ScannedGame

```rust
pub struct ScannedGame {
    pub folder_path: String,
    pub folder_name: String,
    pub clean_title: String,
    pub size_bytes: Option<i64>,
}
```

## Title Cleanup

### Cleanup Patterns

Removed from folder names to extract clean titles:

```rust
const CLEANUP_PATTERNS: &[&str] = &[
    r"\[FitGirl.*?\]",           // [FitGirl Repack]
    r"\[DODI.*?\]",              // [DODI Repack]
    r"\[.*?Repack.*?\]",         // Any repack tag
    r"\[.*?Monkey.*?\]",         // [Tiny Monkey Repack]
    r"\[BluRay\]",               // Video tags
    r"\[720p\]",
    r"\[1080p\]",
    r"\[YTS.*?\]",
    r"\[YIFY\]",
    r"Portable\s+by\s+\w+",      // Portable by Ksenia
    r"by\s+\w+$",                // by Author
    r"\s*v\d+(\.\d+)*\w*",       // Version numbers
    r"\s*-\s*(HRTP|EE|NG|MCE|CGC)$",  // Edition suffixes
    r"\s*NG\s*-\s*HRTP$",
    r"\s*-\s*Dilogy$",
    r"\s*\(.*?\)",               // Parenthetical info
    r"\s+$",                      // Trailing whitespace
    r"^\s+",                      // Leading whitespace
];
```

### clean_title Function

```rust
pub fn clean_title(folder_name: &str) -> String {
    let mut title = folder_name.to_string();

    // Apply all cleanup patterns
    for pattern in CLEANUP_PATTERNS {
        if let Ok(re) = Regex::new(pattern) {
            title = re.replace_all(&title, "").to_string();
        }
    }

    // Clean up multiple spaces
    let re_spaces = Regex::new(r"\s+").unwrap();
    title = re_spaces.replace_all(&title, " ").to_string();

    // Remove trailing dashes
    let re_dashes = Regex::new(r"\s*-\s*$").unwrap();
    title = re_dashes.replace_all(&title, "").to_string();

    title.trim().to_string()
}
```

### Examples

| Folder Name | Clean Title |
|-------------|-------------|
| `Cyberpunk 2077 [FitGirl Repack]` | `Cyberpunk 2077` |
| `STALKER 2 Portable by Ksenia` | `STALKER 2` |
| `Age of Empires II v1.0.5` | `Age of Empires II` |
| `Fallout 4 NG - HRTP [FitGirl]` | `Fallout 4 NG - HRTP` |

## Exclusion Patterns

### Non-Game Content

Patterns to skip movies, TV shows, etc.:

```rust
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
    r"(?i)S\d{2}E\d{2}",  // TV show pattern (S01E05)
];
```

### is_excluded Function

```rust
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
```

## Folder Filtering

### Skip Conditions

```rust
fn should_skip(folder_name: &str) -> bool {
    folder_name.starts_with('.')           // Hidden folders
        || folder_name == "game-library-app" // Our app folder
        || folder_name == "GameVault"       // Our app folder
        || folder_name == "Adult"           // Adult content
        || folder_name.ends_with(".rar")    // Archive files
        || folder_name.ends_with(".zip")
}
```

## Size Estimation

### Quick Size Calculation

For performance, only counts immediate files:

```rust
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
```

## Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_title() {
        assert_eq!(
            clean_title("Cyberpunk 2077 [FitGirl Repack]"),
            "Cyberpunk 2077"
        );
        assert_eq!(
            clean_title("STALKER 2 Portable by Ksenia"),
            "STALKER 2"
        );
        assert_eq!(
            clean_title("Age of Empires II v1.0.5"),
            "Age of Empires II"
        );
    }

    #[test]
    fn test_exclusion() {
        assert!(is_excluded("Movie [1080p] [BluRay]"));
        assert!(is_excluded("Show S01E05"));
        assert!(!is_excluded("Elden Ring"));
    }
}
```
