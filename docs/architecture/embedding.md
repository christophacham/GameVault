---
sidebar_position: 5
---

# Asset Embedding

GameVault uses `rust-embed` to compile the Next.js frontend into the Rust binary, creating a single portable executable.

## How It Works

### Build Process

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  npm run build  │────▶│   frontend/out  │────▶│  rust-embed     │
│  (Next.js)      │     │   (static HTML) │     │  (compile-time) │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                                                        │
                                                        ▼
                                                ┌─────────────────┐
                                                │  GameVault.exe  │
                                                │  (single file)  │
                                                └─────────────────┘
```

### rust-embed Configuration

```rust
// backend/src/embedded.rs

use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../frontend/out/"]  // Relative to Cargo.toml
#[prefix = ""]                  // No prefix for file paths
pub struct StaticAssets;
```

At compile time, `rust-embed` scans `frontend/out/` and embeds all files as compressed binary data.

## Static File Serving

### Path Resolution

```rust
pub async fn serve_static(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    // Priority order:
    // 1. Exact path match
    // 2. Path + .html (Next.js static export)
    // 3. Path + /index.html (directory)
    // 4. Fallback to index.html (SPA routing)
}
```

### Request Flow

```
GET /games ──▶ serve_static
    │
    ├─ Try: StaticAssets::get("games")     ❌ Not found
    ├─ Try: StaticAssets::get("games.html") ✅ Found!
    │
    └─ Return: games.html with text/html content-type
```

### Cache Control

```rust
fn get_cache_control(path: &str) -> &'static str {
    if path.contains("/_next/static/") {
        // Hashed files - cache forever
        "public, max-age=31536000, immutable"
    } else if path.ends_with(".woff2") || path.ends_with(".ttf") {
        // Fonts - cache forever
        "public, max-age=31536000, immutable"
    } else if is_image(path) {
        // Images - cache for a day
        "public, max-age=86400"
    } else {
        // HTML and other - no cache
        "no-cache"
    }
}
```

## Compression

### rust-embed Compression

By default, rust-embed compresses embedded files with gzip:

```toml
# Cargo.toml
rust-embed = { version = "8.2", features = ["compression"] }
```

### Size Comparison

| Component | Uncompressed | Embedded |
|-----------|-------------|----------|
| Frontend (Next.js) | ~15 MB | ~3 MB |
| Rust Backend | ~15 MB | ~15 MB |
| **Total Binary** | - | **~20 MB** |

## Build Requirements

### Frontend Must Be Built First

The frontend must exist before compiling the backend:

```bash
# Order matters!
cd frontend && npm run build  # Creates frontend/out/
cd backend && cargo build     # Embeds frontend/out/
```

### Missing Frontend Error

If `frontend/out/` doesn't exist:

```
error: proc-macro derive panicked
  --> src/embedded.rs:5:10
   |
5  | #[folder = "../frontend/out/"]
   |           ^^^^^^^^^^^^^^^^^^^^
   = help: message: folder '../frontend/out/' does not exist
```

### CI/CD Workaround

For backend-only tests, create a minimal placeholder:

```dockerfile
# Dockerfile.test
RUN mkdir -p frontend/out && \
    echo '<!DOCTYPE html><html></html>' > frontend/out/index.html
```

## Cross-Compilation

### Windows Build from Linux

The `Dockerfile.windows` cross-compiles for Windows:

```dockerfile
# Add Windows target
RUN rustup target add x86_64-pc-windows-gnu

# Configure linker
RUN echo '[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"' > ~/.cargo/config.toml

# Copy frontend build
COPY --from=frontend-builder /app/frontend/out /app/frontend/out

# Build for Windows
RUN cargo build --release --target x86_64-pc-windows-gnu
```

### Build Script

```powershell
# build-portable.ps1
podman build -f Dockerfile.windows -t gamevault-windows-builder .
podman create gamevault-windows-builder
podman cp container:/output/GameVault.exe dist/
```

## Runtime Behavior

### Asset Access

```rust
// Check if asset exists
if StaticAssets::get("favicon.ico").is_some() {
    // Asset is embedded
}

// Get asset content
if let Some(content) = StaticAssets::get("index.html") {
    let bytes: &[u8] = &content.data;
    let owned: Vec<u8> = content.data.into_owned();
}

// List all embedded files
for file in StaticAssets::iter() {
    println!("{}", file);  // "index.html", "_next/static/...", etc.
}
```

### No External Files Required

The embedded assets are served directly from memory. No file extraction or temporary files are needed.

## Development Mode

### Hot Reload Setup

During development, run frontend and backend separately:

```bash
# Terminal 1: Frontend dev server
cd frontend && npm run dev  # Port 5173

# Terminal 2: Backend
cd backend && cargo run     # Port 3000
```

The backend serves the embedded placeholder while the frontend dev server handles actual requests.

### CORS Configuration

CORS allows the frontend dev server:

```rust
let default_origins = vec![
    "http://localhost:3000",
    "http://localhost:5173",  // Vite dev server
];
```

## Troubleshooting

### "Index.html not found" at runtime

1. Ensure frontend was built before backend compilation
2. Check `frontend/out/index.html` exists
3. Rebuild backend after frontend changes

### Large binary size

1. Enable LTO and stripping in release:
   ```toml
   [profile.release]
   lto = true
   strip = true
   ```

2. Check for debug symbols:
   ```toml
   [profile.release]
   debug = false
   ```

### Stale frontend

Frontend changes require rebuilding both:

```bash
cd frontend && npm run build
cd backend && cargo build --release
```

rust-embed embeds at compile time, not runtime.
