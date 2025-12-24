---
sidebar_position: 2
---

# Getting Started

Get GameVault up and running in minutes.

## Prerequisites

- **Windows 10/11** (64-bit)
- A folder containing your games
- Web browser (Chrome, Firefox, Edge)

## Download

Download the latest `GameVault.exe` from the releases page and place it in any folder.

```
GameVault/
├── GameVault.exe    ← The portable executable
├── config.toml      ← Configuration (created on first run)
├── data/            ← Database (created automatically)
├── cache/           ← Cover images (created automatically)
└── logs/            ← Log files (created automatically)
```

## First Run

1. **Double-click `GameVault.exe`** to start the application
2. Your default browser will open to `http://localhost:3000`
3. Click the **Settings** (gear icon) to configure your game library path
4. Enter your games folder path (e.g., `D:\Games`)
5. Click **Save Settings**
6. Click **Scan** to discover your games

## Quick Start Guide

### Step 1: Configure Game Library

Open Settings and set your game library path:

```toml
# Example: Your games are in D:\Games
game_library = "D:\\Games"
```

GameVault will scan this folder and all subfolders for games.

### Step 2: Scan for Games

Click the **Scan** button in the header. GameVault will:

- Traverse your game library folder
- Detect game folders by name
- Add entries to the database
- Show a summary of found games

### Step 3: Enrich with Metadata

Click **Enrich** to fetch metadata from Steam:

- Cover images
- Game descriptions
- Genres and tags
- Developer/publisher info
- Review scores
- Release dates

### Step 4: Browse Your Library

Your games are now organized with:

- Visual cover art grid
- Search by title
- Review score indicators
- Game size information

## System Tray

GameVault runs in your system tray:

- **Right-click** the tray icon for options
- **Open GameVault** - Opens the web interface
- **Quit** - Closes the application

## Portable Usage

GameVault is fully portable:

- Copy the entire folder to a USB drive
- Run from any Windows computer
- All data stays in the same folder
- No registry entries or system modifications

## Troubleshooting First Run

### Browser doesn't open automatically

Navigate manually to: `http://localhost:3000`

### Port 3000 is in use

Edit `config.toml` and change the port:

```toml
[server]
port = 8080
```

### Games folder not found

Ensure the path exists and is accessible:

```toml
[paths]
game_library = "D:\\Games"  # Use double backslashes on Windows
```

## Next Steps

- [Configuration](./configuration) - Customize GameVault settings
- [Usage Guide](./usage) - Learn about all features
- [FAQ](./faq) - Common questions and answers
