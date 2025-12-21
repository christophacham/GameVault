---
sidebar_position: 1
---

# API Overview

GameVault exposes a RESTful API for programmatic access to your game library.

## Base URL

```
http://localhost:3000/api
```

## Response Format

All responses follow a consistent JSON structure:

```json
{
  "success": true,
  "data": { ... },
  "error": null
}
```

### Success Response

```json
{
  "success": true,
  "data": {
    "id": 1,
    "title": "The Witcher 3"
  },
  "error": null
}
```

### Error Response

```json
{
  "success": false,
  "data": null,
  "error": "Game not found"
}
```

## Authentication

By default, no authentication is required. To enable API key authentication:

```bash
set API_KEY=your-secret-key
```

Then include in requests:

```http
Authorization: Bearer your-secret-key
```

### Protected Endpoints

These endpoints require authentication when `API_KEY` is set:

- `POST /api/scan`
- `POST /api/enrich`
- `POST /api/export`
- `POST /api/import`
- `PUT /api/games/:id`
- `POST /api/games/:id/match`
- `POST /api/games/:id/match/confirm`

### Unprotected Endpoints

These are always accessible:

- `GET /api/health`
- `GET /api/games`
- `GET /api/games/:id`
- `GET /api/stats`
- `GET /api/config`
- `PUT /api/config`

## Content Type

All requests and responses use JSON:

```http
Content-Type: application/json
```

## CORS

CORS is configured for localhost access by default:

- `http://localhost:3000`
- `http://127.0.0.1:3000`
- `http://localhost:5173` (Vite dev server)

To add custom origins:

```bash
set CORS_ORIGINS=http://example.com,http://app.local
```

## Rate Limiting

No rate limiting is implemented in GameVault. However, Steam API calls during enrichment are rate-limited to prevent being blocked.

## Endpoint Categories

### Games

- [GET /api/games](#) - List all games
- [GET /api/games/:id](#) - Get game details
- [PUT /api/games/:id](#) - Update game metadata
- [GET /api/games/search](#) - Search games
- [GET /api/games/recent](#) - Get recent games

### Operations

- [POST /api/scan](#) - Scan for games
- [POST /api/enrich](#) - Enrich with metadata
- [POST /api/export](#) - Export metadata
- [POST /api/import](#) - Import metadata

### Configuration

- [GET /api/config](#) - Get configuration
- [PUT /api/config](#) - Update configuration
- [GET /api/config/status](#) - Check setup status

### System

- [GET /api/health](#) - Health check
- [GET /api/stats](#) - Library statistics
- [POST /api/shutdown](#) - Shutdown server
- [POST /api/restart](#) - Restart server

## Error Codes

| Status | Meaning |
|--------|---------|
| 200 | Success |
| 400 | Bad Request - Invalid input |
| 401 | Unauthorized - Missing/invalid API key |
| 404 | Not Found - Resource doesn't exist |
| 500 | Server Error - Internal error |

## Quick Examples

### List Games

```bash
curl http://localhost:3000/api/games
```

### Get Game Details

```bash
curl http://localhost:3000/api/games/1
```

### Scan Library

```bash
curl -X POST http://localhost:3000/api/scan
```

### Update Configuration

```bash
curl -X PUT http://localhost:3000/api/config \
  -H "Content-Type: application/json" \
  -d '{"game_library":"D:\\Games","cache":"./cache","port":3000,"auto_open_browser":true}'
```
