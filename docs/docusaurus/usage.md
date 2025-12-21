---
sidebar_position: 4
---

# Usage Guide

Learn how to use GameVault's features to manage your game library.

## Scanning Games

### How Scanning Works

When you click **Scan**, GameVault:

1. Traverses your `game_library` folder recursively
2. Identifies potential game folders
3. Extracts clean titles from folder names
4. Adds or updates entries in the database

### Folder Name Parsing

GameVault intelligently parses folder names:

| Folder Name | Detected Title |
|-------------|----------------|
| `The Witcher 3 Wild Hunt` | The Witcher 3 Wild Hunt |
| `Cyberpunk.2077-GOG` | Cyberpunk 2077 |
| `Dark Souls III [FitGirl]` | Dark Souls III |
| `ELDEN.RING.v1.10-RUNE` | Elden Ring |

### Re-scanning

Re-scanning is safe and incremental:
- New games are added
- Existing games are updated (size changes)
- Removed games are kept (manual cleanup available)

## Steam Enrichment

### How Enrichment Works

The **Enrich** feature fetches metadata from Steam:

1. **Matching**: Fuzzy title matching against Steam's database
2. **Fetching**: Retrieves game details from Steam API
3. **Caching**: Downloads and caches cover images locally
4. **Storing**: Saves metadata to database and `.gamevault/` folder

### Enriched Data

| Field | Description |
|-------|-------------|
| Cover Image | Game header image (460x215) |
| Background | Library hero image |
| Description | Short game summary |
| Genres | Game genres and tags |
| Developer | Development studio |
| Publisher | Publishing company |
| Release Date | Original release date |
| Review Score | Steam review percentage |
| Review Count | Number of reviews |

### Match Confidence

Each game has a match confidence score:
- **High (>0.8)**: Strong title match, likely correct
- **Medium (0.5-0.8)**: Reasonable match, may need verification
- **Low (<0.5)**: Uncertain match, consider manual adjustment

### Adjusting Matches

If a game is matched incorrectly:

1. Click the game card's menu (three dots)
2. Select **Adjust Match**
3. Enter the correct Steam URL or App ID
4. Preview the new match
5. Confirm to apply

## Editing Game Details

### Manual Editing

Edit any game's metadata manually:

1. Click the game card's menu
2. Select **Edit**
3. Modify fields:
   - Title
   - Summary
   - Genres (comma-separated)
   - Developers
   - Publishers
   - Release Date
   - Review Score (0-100)
4. Click **Save Changes**

### Dual-Write System

Edits are saved in two locations:
- **Database**: Primary storage for quick access
- **Metadata File**: `.gamevault/metadata.json` in game folder

This ensures your edits are preserved even if you reset the database.

## Search and Filtering

### Search

Use the search bar to find games:
- Searches by title
- Real-time results
- Clear to show all games

### Game Grid

Games are displayed in a responsive grid:
- Cover images with hover effects
- Review score indicators
- Match status badges
- Size and playtime info (coming soon)

## Import and Export

### Exporting Metadata

Export all metadata to local folders:

1. Click **Enrich** button
2. Select **Export Metadata**
3. Creates `.gamevault/metadata.json` in each game folder

Useful for:
- Backing up metadata
- Sharing with other tools
- Preserving before database reset

### Importing Metadata

Import metadata from local folders:

1. Click **Enrich** button
2. Select **Import Metadata**
3. Reads `.gamevault/metadata.json` from each game folder

Useful for:
- Restoring after database reset
- Migrating from another installation
- Bulk importing manual edits

## Settings Management

### Accessing Settings

Click the **gear icon** in the header to open Settings.

### Available Settings

- **Game Library Path**: Your games folder location
- **Cache Directory**: Where cover images are stored
- **Server Port**: HTTP server port (requires restart)
- **Auto-open Browser**: Launch browser on startup

### Restart and Shutdown

From the Settings modal:
- **Restart**: Restart GameVault (applies port changes)
- **Shutdown**: Close GameVault completely

### System Tray

Right-click the system tray icon:
- **Open GameVault**: Open web interface
- **Quit**: Close application
