# GameVault

A self-hosted game library application - like Plex, but for your games.

![GameVault](https://img.shields.io/badge/GameVault-v0.1.0-blue)
![Rust](https://img.shields.io/badge/Rust-1.85-orange)
![Next.js](https://img.shields.io/badge/Next.js-15-black)
![Docker](https://img.shields.io/badge/Docker-Ready-blue)
![Tests](https://img.shields.io/badge/Tests-Passing-green)

## Features

- Scans your local game folders automatically
- Cleans up folder names (removes "[FitGirl Repack]", etc.)
- Fetches metadata from Steam API (cover art, descriptions, reviews)
- Beautiful dark-themed UI with responsive grid
- Search and browse your collection
- Review scores (recent and lifetime)
- **Edit game details** - Manually edit titles, descriptions, and metadata
- **Adjust Steam matches** - Fix incorrect game matches with correct Steam ID
- **Keyboard navigation** - Full accessibility support in menus
- **Portable Windows executable** - Single .exe with embedded frontend
- Single Docker container deployment
- **Optional API authentication** for sensitive endpoints
- **Secure by default** - localhost-only binding

## Quick Start

### Option 1: Docker/Podman (Recommended)

```bash
# Navigate to project
cd GameVault

# Build and start
docker-compose up -d
# or with Podman
podman-compose up -d

# View logs
docker-compose logs -f

# Access at http://localhost:3000
```

### Option 2: Run Directly with Podman

```bash
podman run -d \
  --name gamevault \
  -p 3000:3000 \
  -v /path/to/your/games:/games:ro \
  -v ./data:/data \
  -e HOST=0.0.0.0 \
  gamevault:latest
```

### Option 3: Windows Portable Executable (Recommended for Windows)

A single portable `.exe` with the frontend embedded - no installation required.

#### Using Pre-built Release

1. Download `GameVault.exe` from the releases page
2. Place it in any folder
3. Double-click to run - browser opens automatically
4. Configure your game library path in Settings

#### Building from Source (Windows with Podman)

```powershell
# Build the portable executable
./build-portable.ps1

# Output: dist/GameVault.exe (~20MB)
# Run it directly - no dependencies needed
./dist/GameVault.exe
```

The build uses cross-compilation from Linux in a container to produce a Windows executable.

### Option 4: Manual Build

#### Prerequisites
- [Rust](https://rustup.rs/) 1.75+
- [Node.js](https://nodejs.org/) 20+

#### Backend

```bash
cd backend
cargo build --release
```

#### Frontend

```bash
cd frontend
npm install
npm run build
```

#### Run

```bash
# Set environment
export DATABASE_URL="sqlite:./data/games.db?mode=rwc"
export GAMES_PATH="/path/to/games"
export PORT=3000

# Copy frontend to backend public folder
cp -r frontend/out backend/public

# Run server
./backend/target/release/gamevault-backend
```

## Configuration

### TOML Configuration (Windows Portable)

For the Windows portable executable, create a `config.toml` next to `GameVault.exe`:

```toml
[paths]
game_library = "D:\\Games"
database = "sqlite:./data/gamevault.db?mode=rwc"
cache = "./cache"

[server]
port = 3000
auto_open_browser = true
bind_address = "127.0.0.1"
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `sqlite:///data/games.db?mode=rwc` | SQLite database path |
| `GAMES_PATH` | `/games` | Path to scan for games |
| `PORT` | `3000` | Server port |
| `HOST` | `127.0.0.1` | Bind address (use `0.0.0.0` for network) |
| `RUST_LOG` | `info` | Log level (debug, info, warn, error) |
| `API_KEY` | *(none)* | API key for protected endpoints |
| `CORS_ORIGINS` | localhost only | Comma-separated allowed origins |

### Docker Volumes

| Container Path | Purpose |
|----------------|---------|
| `/games` | Your game folders (mount read-only) |
| `/data` | SQLite database storage |

### docker-compose.yml

```yaml
services:
  gamevault:
    build: .
    container_name: gamevault
    ports:
      - "3000:3000"
    volumes:
      - /path/to/games:/games:ro
      - ./data:/data
    environment:
      - DATABASE_URL=sqlite:///data/games.db?mode=rwc
      - GAMES_PATH=/games
      - RUST_LOG=info
      # Optional: Enable authentication
      - API_KEY=your-secret-key-here
      # Optional: Allow additional CORS origins
      - CORS_ORIGINS=https://my-app.example.com
    restart: unless-stopped
```

## Authentication

By default, all endpoints are open. To protect sensitive operations, set the `API_KEY` environment variable.

### Enabling Authentication

```bash
# Add to docker-compose.yml or docker run
API_KEY=my-super-secret-key-change-this
```

### Making Authenticated Requests

Include the API key in the `Authorization` header:

```bash
# Using Bearer token (recommended)
curl -X POST http://localhost:3000/api/scan \
  -H "Authorization: Bearer my-super-secret-key-change-this"

# Or raw API key
curl -X POST http://localhost:3000/api/scan \
  -H "Authorization: my-super-secret-key-change-this"
```

### Protected vs Public Endpoints

| Endpoint | Auth Required | Description |
|----------|---------------|-------------|
| `GET /api/health` | No | Health check |
| `GET /api/games` | No | List all games |
| `GET /api/games/:id` | No | Get game details |
| `GET /api/games/search?q=` | No | Search games |
| `GET /api/stats` | No | Get library statistics |
| `POST /api/scan` | **Yes*** | Scan for new games |
| `POST /api/enrich` | **Yes*** | Fetch Steam metadata |

*Only required when `API_KEY` is set. Without `API_KEY`, all endpoints are open.

## Usage

1. **Start** - Run the container or server
2. **Scan** - Call `/api/scan` or click "Scan" in UI to detect game folders
3. **Enrich** - Call `/api/enrich` or click "Enrich" to fetch metadata from Steam
4. **Browse** - Search and view your game collection

### API Examples

**List all games:**
```bash
curl http://localhost:3000/api/games
```

**Get single game:**
```bash
curl http://localhost:3000/api/games/42
```

**Search games:**
```bash
curl "http://localhost:3000/api/games/search?q=witcher"
```

**Get statistics:**
```bash
curl http://localhost:3000/api/stats
```

Response:
```json
{
  "success": true,
  "data": {
    "total_games": 185,
    "matched_games": 171,
    "pending_games": 14,
    "enriched_games": 171
  }
}
```

**Scan for games (protected):**
```bash
curl -X POST http://localhost:3000/api/scan \
  -H "Authorization: Bearer $API_KEY"
```

**Enrich with Steam data (protected):**
```bash
curl -X POST http://localhost:3000/api/enrich \
  -H "Authorization: Bearer $API_KEY"
```

Run enrichment multiple times to process all games (20 per call, rate-limited).

## CORS Configuration

By default, only localhost origins are allowed:
- `http://localhost:3000`
- `http://127.0.0.1:3000`
- `http://localhost:5173` (Vite dev server)
- `http://127.0.0.1:5173`

To allow additional origins:

```bash
# Single origin
CORS_ORIGINS=https://my-domain.com

# Multiple origins (comma-separated)
CORS_ORIGINS=https://app.example.com,https://admin.example.com
```

## Project Structure

```
GameVault/
├── backend/
│   ├── src/
│   │   ├── main.rs          # Axum server, routing, auth middleware
│   │   ├── handlers.rs      # API endpoint handlers
│   │   ├── db.rs            # SQLite operations (with transactions)
│   │   ├── scanner.rs       # Directory scanning
│   │   ├── steam.rs         # Steam API client
│   │   ├── models.rs        # Data structures
│   │   ├── embedded.rs      # Embedded static assets (rust-embed)
│   │   └── local_storage.rs # Local metadata storage
│   ├── Cargo.toml
│   └── build.rs             # Windows resource embedding
├── frontend/
│   ├── src/
│   │   ├── app/             # Next.js app router
│   │   ├── components/      # React components
│   │   │   ├── EditModal.tsx        # Edit game metadata
│   │   │   ├── AdjustMatchModal.tsx # Fix Steam matches
│   │   │   ├── GameMenu.tsx         # Context menu (keyboard accessible)
│   │   │   └── GameCard.tsx         # Game display card
│   │   ├── lib/             # API client
│   │   └── test/            # Test setup
│   ├── vitest.config.ts     # Test configuration
│   └── package.json
├── docs/
│   └── docusaurus/          # Documentation site
├── Dockerfile               # Linux container build
├── Dockerfile.windows       # Windows cross-compilation
├── Dockerfile.test          # Test runner container
├── docker-compose.yml
├── build-portable.ps1       # Windows portable build script
├── config.example.toml      # Example configuration
└── README.md
```

## Tech Stack

### Backend
- **Rust** - Systems programming language
- **Axum** - Web framework
- **SQLite** - Embedded database
- **SQLx** - Async SQL toolkit
- **Reqwest** - HTTP client
- **Strsim** - Fuzzy string matching

### Frontend
- **Next.js 15** - React framework
- **React 19** - UI library
- **TailwindCSS** - Utility-first CSS
- **TypeScript** - Type safety
- **Vitest** - Unit testing framework
- **Testing Library** - React component testing

### Infrastructure
- **Docker/Podman** - Containerization
- **Alpine Linux** - Minimal runtime
- **rust-embed** - Static asset embedding for portable build

## Game Matching

GameVault uses fuzzy matching to identify games:

1. **Folder Cleanup** - Removes common suffixes:
   - `[FitGirl Repack]`, `[DODI Repack]`
   - Version numbers (`v1.63`)
   - Edition suffixes (`HRTP`, `EE`, `NG`)

2. **Known Mappings** - 200+ pre-configured title-to-Steam-ID mappings

3. **Steam Search** - Queries Steam API with cleaned title

4. **Similarity Scoring** - Uses Jaro-Winkler algorithm:
   - > 0.85: Auto-match
   - 0.60-0.85: Manual review recommended
   - < 0.60: No match

## Security

GameVault includes several security features:

- **Localhost-only by default** - Set `HOST=0.0.0.0` to expose to network
- **Optional API authentication** - Protect scan/enrich with `API_KEY`
- **CORS restrictions** - Localhost-only, configurable via `CORS_ORIGINS`
- **No path exposure** - Local filesystem paths hidden from API responses
- **Input validation** - Search queries validated (1-200 chars)
- **Error sanitization** - Database errors don't leak to clients

## Troubleshooting

### Container won't start
```bash
docker logs gamevault
# or
podman logs gamevault
```

### Database errors
```bash
# Reset database
rm -rf data/games.db*
docker-compose restart
```

### Games not found
- Check `GAMES_PATH` is correctly mounted
- Ensure folder names contain recognizable game titles
- Check logs for "Excluding non-game content" messages

### Steam API issues
- Rate limiting: Wait between enrichment runs
- Some games (Epic exclusives) won't have Steam data

### Can't connect from other devices
- Set `HOST=0.0.0.0` to bind to all interfaces
- Add your frontend origin to `CORS_ORIGINS`

### Authentication errors
- Ensure `API_KEY` is set in the container environment
- Include the exact key in the `Authorization` header
- Check for trailing whitespace in your API key

## Development

### Run Tests

```bash
# Backend tests (17 tests)
cd backend
cargo test

# Frontend tests (28 tests)
cd frontend
npm test

# Run all tests in containers (CI/CD ready)
podman build -f Dockerfile.test -t gamevault-test .
```

### Hot Reload (Development)

```bash
# Terminal 1: Backend
cd backend
cargo watch -x run

# Terminal 2: Frontend
cd frontend
npm run dev
```

### Windows Build Scripts

```powershell
# Development (hot reload)
./dev.ps1

# Build portable executable
./build-portable.ps1

# Start/stop in background
./start.ps1
./stop.ps1
```

## Roadmap

- [x] Manual game editing UI
- [x] Manual game matching UI (Adjust Match)
- [x] Keyboard navigation and accessibility
- [x] Portable Windows executable
- [ ] IGDB integration for better metadata
- [ ] Epic Games Store support
- [ ] GOG integration
- [ ] Game launching

## License

MIT

## Credits

Built with Rust, Next.js, and the Steam API.
