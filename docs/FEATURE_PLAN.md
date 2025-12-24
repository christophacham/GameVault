# GameVault Feature Plan

## Overview
6 UX features planned with consensus from Gemini 3 Pro Preview.

## Features to Implement

### 1. Save Game Backup
- **Location**: `.gamevault/saves/{timestamp}.zip` within each game folder
- **Data Source**: Ludusavi manifest for save path patterns
- **Prerequisite**: Check write permissions before backup
- **Endpoint**: `POST /api/games/:id/backup`

### 2. Recently Added Row
- **UI**: Horizontal scroll of last 10 games on dashboard
- **Backend**: `GET /api/games/recent?limit=10`
- **Sort**: By `created_at` DESC

### 3. HLTB Integration (HowLongToBeat)
- **Data**: Main story, Main+Extras, Completionist times
- **Strategy**: Background job with 30-day DB cache
- **Rate Limit**: 1 request per 2 seconds (no official API)
- **Columns**: `hltb_main_mins`, `hltb_extra_mins`, `hltb_completionist_mins`

### 4. Manual Fix Match UI
- **Purpose**: User manually sets Steam AppID for failed matches
- **Endpoint**: `POST /api/games/:id/match` with `{ steam_app_id }`
- **DB**: Add `match_locked` to prevent auto-overwrite
- **UI**: Modal with Steam search, shows preview before confirming

### 5. Collections/Tags
- **Tables**: `collections` (id, name, color) + `game_collections` (game_id, collection_id)
- **Endpoints**: CRUD for collections, assign/remove games
- **UI**: Tag chips on cards, filter sidebar, manage modal

### 6. Backlog Filter
- **Column**: `user_status` (unplayed, playing, completed, abandoned)
- **UI**: Filter dropdown with presets
- **Bonus**: `playtime_mins` for future tracking

---

## Database Schema Changes

```sql
-- Games table additions (metadata - shared)
ALTER TABLE games ADD COLUMN hltb_main_mins INTEGER;
ALTER TABLE games ADD COLUMN hltb_extra_mins INTEGER;
ALTER TABLE games ADD COLUMN hltb_completionist_mins INTEGER;
ALTER TABLE games ADD COLUMN save_path_pattern TEXT;
ALTER TABLE games ADD COLUMN local_cover_path TEXT;      -- cached artwork
ALTER TABLE games ADD COLUMN local_background_path TEXT; -- cached artwork

-- User state additions (on games table for single-user)
ALTER TABLE games ADD COLUMN match_locked BOOLEAN DEFAULT 0;
ALTER TABLE games ADD COLUMN playtime_mins INTEGER DEFAULT 0;
ALTER TABLE games ADD COLUMN user_status TEXT DEFAULT 'unplayed';

-- Collections
CREATE TABLE collections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    color TEXT DEFAULT '#6366f1',
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE game_collections (
    game_id INTEGER REFERENCES games(id) ON DELETE CASCADE,
    collection_id INTEGER REFERENCES collections(id) ON DELETE CASCADE,
    PRIMARY KEY (game_id, collection_id)
);
```

---

## Local Storage Structure (per game folder)

```
Game Folder/
├── game.exe
├── ...
└── .gamevault/
    ├── cover.jpg           # Cached cover art
    ├── background.jpg      # Cached background art
    └── saves/
        ├── 2024-01-15_120000.zip
        └── 2024-01-20_180000.zip
```

---

## Implementation Order

### Phase 0: Infrastructure (DO FIRST) - COMPLETE
1. Local artwork caching in game folders
2. Write permission checking
3. .gamevault folder structure

### Phase 1: Foundation
1. DB Migration - COMPLETE
2. Recently Added Row - Backend COMPLETE, Frontend pending
3. Backlog Filter + Status - Schema ready, UI pending

### Phase 2: Organization
4. Collections/Tags CRUD
5. Manual Fix Match UI

### Phase 3: Enrichment
6. HLTB Integration
7. Save Game Backup (Ludusavi)

---

## API Endpoints (New)

| Method | Endpoint | Description | Status |
|--------|----------|-------------|--------|
| GET | `/api/games/recent` | Last N games by created_at | DONE |
| GET | `/api/games/:id/cover` | Serve local cover image | DONE |
| GET | `/api/games/:id/background` | Serve local background image | DONE |
| GET | `/api/games/:id/storage` | Check folder write permissions | DONE |
| POST | `/api/games/:id/match` | Manual Steam ID assignment | Pending |
| POST | `/api/games/:id/backup` | Create save backup | Pending |
| GET | `/api/games/:id/backups` | List backups for game | Pending |
| POST | `/api/games/:id/restore/:backup_id` | Restore a backup | Pending |
| GET | `/api/collections` | List all collections | Pending |
| POST | `/api/collections` | Create collection | Pending |
| PUT | `/api/collections/:id` | Update collection | Pending |
| DELETE | `/api/collections/:id` | Delete collection | Pending |
| POST | `/api/games/:id/collections` | Assign game to collections | Pending |

---

## External Integrations

### Ludusavi Manifest
- URL: https://raw.githubusercontent.com/mtkennerly/ludusavi-manifest/master/data/manifest.yaml
- Format: YAML with save path patterns per game
- Match by: Steam App ID or game name

### HowLongToBeat
- No official API - use scraping with caching
- Rate limit: 1 req/2sec, cache 30+ days
- Library: Consider `howlongtobeat` npm package patterns

---

## Notes

- Single-user for now (user state on games table)
- Can refactor to `user_games` table later for multi-user
- Games folder mounted read-write in Docker for .gamevault directories
- Images and backups stored locally in each game folder

---

## Current Implementation State (2024-12-17)

### Completed Features

| Feature | Backend | Frontend | Notes |
|---------|---------|----------|-------|
| Local image caching | DONE | DONE | Images cached in `.gamevault/` during enrichment |
| Write permission check | DONE | - | `GET /api/games/:id/storage` |
| Recent games endpoint | DONE | API ready | `GET /api/games/recent` |
| Local image serving | DONE | DONE | Falls back to CDN if no local image |

### Files Modified/Created

#### Backend (`backend/src/`)

| File | Purpose |
|------|---------|
| `local_storage.rs` | NEW - .gamevault folder management, image caching |
| `db.rs` | Added migrations for new columns, `get_recent_games()`, `update_game_local_images()` |
| `handlers.rs` | Added `serve_game_cover()`, `serve_game_background()`, `check_folder_writable()`, `get_recent_games()` |
| `main.rs` | Added routes for new endpoints |
| `models.rs` | Added new fields to Game struct |

#### Frontend (`frontend/src/`)

| File | Purpose |
|------|---------|
| `lib/api.ts` | Added `local_cover_path` to interfaces, `getCoverUrl()`, `getBackgroundUrl()` helpers |
| `components/GameCard.tsx` | Uses `getCoverUrl()` with fallback to CDN |

#### Config

| File | Change |
|------|--------|
| `docker-compose.yml` | Port changed to 9282, games mount changed to read-write |

### Database Schema (Current)

```sql
-- Games table (all columns)
CREATE TABLE games (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    folder_path TEXT NOT NULL UNIQUE,
    folder_name TEXT NOT NULL,
    title TEXT NOT NULL,
    igdb_id INTEGER,
    steam_app_id INTEGER,
    summary TEXT,
    release_date TEXT,
    cover_url TEXT,
    background_url TEXT,
    local_cover_path TEXT,           -- NEW
    local_background_path TEXT,      -- NEW
    genres TEXT,
    developers TEXT,
    publishers TEXT,
    review_score INTEGER,
    review_count INTEGER,
    review_summary TEXT,
    review_score_recent INTEGER,
    review_count_recent INTEGER,
    size_bytes INTEGER,
    match_confidence REAL,
    match_status TEXT NOT NULL DEFAULT 'pending',
    user_status TEXT DEFAULT 'unplayed',      -- NEW
    playtime_mins INTEGER DEFAULT 0,          -- NEW
    match_locked INTEGER DEFAULT 0,           -- NEW
    hltb_main_mins INTEGER,                   -- NEW
    hltb_extra_mins INTEGER,                  -- NEW
    hltb_completionist_mins INTEGER,          -- NEW
    save_path_pattern TEXT,                   -- NEW
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### How Local Image Caching Works

1. **During Enrichment**: When `POST /api/enrich` runs, after fetching Steam data:
   - `local_storage::cache_game_images()` is called
   - Checks if folder is writable via `is_folder_writable()`
   - Creates `.gamevault/` directory if needed
   - Downloads cover to `.gamevault/cover.jpg`
   - Downloads background to `.gamevault/background.jpg`
   - Updates `local_cover_path` and `local_background_path` in DB

2. **Serving Images**:
   - `GET /api/games/:id/cover` reads from `.gamevault/cover.jpg`
   - `GET /api/games/:id/background` reads from `.gamevault/background.jpg`
   - Returns 404 if local image doesn't exist

3. **Frontend Logic** (`getCoverUrl()`):
   ```typescript
   if (game.local_cover_path) {
     return `/api/games/${game.id}/cover`;  // Local
   }
   return game.cover_url;  // CDN fallback
   ```

### Pending Work

- [ ] HLTB integration (scraping with cache)
- [ ] Save game backup (Ludusavi manifest integration)
- [ ] Collections/Tags (CRUD + UI)
- [ ] Backlog filter UI (user_status dropdown)
- [ ] Manual fix match UI (Steam search modal)
- [ ] Recently Added row UI (horizontal scroll component)

### Code Review Findings (from Gemini 3 Pro Preview)

| Severity | Issue | Recommendation |
|----------|-------|----------------|
| HIGH | Blocking `std::fs` in async handlers | Use `tokio::fs` instead |
| HIGH | Regex recompilation in scanner.rs | Use `OnceLock` or `lazy_static` |
| MEDIUM | Hardcoded game mappings | Move to database |
| MEDIUM | Migration errors silently ignored | Log warnings |
| MEDIUM | Content-Type hardcoded as jpeg | Detect from file extension |
| LOW | No image validation | Validate magic bytes |
| LOW | No file size limit on downloads | Add size limit |

### Running the App

```bash
# Build and run with Docker/Podman
cd F:/Games/GameVault
podman compose up --build

# Access at http://localhost:9282
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `sqlite:///data/games.db?mode=rwc` | SQLite database path |
| `GAMES_PATH` | `/games` | Path to games folder (in container) |
| `HOST` | `127.0.0.1` | Bind address (use 0.0.0.0 for network) |
| `PORT` | `3000` | Server port |
| `API_KEY` | (none) | Optional auth for /scan and /enrich |
| `CORS_ORIGINS` | localhost:3000,5173 | Allowed CORS origins |
| `RUST_LOG` | `info` | Log level |
