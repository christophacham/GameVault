---
sidebar_position: 6
---

# FAQ & Troubleshooting

Common questions and solutions for GameVault.

## General Questions

### What is GameVault?

GameVault is a portable game library manager for Windows that helps you organize your local game collection and enrich it with metadata from Steam.

### Is GameVault free?

Yes, GameVault is completely free and open source.

### Does GameVault modify my game files?

No. GameVault only reads folder names and sizes. The only files it creates are:
- `.gamevault/metadata.json` in each game folder (optional, for metadata backup)
- Cover images in the `cache/` folder

### Does GameVault require an internet connection?

- **Browsing**: No, works offline
- **Enrichment**: Yes, requires internet to fetch Steam metadata
- **Scanning**: No, works offline

## Installation Issues

### GameVault won't start

**Possible causes:**
1. **Port in use**: Another application is using port 3000
2. **Antivirus blocking**: Some antivirus may block the executable
3. **Missing Visual C++ Runtime**: Rarely needed, but try installing it

**Solutions:**
1. Change port in `config.toml` to 8080
2. Add GameVault to antivirus exceptions
3. Install [Visual C++ Redistributable](https://aka.ms/vs/17/release/vc_redist.x64.exe)

### Browser doesn't open automatically

1. Check `auto_open_browser = true` in `config.toml`
2. Navigate manually to `http://localhost:3000`
3. Check if running in Docker/headless mode

### "Address already in use" error

Another process is using the port. Either:
1. Close the other application
2. Change GameVault's port:
   ```toml
   [server]
   port = 8080
   ```

## Scanning Issues

### Games not detected

**Check:**
1. Is the `game_library` path correct?
2. Do game folders exist in that path?
3. Are folder names recognizable as game titles?

**Folder structure expected:**
```
D:\Games\
├── The Witcher 3\
├── Cyberpunk 2077\
└── Elden Ring\
```

### Wrong game detected

Some folder names may confuse the scanner. Solutions:
1. Rename the folder to match the game title
2. Use the **Adjust Match** feature after scanning

### Scan is slow

Large libraries take time. GameVault scans recursively, so:
- 100 games: ~5 seconds
- 500 games: ~15 seconds
- 1000+ games: ~30+ seconds

## Enrichment Issues

### Enrichment fails for some games

**Common causes:**
1. Game title doesn't match Steam's database
2. Game isn't on Steam
3. Rate limiting from Steam API

**Solutions:**
1. Use **Adjust Match** to manually set the correct Steam ID
2. Edit game details manually
3. Wait and retry (rate limiting resets)

### Wrong game matched

Use **Adjust Match**:
1. Click the game card menu (three dots)
2. Select **Adjust Match**
3. Enter the correct Steam URL or App ID:
   - URL: `https://store.steampowered.com/app/292030/`
   - App ID: `292030`
4. Preview and confirm

### Cover images not loading

**Check:**
1. Internet connection
2. Cache folder exists and is writable
3. Steam store is accessible

**Reset cache:**
1. Close GameVault
2. Delete `cache/` folder
3. Restart and re-enrich

## Configuration Issues

### Settings not saving

**Check:**
1. `config.toml` is not read-only
2. GameVault has write permissions to its folder
3. No syntax errors in config file

### Config file corrupted

Delete `config.toml` and restart GameVault to regenerate defaults.

### Changes don't take effect

Some settings require restart:
- Server port
- Bind address

Use the **Restart** button in Settings.

## Database Issues

### Database is corrupted

1. Close GameVault
2. Delete `data/gamevault.db`
3. Restart GameVault
4. Re-scan and re-enrich

Your `.gamevault/metadata.json` files preserve metadata, use **Import** to restore.

### Games missing after update

Try re-scanning:
1. Click **Scan**
2. All games in your library folder will be detected

## Performance Issues

### Web interface is slow

1. Reduce number of games displayed (use search)
2. Check browser console for errors
3. Clear browser cache

### High memory usage

GameVault typically uses 30-50 MB RAM. If higher:
1. Restart the application
2. Check for many cached images

## Network Issues

### Can't access from other devices

1. Change bind address:
   ```toml
   [server]
   bind_address = "0.0.0.0"
   ```
2. Allow through Windows Firewall
3. Access via your IP: `http://192.168.1.x:3000`

### CORS errors in browser

If accessing from a different origin, check `CORS_ORIGINS` environment variable.

## Editing Issues

### Can't save edited game details

**Check:**
1. Title field is not empty (required)
2. Date format is YYYY-MM-DD (e.g., 2023-12-25)
3. Review score is 0-100

### Keyboard shortcuts not working

Ensure the modal or menu has focus:
- Click on the modal/menu first
- Use **Tab** to navigate between elements
- **Escape** closes modals and menus
- **Arrow keys** navigate menu items

## Getting Help

### Where to report bugs?

Open an issue on the GitHub repository with:
- Steps to reproduce
- Expected vs actual behavior
- GameVault version
- Windows version

### Where are log files?

Logs are output to the console. For file logging, run from terminal:
```powershell
.\GameVault.exe > logs\output.log 2>&1
```
