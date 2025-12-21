---
sidebar_position: 3
---

# Configuration

GameVault uses a `config.toml` file for persistent settings. This file is created automatically on first run.

## Configuration File Location

The `config.toml` file is located next to `GameVault.exe`:

```
GameVault/
├── GameVault.exe
├── config.toml      ← Configuration file
└── ...
```

## Full Configuration Reference

```toml
# GameVault Configuration
# ======================

[paths]
# Root directory containing your games to scan
# Examples:
#   game_library = "D:\\Games"
#   game_library = "C:\\Users\\YourName\\Games"
#   game_library = "./games"  (relative to executable)
game_library = ""

# SQLite database file location
# The database will be created automatically on first run
database = "sqlite:./data/gamevault.db?mode=rwc"

# Cache directory for downloaded cover images and metadata
cache = "./cache"

[server]
# Port to run the web server on (default: 3000)
port = 3000

# Automatically open your default browser when GameVault starts
auto_open_browser = true

# Network address to bind to
# - "127.0.0.1" = localhost only (more secure, default)
# - "0.0.0.0" = all network interfaces (accessible from other devices)
bind_address = "127.0.0.1"
```

## Configuration Options

### Paths Section

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `game_library` | string | `""` | Root folder containing your games |
| `database` | string | `sqlite:./data/gamevault.db?mode=rwc` | SQLite database path |
| `cache` | string | `./cache` | Directory for cached images |

### Server Section

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `port` | number | `3000` | HTTP server port |
| `auto_open_browser` | boolean | `true` | Open browser on startup |
| `bind_address` | string | `127.0.0.1` | Network interface to bind |

## Path Resolution

Paths can be **absolute** or **relative**:

```toml
# Absolute path (recommended for game library)
game_library = "D:\\Games"

# Relative path (relative to GameVault.exe location)
cache = "./cache"
```

## Environment Variables

You can override configuration using environment variables:

```bash
# Override game library path
set GAMEVAULT_PATHS__GAME_LIBRARY=D:\Games

# Override server port
set GAMEVAULT_SERVER__PORT=8080

# Override auto-open browser
set GAMEVAULT_SERVER__AUTO_OPEN_BROWSER=false
```

Legacy environment variables are also supported:

| Variable | Maps to |
|----------|---------|
| `DATABASE_URL` | `paths.database` |
| `GAMES_PATH` | `paths.game_library` |
| `PORT` | `server.port` |
| `HOST` | `server.bind_address` |

## Settings Modal

Most settings can be changed through the web interface:

1. Click the **gear icon** in the header
2. Modify settings as needed
3. Click **Save Settings**

Changes to the port require a restart. Use the **Restart** button in Settings.

## Network Access

To access GameVault from other devices on your network:

```toml
[server]
bind_address = "0.0.0.0"  # Listen on all interfaces
```

Then access via your computer's IP address: `http://192.168.1.x:3000`

:::warning Security Note
Only expose GameVault on trusted networks. There is no authentication by default.
:::

## API Key Authentication

For additional security, set an API key:

```bash
set API_KEY=your-secret-key
```

Protected endpoints (scan, enrich, export) will require:

```
Authorization: Bearer your-secret-key
```

## Atomic Configuration Updates

Configuration changes are written atomically:

1. Changes written to `config.toml.tmp`
2. Temporary file renamed to `config.toml`
3. Prevents corruption if write is interrupted

## Resetting Configuration

To reset to defaults:

1. Close GameVault
2. Delete `config.toml`
3. Start GameVault (new config created automatically)

To reset database and cache:

1. Close GameVault
2. Delete `data/` and `cache/` folders
3. Start GameVault
