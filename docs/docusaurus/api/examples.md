---
sidebar_position: 3
---

# API Examples

Practical examples of using the GameVault API.

## cURL Examples

### List All Games

```bash
curl http://localhost:3000/api/games
```

### Get Specific Game

```bash
curl http://localhost:3000/api/games/1
```

### Search for Games

```bash
curl "http://localhost:3000/api/games/search?q=witcher"
```

### Scan Library

```bash
curl -X POST http://localhost:3000/api/scan
```

### Enrich Games

```bash
curl -X POST http://localhost:3000/api/enrich
```

### Update Game Title

```bash
curl -X PUT http://localhost:3000/api/games/1 \
  -H "Content-Type: application/json" \
  -d '{"title":"The Witcher 3: Wild Hunt - GOTY Edition"}'
```

### Update Configuration

```bash
curl -X PUT http://localhost:3000/api/config \
  -H "Content-Type: application/json" \
  -d '{
    "game_library": "D:\\Games",
    "cache": "./cache",
    "port": 3000,
    "auto_open_browser": true
  }'
```

### Check Health

```bash
curl http://localhost:3000/api/health
```

### Shutdown Server

```bash
curl -X POST http://localhost:3000/api/shutdown
```

---

## PowerShell Examples

### List All Games

```powershell
Invoke-RestMethod -Uri "http://localhost:3000/api/games"
```

### Get Game by ID

```powershell
$game = Invoke-RestMethod -Uri "http://localhost:3000/api/games/1"
Write-Host "Title: $($game.data.title)"
Write-Host "Score: $($game.data.review_score)"
```

### Scan and Enrich

```powershell
# Scan for new games
$scan = Invoke-RestMethod -Uri "http://localhost:3000/api/scan" -Method POST
Write-Host "Found $($scan.data.total_found) games"

# Enrich with metadata
$enrich = Invoke-RestMethod -Uri "http://localhost:3000/api/enrich" -Method POST
Write-Host "Enriched $($enrich.data.enriched) games"
```

### Update Configuration

```powershell
$config = @{
    game_library = "D:\Games"
    cache = "./cache"
    port = 3000
    auto_open_browser = $true
} | ConvertTo-Json

Invoke-RestMethod -Uri "http://localhost:3000/api/config" `
  -Method PUT `
  -Body $config `
  -ContentType "application/json"
```

### Export All Games to CSV

```powershell
$games = Invoke-RestMethod -Uri "http://localhost:3000/api/games"
$games.data | Select-Object id, title, review_score | Export-Csv -Path "games.csv"
```

---

## Python Examples

### Basic Setup

```python
import requests

BASE_URL = "http://localhost:3000/api"

def api_get(endpoint):
    response = requests.get(f"{BASE_URL}{endpoint}")
    return response.json()

def api_post(endpoint, data=None):
    response = requests.post(f"{BASE_URL}{endpoint}", json=data)
    return response.json()
```

### List and Display Games

```python
import requests

response = requests.get("http://localhost:3000/api/games")
data = response.json()

if data["success"]:
    for game in data["data"]:
        score = game.get("review_score", "N/A")
        print(f"{game['title']} - Score: {score}")
```

### Scan and Enrich Library

```python
import requests
import time

# Scan for games
scan = requests.post("http://localhost:3000/api/scan").json()
print(f"Found {scan['data']['total_found']} games")

# Wait a moment
time.sleep(1)

# Enrich with metadata
enrich = requests.post("http://localhost:3000/api/enrich").json()
print(f"Enriched {enrich['data']['enriched']} games")
print(f"Failed: {enrich['data']['failed']}")
```

### Update Game Metadata

```python
import requests

game_id = 1
update_data = {
    "title": "The Witcher 3: Wild Hunt - Complete Edition",
    "genres": ["RPG", "Open World", "Fantasy"]
}

response = requests.put(
    f"http://localhost:3000/api/games/{game_id}",
    json=update_data
)

result = response.json()
if result["success"]:
    print(f"Updated: {result['data']['title']}")
```

### Monitor Library Stats

```python
import requests
import time

def get_stats():
    response = requests.get("http://localhost:3000/api/stats")
    return response.json()["data"]

while True:
    stats = get_stats()
    print(f"Total: {stats['total_games']}, "
          f"Matched: {stats['matched_games']}, "
          f"Pending: {stats['pending_games']}")
    time.sleep(5)
```

---

## JavaScript/Fetch Examples

### List Games

```javascript
async function listGames() {
  const response = await fetch('http://localhost:3000/api/games');
  const data = await response.json();

  if (data.success) {
    data.data.forEach(game => {
      console.log(`${game.title} - ${game.review_score || 'N/A'}`);
    });
  }
}
```

### Update Configuration

```javascript
async function updateConfig(gamePath) {
  const response = await fetch('http://localhost:3000/api/config', {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      game_library: gamePath,
      cache: './cache',
      port: 3000,
      auto_open_browser: true
    })
  });

  const result = await response.json();
  if (result.data.restart_required) {
    console.log('Restart required for changes to take effect');
  }
}
```

### Search Games

```javascript
async function searchGames(query) {
  const response = await fetch(
    `http://localhost:3000/api/games/search?q=${encodeURIComponent(query)}`
  );
  const data = await response.json();
  return data.data;
}

// Usage
const results = await searchGames('witcher');
console.log(`Found ${results.length} games`);
```

---

## Complete Automation Script

### PowerShell: Full Library Refresh

```powershell
# GameVault Library Refresh Script

$baseUrl = "http://localhost:3000/api"

Write-Host "=== GameVault Library Refresh ===" -ForegroundColor Cyan

# Check health
$health = Invoke-RestMethod -Uri "$baseUrl/health"
if ($health.data -ne "OK") {
    Write-Error "GameVault is not running!"
    exit 1
}
Write-Host "[OK] GameVault is running" -ForegroundColor Green

# Scan for games
Write-Host "`nScanning library..."
$scan = Invoke-RestMethod -Uri "$baseUrl/scan" -Method POST
Write-Host "Found $($scan.data.total_found) games, $($scan.data.added_or_updated) new/updated"

# Enrich games
Write-Host "`nEnriching metadata..."
$enrich = Invoke-RestMethod -Uri "$baseUrl/enrich" -Method POST
Write-Host "Enriched: $($enrich.data.enriched)"
Write-Host "Failed: $($enrich.data.failed)"
Write-Host "Remaining: $($enrich.data.remaining)"

# Export metadata
Write-Host "`nExporting metadata..."
$export = Invoke-RestMethod -Uri "$baseUrl/export" -Method POST
Write-Host "Exported: $($export.data.exported)"
Write-Host "Skipped: $($export.data.skipped)"

# Final stats
$stats = Invoke-RestMethod -Uri "$baseUrl/stats"
Write-Host "`n=== Library Statistics ===" -ForegroundColor Cyan
Write-Host "Total Games: $($stats.data.total_games)"
Write-Host "Matched: $($stats.data.matched_games)"
Write-Host "Enriched: $($stats.data.enriched_games)"
Write-Host "Pending: $($stats.data.pending_games)"

Write-Host "`nRefresh complete!" -ForegroundColor Green
```
