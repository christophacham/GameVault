---
sidebar_position: 5
---

# App Settings

GameVault provides both file-based configuration and a runtime settings modal.

## Settings Modal

Access the Settings modal by clicking the **gear icon** in the header.

### Paths Section

#### Game Library Path

The root folder containing your games.

```
D:\Games
```

**Indicators:**
- **Yellow border**: Path not configured (needs setup)
- **Red warning**: Path does not exist
- **No warning**: Path is valid and accessible

#### Cache Directory

Where cover images are stored.

```
./cache
```

Relative paths are resolved from the GameVault.exe location.

### Server Section

#### Server Port

The HTTP port for the web interface.

| Value | Description |
|-------|-------------|
| `3000` | Default port |
| `8080` | Common alternative |
| `1024-65535` | Valid range |

:::info
Port changes require a restart to take effect.
:::

#### Auto-open Browser

When enabled, your default browser opens automatically on startup.

### Actions Section

#### Restart Button

Restarts GameVault:
- Spawns a new process
- Closes the current process
- Page reloads automatically after 2 seconds

Use restart when:
- Changing the server port
- After configuration changes
- If the app becomes unresponsive

#### Shutdown Button

Closes GameVault completely:
- Stops the web server
- Removes system tray icon
- Terminates the process

## System Tray Integration

GameVault runs in the Windows system tray.

### Tray Icon

The purple GameVault icon appears in your system tray when running.

### Context Menu

Right-click the tray icon for options:

| Option | Action |
|--------|--------|
| **Open GameVault** | Opens the web interface in your browser |
| **Quit** | Closes GameVault completely |

### Tooltip

Hover over the tray icon to see:
```
GameVault - localhost:3000
```

### Starting Minimized

GameVault currently opens the browser on startup (if configured). To start minimized:

1. Set `auto_open_browser = false` in `config.toml`
2. Access via system tray when needed

## Runtime vs Persistent Settings

### Persistent Settings (config.toml)

Saved to disk, persist across restarts:
- Game library path
- Cache directory
- Server port
- Auto-open browser
- Bind address

### Runtime State

Not persisted, reset on restart:
- Current view/filter state
- Search query
- Modal open state

## Saving Settings

When you click **Save Settings**:

1. Settings are validated
2. Paths are checked for existence
3. `config.toml` is updated atomically
4. Success message is shown
5. Modal closes

### Validation Errors

| Error | Cause | Solution |
|-------|-------|----------|
| "Path does not exist" | Game library folder not found | Create the folder or fix the path |
| "Port must be between 1024 and 65535" | Invalid port number | Use a valid port |
| "Failed to save configuration" | Write permission issue | Check folder permissions |

## Restart Required Warning

Some settings require a restart:
- **Server Port**: Binding to a new port requires restart
- (Future: Bind address changes)

The Settings modal shows a yellow warning for these fields.

## First-Run Experience

On first run with an empty `game_library`:

1. Settings modal shows yellow warning
2. "Please set your games folder path" message
3. User enters their games folder
4. Click Save and then Scan

This guides new users to configure before using.
