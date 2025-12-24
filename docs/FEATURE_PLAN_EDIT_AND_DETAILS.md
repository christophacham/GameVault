# GameVault Feature Plan: Game Edit & Enhanced Details

**Date:** 2025-12-19
**Status:** Proposed
**Consensus Score:** 8.5/10 (Gemini 3 Pro + GPT-4.1)

---

## Overview

Two major features to transform GameVault from a passive library viewer into a robust management tool:

1. **Game Edit Menu** - "..." dropdown for editing metadata and fixing matches
2. **Enhanced Game Details** - Expanded view with full description and screenshots

---

## Feature 1: Game Menu ("..." Dropdown)

### 1.1 UI Design

```
+------------------+
|  [Game Card]  ...  |  <-- "..." button (top-right corner)
+------------------+
        |
        v
   +------------------+
   | Edit Details     |  --> Opens modal to edit metadata
   | Adjust Match     |  --> Re-link to correct Steam game
   +------------------+
```

### 1.2 Edit Details Modal

Opens a modal dialog with editable fields:

| Field | Type | Notes |
|-------|------|-------|
| Title | Text input | Game display name |
| Genres | Text input | Comma-separated list |
| Developers | Text input | Comma-separated list |
| Publishers | Text input | Comma-separated list |
| Release Date | Date picker | YYYY-MM-DD format |
| Summary | Textarea | Full game description |
| Review Score | Number (0-100) | Percentage score |

**Save Behavior:**
- Write to database (primary)
- Write to `.gamevault/metadata.json` (portable backup)
- Update `updated_at` timestamp
- Show success/error toast notification

### 1.3 Adjust Match Feature

Allows user to manually link a game to the correct Steam entry.

**Input Options:**
- Full Steam URL: `https://store.steampowered.com/app/292030/The_Witcher_3`
- App ID only: `292030`

**Help Tooltip ("?" icon):**
```
How to find the correct game:
1. Go to store.steampowered.com
2. Search for your game
3. Copy the URL from your browser
   Example: https://store.steampowered.com/app/292030/
4. Paste it here, or just enter the number (292030)
```

**Flow:**
1. User pastes URL or enters App ID
2. System extracts App ID (regex: `/app/(\d+)/`)
3. Fetch metadata from Steam API
4. Show preview of fetched data
5. User confirms -> Save to DB + metadata.json

---

## Feature 2: Enhanced Game Details Page

### 2.1 Layout Design

```
+----------------------------------------------------------+
|  [Background Image - blurred/dimmed]                      |
|                                                           |
|  +----------+  TITLE                                      |
|  |          |  Genres: Action, RPG, Open World           |
|  |  Cover   |  Developer: CD Projekt Red                 |
|  |  Image   |  Publisher: CD Projekt                     |
|  |          |  Released: May 19, 2015                    |
|  +----------+  Review: 92% - Overwhelmingly Positive     |
|                                                           |
|                HLTB: Main 51h | Extra 103h | 100% 173h   |
+----------------------------------------------------------+
|  DESCRIPTION                                              |
|                                                           |
|  You are Geralt of Rivia, a professional monster         |
|  hunter known as a Witcher. You've been contracted       |
|  to track down the Child of Prophecy...                  |
|                                                           |
+----------------------------------------------------------+
|  SCREENSHOTS                                              |
|                                                           |
|  +-------+  +-------+  +-------+  +-------+  +-------+   |
|  | img 1 |  | img 2 |  | img 3 |  | img 4 |  | img 5 |   |
|  +-------+  +-------+  +-------+  +-------+  +-------+   |
|                                                           |
+----------------------------------------------------------+
|                                          [Close] [Edit]   |
+----------------------------------------------------------+
```

### 2.2 Screenshot Management

**Source Priority:**
1. Steam CDN (primary) - High bandwidth, reliable
2. SteamGridDB (fallback) - Alternative source
3. IGDB (tertiary) - If Steam unavailable

**Storage:**
```
.gamevault/
  metadata.json
  cover.jpg
  background.jpg
  screenshots/
    screenshot_1.jpg
    screenshot_2.jpg
    screenshot_3.jpg
    screenshot_4.jpg
    screenshot_5.jpg
```

**Constraints:**
- Maximum 5 screenshots per game (disk space management)
- Download asynchronously (don't block UI)
- Skip if already cached locally

### 2.3 Steam Screenshot URLs

Steam CDN pattern:
```
https://cdn.akamai.steamstatic.com/steam/apps/{APP_ID}/ss_{HASH}.jpg
```

Fetch from Steam API response field: `screenshots[]`

---

## Technical Architecture

### 3.1 New API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| PUT | `/api/games/{id}` | Update game metadata |
| POST | `/api/games/{id}/match` | Re-match to Steam game |
| GET | `/api/games/{id}/full` | Get full details + screenshots |
| GET | `/api/games/{id}/screenshots/{n}` | Serve screenshot image |

### 3.2 Backend Changes

**db.rs:**
```rust
pub async fn update_game_metadata(
    pool: &SqlitePool,
    id: i64,
    title: Option<&str>,
    summary: Option<&str>,
    genres: Option<&str>,
    developers: Option<&str>,
    publishers: Option<&str>,
    release_date: Option<&str>,
    review_score: Option<i64>,
) -> Result<(), sqlx::Error>
```

**local_storage.rs:**
```rust
pub fn save_game_metadata(game: &Game) -> Result<(), Error>
pub async fn download_screenshots(
    client: &Client,
    game_folder: &str,
    screenshot_urls: Vec<String>,
) -> Result<Vec<String>, Error>
pub fn get_screenshot_paths(game_folder: &str) -> Vec<PathBuf>
```

**handlers.rs:**
```rust
pub async fn update_game(...)        // PUT /games/{id}
pub async fn rematch_game(...)       // POST /games/{id}/match
pub async fn get_game_full(...)      // GET /games/{id}/full
pub async fn serve_screenshot(...)   // GET /games/{id}/screenshots/{n}
```

### 3.3 Frontend Components

**New Components:**
- `GameMenu.tsx` - "..." dropdown menu
- `EditModal.tsx` - Edit metadata form
- `AdjustMatchModal.tsx` - Re-match with help tooltip
- `GameDetailView.tsx` - Full game details page
- `ScreenshotGallery.tsx` - Screenshot display with lightbox

**Modified Components:**
- `GameCard.tsx` - Add "..." menu button
- `page.tsx` - Handle game click to open details

### 3.4 Data Flow

```
User Action                Backend                    Storage
-----------                -------                    -------

Click "..."         -->    (no request)

Edit Details        -->    PUT /games/{id}     -->    DB + metadata.json

Adjust Match        -->    POST /games/{id}/match
                           - Parse URL/ID
                           - Fetch Steam API
                           - Return preview

Confirm Match       -->    PUT /games/{id}     -->    DB + metadata.json
                           - Download images   -->    .gamevault/

Click Game Card     -->    GET /games/{id}/full
                           - Return all data
                           - Include screenshot URLs

Load Screenshot     -->    GET /games/{id}/screenshots/1
                           - Serve from .gamevault/screenshots/
```

---

## Import/Export Compatibility

### 4.1 Updated metadata.json Schema

```json
{
  "schema_version": 2,
  "title": "The Witcher 3",
  "steam_app_id": 292030,
  "summary": "...",
  "genres": ["Action", "RPG"],
  "developers": ["CD Projekt Red"],
  "publishers": ["CD Projekt"],
  "release_date": "2015-05-19",
  "review_score": 92,
  "review_summary": "Overwhelmingly Positive",
  "hltb": {
    "main_mins": 3060,
    "extra_mins": 6180,
    "completionist_mins": 10380
  },
  "screenshots": [
    "screenshot_1.jpg",
    "screenshot_2.jpg",
    "screenshot_3.jpg"
  ],
  "exported_at": "2025-12-19T10:30:00Z",
  "manually_edited": true
}
```

### 4.2 Export Changes

- Include `screenshots/` folder in export
- Add `schema_version` field for future compatibility
- Add `manually_edited` flag to track user modifications

### 4.3 Import Changes

- Check `schema_version` and handle migration
- Restore screenshots from folder if present
- Preserve `manually_edited` flag

---

## Implementation Phases

### Phase 1: Edit Menu (Priority: High)

**Backend:**
1. Add `update_game_metadata()` to db.rs
2. Add `save_game_metadata()` to local_storage.rs (dual-write)
3. Add `PUT /games/{id}` endpoint to handlers.rs
4. Add route to main.rs

**Frontend:**
1. Create `GameMenu.tsx` component (dropdown)
2. Create `EditModal.tsx` component (form)
3. Add `updateGame()` to api.ts
4. Modify `GameCard.tsx` to include menu

**Estimated Changes:** ~400 lines

### Phase 2: Adjust Match (Priority: High)

**Backend:**
1. Add URL parser utility (extract Steam App ID)
2. Add `POST /games/{id}/match` endpoint
3. Return preview data before confirming

**Frontend:**
1. Create `AdjustMatchModal.tsx` component
2. Add help tooltip with instructions
3. Show preview before save
4. Add `rematchGame()` to api.ts

**Estimated Changes:** ~300 lines

### Phase 3: Enhanced Details (Priority: Medium)

**Backend:**
1. Add `download_screenshots()` to local_storage.rs
2. Add `GET /games/{id}/full` endpoint
3. Add `GET /games/{id}/screenshots/{n}` endpoint
4. Fetch screenshots during enrich

**Frontend:**
1. Create `GameDetailView.tsx` component
2. Create `ScreenshotGallery.tsx` component
3. Handle game card click to open details
4. Add `getGameFull()` to api.ts

**Estimated Changes:** ~500 lines

### Phase 4: Integration (Priority: Medium)

1. Update export to include screenshots folder
2. Update import to restore screenshots
3. Add schema versioning to metadata.json
4. Test full round-trip (edit -> export -> import)

**Estimated Changes:** ~200 lines

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| DB/JSON desync | Always write to both atomically; JSON is secondary |
| Large screenshots | Limit to 5 per game; compress on download |
| Steam API rate limits | Use existing rate limiting (500ms between calls) |
| Race conditions | File locking when writing metadata.json |
| URL parsing errors | Validate input; show clear error messages |

---

## Success Criteria

- [ ] User can edit any game's metadata via "..." menu
- [ ] Edits are saved to both DB and metadata.json
- [ ] User can fix incorrect matches with Steam URL
- [ ] Help tooltip clearly explains how to find URLs
- [ ] Clicking a game shows full details + screenshots
- [ ] Screenshots are cached locally in .gamevault/
- [ ] Import/export works with all new data
- [ ] No breaking changes to existing functionality

---

## Open Questions

1. Should we support IGDB URLs in addition to Steam?
2. Should screenshots be downloadable/exportable as a zip?
3. Should there be a "Reset to Original" option to undo manual edits?
4. Should we add keyboard shortcuts for the edit modal?

---

## Appendix: Steam URL Patterns

```
# Standard store page
https://store.steampowered.com/app/292030/The_Witcher_3_Wild_Hunt/

# With query params
https://store.steampowered.com/app/292030/?curator_clanid=...

# Regional variants
https://store.steampowered.com/app/292030/

# Short form
store.steampowered.com/app/292030

# Regex pattern
/(?:store\.steampowered\.com\/app\/|^)(\d+)/
```
