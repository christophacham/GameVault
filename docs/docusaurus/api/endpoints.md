---
sidebar_position: 2
---

# API Endpoints

Complete reference for all GameVault API endpoints.

## Games

### List All Games

```http
GET /api/games
```

Returns all games in the library.

**Response:**

```json
{
  "success": true,
  "data": [
    {
      "id": 1,
      "title": "The Witcher 3: Wild Hunt",
      "cover_url": "https://steamcdn-a.akamaihd.net/...",
      "local_cover_path": ".gamevault/cover.jpg",
      "genres": ["RPG", "Open World"],
      "review_score": 95,
      "review_summary": "Overwhelmingly Positive",
      "match_status": "matched",
      "user_status": null,
      "hltb_main_mins": 3120
    }
  ],
  "error": null
}
```

### Get Game by ID

```http
GET /api/games/:id
```

Returns detailed information for a single game.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `id` | number | Game ID |

**Response:**

```json
{
  "success": true,
  "data": {
    "id": 1,
    "folder_path": "D:\\Games\\The Witcher 3",
    "folder_name": "The Witcher 3",
    "title": "The Witcher 3: Wild Hunt",
    "steam_app_id": 292030,
    "summary": "You are Geralt of Rivia...",
    "release_date": "2015-05-18",
    "cover_url": "https://...",
    "background_url": "https://...",
    "local_cover_path": ".gamevault/cover.jpg",
    "local_background_path": ".gamevault/background.jpg",
    "genres": "[\"RPG\",\"Open World\"]",
    "developers": "[\"CD PROJEKT RED\"]",
    "publishers": "[\"CD PROJEKT RED\"]",
    "review_score": 95,
    "review_count": 500000,
    "review_summary": "Overwhelmingly Positive",
    "size_bytes": 50000000000,
    "match_confidence": 0.95,
    "match_status": "matched"
  },
  "error": null
}
```

:::note
The `genres`, `developers`, and `publishers` fields are returned as JSON-encoded strings from the database. Parse them with `JSON.parse()` in your client code.
:::

### Search Games

```http
GET /api/games/search?q=:query
```

Search games by title.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `q` | string | Search query (1-200 characters) |

**Response:** Same as List All Games, filtered by query.

### Get Recent Games

```http
GET /api/games/recent
```

Returns recently added/updated games.

### Update Game

```http
PUT /api/games/:id
```

Update game metadata manually.

**Request Body:**

```json
{
  "title": "New Title",
  "summary": "Game description",
  "genres": ["Action", "RPG"],
  "developers": ["Studio Name"],
  "publishers": ["Publisher Name"],
  "release_date": "2023-01-15",
  "review_score": 85
}
```

All fields are optional.

**Response:** Updated game object.

### Rematch Game (Preview)

```http
POST /api/games/:id/match
```

Preview matching a game to a different Steam entry.

**Request Body:**

```json
{
  "steam_input": "292030"
}
```

Accepts Steam App ID or URL.

**Response:**

```json
{
  "success": true,
  "data": {
    "steam_app_id": 292030,
    "title": "The Witcher 3: Wild Hunt",
    "summary": "...",
    "genres": ["RPG"],
    "cover_url": "https://..."
  },
  "error": null
}
```

### Rematch Game (Confirm)

```http
POST /api/games/:id/match/confirm
```

Confirm and apply a rematch.

**Request Body:** Same as preview.

**Response:** Updated game object.

### Serve Cover Image

```http
GET /api/games/:id/cover
```

Returns the cached cover image for a game.

### Serve Background Image

```http
GET /api/games/:id/background
```

Returns the cached background image for a game.

---

## Operations

### Scan Games

```http
POST /api/scan
```

Scan the game library folder for games.

**Response:**

```json
{
  "success": true,
  "data": {
    "total_found": 150,
    "added_or_updated": 5
  },
  "error": null
}
```

### Enrich Games

```http
POST /api/enrich
```

Fetch Steam metadata for unmatched games.

**Response:**

```json
{
  "success": true,
  "data": {
    "enriched": 10,
    "failed": 2,
    "remaining": 5,
    "total": 17
  },
  "error": null
}
```

### Export Metadata

```http
POST /api/export
```

Export metadata to `.gamevault/metadata.json` in each game folder.

**Response:**

```json
{
  "success": true,
  "data": {
    "exported": 145,
    "skipped": 5,
    "failed": 0,
    "total": 150
  },
  "error": null
}
```

### Import Metadata

```http
POST /api/import
```

Import metadata from `.gamevault/metadata.json` files.

**Response:**

```json
{
  "success": true,
  "data": {
    "imported": 50,
    "skipped": 95,
    "not_found": 5,
    "failed": 0,
    "total": 150
  },
  "error": null
}
```

---

## Configuration

### Get Configuration

```http
GET /api/config
```

**Response:**

```json
{
  "success": true,
  "data": {
    "paths": {
      "game_library": "D:\\Games",
      "cache": "./cache",
      "game_library_exists": true,
      "cache_exists": true
    },
    "server": {
      "port": 3000,
      "auto_open_browser": true,
      "bind_address": "127.0.0.1"
    }
  },
  "error": null
}
```

### Update Configuration

```http
PUT /api/config
```

**Request Body:**

```json
{
  "game_library": "D:\\Games",
  "cache": "./cache",
  "port": 3000,
  "auto_open_browser": true
}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "success": true,
    "restart_required": false,
    "message": "Configuration saved successfully."
  },
  "error": null
}
```

### Get Configuration Status

```http
GET /api/config/status
```

Check if initial setup is needed.

**Response:**

```json
{
  "success": true,
  "data": {
    "needs_setup": true,
    "game_library_configured": false,
    "game_library_path": ""
  },
  "error": null
}
```

---

## System

### Health Check

```http
GET /api/health
```

**Response:**

```json
{
  "success": true,
  "data": "OK",
  "error": null
}
```

### Get Statistics

```http
GET /api/stats
```

**Response:**

```json
{
  "success": true,
  "data": {
    "total_games": 150,
    "matched_games": 145,
    "pending_games": 5,
    "enriched_games": 140
  },
  "error": null
}
```

### Shutdown Server

```http
POST /api/shutdown
```

Gracefully shuts down GameVault.

**Response:**

```json
{
  "success": true,
  "data": "Shutting down...",
  "error": null
}
```

### Restart Server

```http
POST /api/restart
```

Restarts GameVault (spawns new process, exits current).

**Response:**

```json
{
  "success": true,
  "data": "Restarting...",
  "error": null
}
```
