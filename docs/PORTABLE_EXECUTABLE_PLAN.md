# GameVault Portable Windows Executable Plan

> **Research conducted with:** Google Gemini 3 Pro Preview & OpenAI GPT-5.1 Codex
> **Consensus confidence:** High (7/10)
> **Date:** December 2024

---

## Executive Summary

This document outlines the comprehensive plan to package GameVault as a **portable Windows executable** that:
- Runs without installation
- Contains all components in a single binary
- Allows configurable paths (game library, database, cache)
- Works on any Windows 10/11 machine

**Recommended Approach:** Static Next.js Export + Rust Embedded Server (Option A)

---

## Table of Contents

1. [Current Architecture](#current-architecture)
2. [Options Evaluated](#options-evaluated)
3. [Recommended Solution](#recommended-solution)
4. [Implementation Plan](#implementation-plan)
5. [Configuration System](#configuration-system)
6. [Build Pipeline](#build-pipeline)
7. [Windows-Specific Considerations](#windows-specific-considerations)
8. [Future Enhancements](#future-enhancements)
9. [Risk Assessment](#risk-assessment)

---

## Current Architecture

```
GameVault/
├── backend/           # Rust + Axum web server
│   ├── src/
│   │   ├── main.rs   # Entry point, API routes
│   │   ├── db.rs     # SQLite database operations
│   │   ├── handlers.rs
│   │   └── models.rs
│   └── Cargo.toml
├── frontend/          # Next.js 15 + React 19
│   ├── src/
│   │   ├── app/      # App router pages
│   │   ├── components/
│   │   └── lib/
│   └── package.json
└── data/              # SQLite database
```

**Current Stack:**
- **Backend:** Rust 2021, Axum 0.7, SQLx (SQLite), Tokio
- **Frontend:** Next.js 15, React 19, Tailwind CSS
- **Database:** SQLite (file-based, portable)

---

## Options Evaluated

### Option A: Static Export + Rust Embedded Server ✅ RECOMMENDED

| Aspect | Details |
|--------|---------|
| **Approach** | Export Next.js as static HTML/CSS/JS, embed in Rust binary |
| **Binary Size** | ~15-25 MB |
| **Complexity** | Moderate |
| **Changes Required** | Minimal - build pipeline only |
| **User Experience** | Opens in system browser |

**Pros:**
- Preserves existing architecture
- Single executable output
- No new frameworks to learn
- Low maintenance overhead
- Industry-validated pattern

**Cons:**
- Uses system browser (not native window)
- Requires static export compatibility

### Option B: Tauri Application

| Aspect | Details |
|--------|---------|
| **Approach** | Full migration to Tauri framework |
| **Binary Size** | ~3-10 MB (uses system WebView2) |
| **Complexity** | High |
| **Changes Required** | Major architectural changes |
| **User Experience** | Native window with system tray |

**Pros:**
- Native desktop experience
- System tray integration
- Smaller binary (uses WebView2)
- Deep OS integration possible

**Cons:**
- Requires WebView2 runtime (usually pre-installed on Win10/11)
- Major codebase restructuring
- New build toolchain (Tauri CLI)
- IPC patterns instead of HTTP API

### Option C: Portable Folder Distribution

| Aspect | Details |
|--------|---------|
| **Approach** | Rust exe + frontend folder + 7z self-extractor |
| **Binary Size** | ~20-30 MB (extracted) |
| **Complexity** | Low |
| **Changes Required** | Minimal |
| **User Experience** | Folder with multiple files |

**Pros:**
- Simplest to implement
- No code changes needed
- Easy to update individual components

**Cons:**
- Not a single executable
- Users can break by moving/deleting files
- Less polished distribution

---

## Recommended Solution

### Phase 1: Static Export + Embedded Server (Primary)

This approach achieves the portable executable goal with **minimal changes** to the existing codebase.

```
┌─────────────────────────────────────────────────────────────┐
│                    GameVault.exe (~20MB)                    │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────────────────────┐  │
│  │   Rust Binary   │  │   Embedded Static Assets        │  │
│  │                 │  │   (Next.js export)              │  │
│  │  • Axum Server  │  │                                 │  │
│  │  • SQLite       │  │  • index.html                   │  │
│  │  • Config Mgmt  │  │  • _next/static/...             │  │
│  │  • API Routes   │  │  • CSS, JS bundles              │  │
│  └─────────────────┘  └─────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Alongside Executable                      │
├─────────────────────────────────────────────────────────────┤
│  config.toml          # User configuration                  │
│  data/gamevault.db    # SQLite database                     │
│  cache/               # Cover images, metadata              │
│  logs/                # Application logs                    │
└─────────────────────────────────────────────────────────────┘
```

### Phase 2: Tauri Migration (Optional Future)

Only pursue if native desktop features become essential:
- System tray with quick actions
- Native file drag-and-drop
- Windows notification integration
- Auto-start with Windows

---

## Implementation Plan

### Step 1: Validate Next.js Static Export Compatibility

**Tasks:**
1. Add static export configuration to `next.config.js`:
   ```javascript
   /** @type {import('next').NextConfig} */
   const nextConfig = {
     output: 'export',
     trailingSlash: true,
     images: {
       unoptimized: true  // Required for static export
     }
   };
   module.exports = nextConfig;
   ```

2. Audit all pages for SSR-only features:
   - Replace `getServerSideProps` with client-side fetching
   - Ensure all API calls go to backend, not Next.js API routes
   - Remove any server-only imports

3. Test static build:
   ```bash
   cd frontend
   npm run build
   # Output in frontend/out/
   ```

### Step 2: Add Rust Dependencies

**Update `backend/Cargo.toml`:**
```toml
[dependencies]
# ... existing dependencies ...

# Embedding static files
rust-embed = { version = "8.0", features = ["compression"] }

# Configuration management
config = "0.14"
toml = "0.8"

# Open browser automatically
open = "5.0"

# Portable path handling
directories = "5.0"

# Windows-specific
[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = ["wincon"] }

[profile.release]
opt-level = "z"        # Optimize for size
lto = true             # Link-time optimization
codegen-units = 1      # Better optimization
strip = true           # Strip symbols
panic = "abort"        # Smaller binary
```

### Step 3: Embed Static Assets in Rust

**Create `backend/src/embedded.rs`:**
```rust
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../frontend/out/"]
#[prefix = ""]
pub struct StaticAssets;
```

**Update `backend/src/main.rs` to serve embedded files:**
```rust
use axum::{
    body::Body,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
};

mod embedded;
use embedded::StaticAssets;

async fn serve_static(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    // Try exact path first
    if let Some(content) = StaticAssets::get(path) {
        let mime = mime_guess::from_path(path)
            .first_or_octet_stream();
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime.as_ref())
            .body(Body::from(content.data.into_owned()))
            .unwrap();
    }

    // Try with index.html for directories
    let index_path = format!("{}/index.html", path.trim_end_matches('/'));
    if let Some(content) = StaticAssets::get(&index_path) {
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html")
            .body(Body::from(content.data.into_owned()))
            .unwrap();
    }

    // Fallback to root index.html (SPA routing)
    if let Some(content) = StaticAssets::get("index.html") {
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html")
            .body(Body::from(content.data.into_owned()))
            .unwrap();
    }

    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("Not Found"))
        .unwrap()
}
```

### Step 4: Implement Configuration System

**Create `backend/src/config.rs`:**
```rust
use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub paths: PathsConfig,
    pub server: ServerConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PathsConfig {
    pub game_library: PathBuf,
    pub database: PathBuf,
    pub cache: PathBuf,
    pub logs: PathBuf,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub auto_open_browser: bool,
    pub bind_address: String,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        let config_path = exe_dir.join("config.toml");

        let config = Config::builder()
            // Default values
            .set_default("paths.game_library", ".")?
            .set_default("paths.database", "./data/gamevault.db")?
            .set_default("paths.cache", "./cache")?
            .set_default("paths.logs", "./logs")?
            .set_default("server.port", 3000)?
            .set_default("server.auto_open_browser", true)?
            .set_default("server.bind_address", "127.0.0.1")?
            // Load from file if exists
            .add_source(File::from(config_path).required(false))
            // Environment variable overrides
            .add_source(config::Environment::with_prefix("GAMEVAULT"))
            .build()?;

        config.try_deserialize()
    }
}
```

**Default `config.toml` template:**
```toml
# GameVault Configuration
# Place this file next to GameVault.exe

[paths]
# Root directory containing your games
game_library = "D:\\Games"

# Database file location (relative to executable or absolute)
database = "./data/gamevault.db"

# Cache directory for cover images and metadata
cache = "./cache"

# Log file directory
logs = "./logs"

[server]
# Port to run the web server on
port = 3000

# Automatically open browser when starting
auto_open_browser = true

# Address to bind to (127.0.0.1 = localhost only)
bind_address = "127.0.0.1"
```

### Step 5: Update Main Entry Point

**Modified `backend/src/main.rs`:**
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::net::SocketAddr;
use axum::Router;
use tokio::net::TcpListener;

mod config;
mod embedded;
// ... other modules ...

use config::AppConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = AppConfig::load()?;

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("gamevault=info")
        .init();

    // Ensure directories exist
    std::fs::create_dir_all(&config.paths.cache)?;
    std::fs::create_dir_all(&config.paths.logs)?;
    if let Some(parent) = config.paths.database.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Build router with API routes and static file serving
    let app = Router::new()
        .nest("/api", api_routes())
        .fallback(serve_static);

    let addr = SocketAddr::new(
        config.server.bind_address.parse()?,
        config.server.port
    );

    let listener = TcpListener::bind(addr).await?;

    println!("GameVault running at http://{}", addr);

    // Open browser automatically
    if config.server.auto_open_browser {
        let url = format!("http://localhost:{}", config.server.port);
        if let Err(e) = open::that(&url) {
            eprintln!("Failed to open browser: {}", e);
        }
    }

    axum::serve(listener, app).await?;

    Ok(())
}
```

---

## Configuration System

### Configuration Priority (highest to lowest)

1. **Environment Variables** - `GAMEVAULT_PATHS__GAME_LIBRARY`
2. **Config File** - `config.toml` next to executable
3. **Default Values** - Built-in sensible defaults

### Path Resolution

All relative paths are resolved relative to the executable location:

```
D:\Tools\GameVault\
├── GameVault.exe
├── config.toml           # ./data → D:\Tools\GameVault\data
├── data\
│   └── gamevault.db
├── cache\
│   └── covers\
└── logs\
    └── gamevault.log
```

### First-Run Experience

On first launch without `config.toml`:
1. Application starts with defaults
2. Opens browser to `http://localhost:3000`
3. User can configure paths in the UI
4. Settings are saved to `config.toml`

---

## Build Pipeline

### Development Build Script (`build-portable.ps1`)

```powershell
#!/usr/bin/env pwsh
# Build portable Windows executable

$ErrorActionPreference = "Stop"

Write-Host "Building GameVault Portable..." -ForegroundColor Cyan

# Step 1: Build frontend
Write-Host "`n[1/4] Building frontend..." -ForegroundColor Yellow
Push-Location frontend
npm ci
npm run build
Pop-Location

# Verify static export
if (-not (Test-Path "frontend/out/index.html")) {
    Write-Error "Frontend build failed - no static export found"
    exit 1
}

# Step 2: Build Rust backend (release)
Write-Host "`n[2/4] Building backend..." -ForegroundColor Yellow
Push-Location backend
cargo build --release --target x86_64-pc-windows-msvc
Pop-Location

# Step 3: Create distribution folder
Write-Host "`n[3/4] Creating distribution..." -ForegroundColor Yellow
$distDir = "dist"
Remove-Item -Recurse -Force $distDir -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path $distDir | Out-Null

# Copy executable
Copy-Item "backend/target/x86_64-pc-windows-msvc/release/gamevault-backend.exe" `
    "$distDir/GameVault.exe"

# Copy default config
Copy-Item "config.example.toml" "$distDir/config.toml"

# Create empty directories
New-Item -ItemType Directory -Path "$distDir/data" | Out-Null
New-Item -ItemType Directory -Path "$distDir/cache" | Out-Null

# Step 4: Report size
Write-Host "`n[4/4] Build complete!" -ForegroundColor Green
$exeSize = (Get-Item "$distDir/GameVault.exe").Length / 1MB
Write-Host "Executable size: $([math]::Round($exeSize, 2)) MB"
Write-Host "Distribution folder: $distDir/"
```

### CI/CD Pipeline (GitHub Actions)

```yaml
# .github/workflows/build-windows.yml
name: Build Windows Portable

on:
  push:
    tags: ['v*']
  workflow_dispatch:

jobs:
  build:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: frontend/package-lock.json

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: x86_64-pc-windows-msvc

      - name: Cache Rust dependencies
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: backend

      - name: Build frontend
        run: |
          cd frontend
          npm ci
          npm run build

      - name: Build backend
        run: |
          cd backend
          cargo build --release --target x86_64-pc-windows-msvc

      - name: Package distribution
        run: |
          mkdir dist
          copy backend\target\x86_64-pc-windows-msvc\release\gamevault-backend.exe dist\GameVault.exe
          copy config.example.toml dist\config.toml
          mkdir dist\data
          mkdir dist\cache

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: GameVault-Windows-Portable
          path: dist/

      - name: Create release
        if: startsWith(github.ref, 'refs/tags/')
        uses: softprops/action-gh-release@v1
        with:
          files: |
            dist/GameVault.exe
            dist/config.toml
```

---

## Windows-Specific Considerations

### 1. Hide Console Window

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
```

This hides the console window in release builds while keeping it visible during development.

### 2. Windows Manifest

Create `backend/gamevault.manifest`:
```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true/pm</dpiAware>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">permonitorv2</dpiAwareness>
    </windowsSettings>
  </application>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <!-- Windows 10 and 11 -->
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/>
    </application>
  </compatibility>
</assembly>
```

Add to `Cargo.toml`:
```toml
[package.metadata.winres]
FileDescription = "GameVault - Game Library Manager"
ProductName = "GameVault"
OriginalFilename = "GameVault.exe"
LegalCopyright = "Copyright (c) 2024"
```

Add build dependency:
```toml
[build-dependencies]
winres = "0.1"
```

Create `backend/build.rs`:
```rust
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_manifest_file("gamevault.manifest");
        res.compile().unwrap();
    }
}
```

### 3. Code Signing (Recommended for Distribution)

To avoid Windows SmartScreen warnings:
1. Obtain a code signing certificate (DigiCert, Sectigo, etc.)
2. Sign the executable:
   ```powershell
   signtool sign /tr http://timestamp.digicert.com /td sha256 /fd sha256 /a GameVault.exe
   ```

### 4. Portable Path Handling

```rust
use std::path::PathBuf;

fn get_exe_directory() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn resolve_path(relative: &str) -> PathBuf {
    let path = PathBuf::from(relative);
    if path.is_absolute() {
        path
    } else {
        get_exe_directory().join(path)
    }
}
```

---

## Future Enhancements

### Phase 2: Tauri Migration (If Needed)

Pursue only if these features become essential:

| Feature | Benefit |
|---------|---------|
| System Tray | Quick access, minimize to tray |
| Native Notifications | Game updates, scan complete |
| File Drag & Drop | Add games by dropping folders |
| Auto-Start | Launch with Windows |
| Deep Linking | `gamevault://open/game/123` |

**Estimated Additional Effort:** Significant (major refactor)

### Other Enhancements

1. **Auto-Update System**
   - Check for updates on GitHub releases
   - Download and replace executable

2. **Portable Mode Detection**
   - Check for `portable.txt` marker file
   - Store all data relative to exe vs. AppData

3. **Single Instance Lock**
   - Prevent multiple instances
   - Focus existing window if running

4. **CLI Arguments**
   ```
   GameVault.exe --port 8080 --no-browser --config custom.toml
   ```

---

## Risk Assessment

### Low Risk
- SQLite portability (already file-based)
- Rust binary compilation (well-supported)
- Static asset embedding (proven pattern)

### Medium Risk
- **Next.js Static Export Compatibility**
  - *Mitigation:* Audit all pages before implementation
  - *Fallback:* Refactor SSR features to client-side

- **Binary Size Growth**
  - *Mitigation:* Enable LTO, compression in rust-embed
  - *Expected:* ~15-25 MB (acceptable)

### Considerations
- **Windows Defender/SmartScreen**
  - Unsigned executables may trigger warnings
  - *Mitigation:* Code signing certificate

- **Port Conflicts**
  - Port 3000 may be in use
  - *Mitigation:* Allow configurable port, auto-detect available port

---

## Checklist

### Pre-Implementation
- [ ] Audit Next.js pages for SSR-only features
- [ ] Test static export builds correctly
- [ ] Verify all API calls use backend (not Next.js API routes)

### Implementation
- [ ] Add rust-embed and configuration crates
- [ ] Create embedded.rs for static assets
- [ ] Implement config.rs with TOML loading
- [ ] Update main.rs with embedded serving
- [ ] Add Windows manifest for DPI awareness
- [ ] Create build-portable.ps1 script
- [ ] Test on clean Windows install

### Distribution
- [ ] Create config.example.toml with documentation
- [ ] Write user README for portable version
- [ ] Set up GitHub Actions for releases
- [ ] Consider code signing certificate

---

## Conclusion

The **Static Export + Rust Embedded Server** approach provides the optimal balance of:

- **Minimal code changes** to existing architecture
- **Single portable executable** output
- **Configurable paths** via TOML configuration
- **No installation required** - truly portable
- **Industry-validated pattern** used by many similar tools

This approach can be implemented incrementally alongside existing development, with Tauri remaining a future option if native desktop features become necessary.

---

*Document generated with AI consensus from Google Gemini 3 Pro Preview and OpenAI GPT-5.1 Codex*
