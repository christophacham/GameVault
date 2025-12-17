# GameVault Enrichment Architecture Analysis

> Generated: 2025-12-17
> Analysis: Claude Opus 4.5 + Gemini Pro 3 Preview (thinkdeep + challenge)

---

## Current State

| Metric | Value |
|--------|-------|
| Total games detected | 180 |
| Matched games | 0 |
| Pending enrichment | 180 |
| Data source | Steam API only |

## Current Implementation Overview

### Scanner (`scanner.rs`)
- Regex-based title cleanup removing `[FitGirl]`, `[DODI]`, version numbers, etc.
- Extracts clean game titles from folder names
- Estimates folder sizes

### Steam Integration (`steam.rs`)
- ~60 hardcoded known mappings (popular games)
- Steam Search API for unknown games
- Jaro-Winkler similarity matching:
  - Threshold 0.85 for known mappings
  - Threshold 0.60 for Steam search results
- Fetches game details + reviews
- 500ms rate limiting between requests

### Enrichment Handler (`handlers.rs`)
- Processes 20 games per API request
- Sequential processing with rate limits
- Updates DB with cover, genres, reviews, confidence score

---

## Identified Limitations

| Issue | Impact | Severity |
|-------|--------|----------|
| Single data source (Steam only) | Misses Epic exclusives, GOG games, ~20% coverage gap | High |
| Fixed batch size (20) | Multiple API calls needed for full library | Low |
| Hardcoded mappings | Can't add new mappings without code change | Medium |
| No progress tracking | User can't see enrichment progress | Medium |
| No manual override | Can't fix mismatches | High |
| Fixed confidence threshold | False positives on short titles, false negatives on subtitled games | Medium |

---

## Recommended Improvements

### Priority 1: Immediate (Low Effort, High Impact)

#### 1.1 Enhanced Title Preprocessing
```rust
// Before matching, normalize:
fn normalize_title(title: &str) -> String {
    let mut t = title.to_lowercase();

    // Remove common prefixes
    t = t.trim_start_matches("the ");

    // Normalize roman numerals
    t = t.replace(" iii", " 3").replace(" ii", " 2").replace(" iv", " 4");

    // Normalize editions
    t = t.replace("goty", "").replace("definitive edition", "")
         .replace("complete edition", "").replace("enhanced edition", "");

    // Handle special cases
    t = t.replace("c&c", "command and conquer");

    t.trim().to_string()
}
```

#### 1.2 Move Known Mappings to Database
```sql
CREATE TABLE manual_mappings (
    id INTEGER PRIMARY KEY,
    clean_title TEXT NOT NULL UNIQUE,
    steam_app_id INTEGER,
    igdb_id INTEGER,
    source TEXT DEFAULT 'user',  -- 'user', 'community', 'auto'
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

Priority order: DB mapping > Steam search

#### 1.3 Confidence Tiers with Dynamic Thresholds
```rust
enum MatchConfidence {
    High,      // >0.9 - Auto-enrich, mark as "matched"
    Medium,    // 0.7-0.9 - Auto-enrich, mark as "needs_review"
    Low,       // <0.7 - Skip, mark as "unmatched"
}

// Dynamic threshold based on title length
fn get_threshold(title: &str) -> f64 {
    if title.len() < 10 { 0.85 } else { 0.70 }
}
```

### Priority 2: Medium-Term

#### 2.1 Add IGDB as Parallel Source (Not Fallback)
```
Query BOTH Steam + IGDB in parallel
    |
    v
Compare confidence scores
    |
    v
Pick highest confidence match
```

IGDB advantages:
- Better coverage for Epic exclusives (Alan Wake 2)
- Better for GOG classics
- Better for indie games
- Free API with Twitch authentication

#### 2.2 Manual Match Override API
```
POST /api/games/:id/match
Body: { "steam_app_id": 1091500 }
     // OR
Body: { "igdb_id": 12345 }

Response: { "success": true, "game": { ... enriched data ... } }
```

#### 2.3 Background Processing with Polling
```
POST /api/enrich/start
Response: { "job_id": "abc123", "total": 180 }

GET /api/enrich/status/abc123
Response: {
    "status": "running",
    "current": "Cyberpunk 2077",
    "progress": 45,
    "total": 180,
    "enriched": 40,
    "failed": 5
}
```

Simple polling (every 2s) is preferable to SSE complexity for occasional use.

### Priority 3: Long-Term / Nice-to-Have

#### 3.1 Fuzzy Match Candidates UI
- Show top 3-5 candidates when confidence is low
- Let user pick correct match from dropdown

#### 3.2 Community Mapping Database
- Shared mappings across users
- Crowdsourced corrections
- API: `GET /api/community/mappings?title=...`

#### 3.3 Local Image Cache
- Store cover images locally in `/data/covers/`
- Avoid broken links if Steam changes URLs
- Reduces external API dependency

---

## Algorithm Analysis

### Current: Jaro-Winkler

**Pros:**
- Good for typos and minor variations
- Fast computation
- Works well for similar-length strings

**Cons:**
- Favors prefix matches (problematic for subtitles)
- Struggles with word reordering
- Fixed threshold is brittle

### Recommended: Hybrid Scoring
```rust
fn hybrid_score(a: &str, b: &str) -> f64 {
    let jw = jaro_winkler(a, b);
    let jaccard = token_jaccard(a, b);  // Word-level overlap

    0.6 * jw + 0.4 * jaccard
}

fn token_jaccard(a: &str, b: &str) -> f64 {
    let words_a: HashSet<_> = a.split_whitespace().collect();
    let words_b: HashSet<_> = b.split_whitespace().collect();

    let intersection = words_a.intersection(&words_b).count() as f64;
    let union = words_a.union(&words_b).count() as f64;

    if union == 0.0 { 0.0 } else { intersection / union }
}
```

This catches both:
- Similar spellings (Jaro-Winkler)
- Matching word sets (Jaccard)

---

## Trade-offs Summary

| Approach | Pros | Cons |
|----------|------|------|
| Steam-only | Simple, one API, good for mainstream | Misses ~20% of games |
| Steam + IGDB | Better coverage, parallel is fast | Two APIs, more complexity |
| Full auto-match | Fast, no user intervention | More false positives |
| Human-in-loop | Most accurate | Slower, needs UI work |
| Fixed thresholds | Simple to implement | Fails edge cases |
| Dynamic thresholds | Handles edge cases | More complex logic |

---

## Implementation Roadmap

```
Phase 1 (Now)
├── Run current enrichment to baseline match rate
├── Add 50+ known mappings for user's library
└── Improve title normalization

Phase 2 (Short-term)
├── Move mappings to database
├── Add manual match endpoint
└── Implement confidence tiers

Phase 3 (Medium-term)
├── Add IGDB parallel search
├── Background job with polling
└── Match candidates UI

Phase 4 (Long-term)
├── Community mapping database
├── Local image cache
└── Advanced fuzzy matching
```

---

## API Endpoints Summary

### Current
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/health` | Health check |
| GET | `/api/games` | List all games |
| GET | `/api/games/:id` | Get game details |
| GET | `/api/games/search?q=` | Search games |
| POST | `/api/scan` | Scan games directory |
| POST | `/api/enrich` | Enrich pending games (20 at a time) |
| GET | `/api/stats` | Get statistics |

### Proposed Additions
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/games/:id/match` | Manual match override |
| POST | `/api/enrich/start` | Start background enrichment |
| GET | `/api/enrich/status/:job_id` | Get enrichment progress |
| GET | `/api/mappings` | List manual mappings |
| POST | `/api/mappings` | Add manual mapping |

---

## References

- Steam Store API: `https://store.steampowered.com/api`
- Steam Search: `https://steamcommunity.com/actions/SearchApps`
- IGDB API: `https://api.igdb.com/v4/` (requires Twitch auth)
- Jaro-Winkler: `strsim` crate
- Current thresholds: 0.85 (known), 0.60 (search)
