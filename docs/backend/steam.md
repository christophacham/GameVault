---
sidebar_position: 5
---

# steam.rs

Steam Store API client for fetching game metadata.

## API Endpoints

| Endpoint | Purpose |
|----------|---------|
| `store.steampowered.com/api/storesearch` | Search games by title |
| `store.steampowered.com/api/appdetails` | Get game details |
| `store.steampowered.com/appreviews` | Get review data |

## Search Function

```rust
pub async fn search_steam_app(
    client: &reqwest::Client,
    title: &str,
) -> Option<(i64, f64)> {
    // Check known mappings first
    if let Some(app_id) = KNOWN_MAPPINGS.get(title.to_lowercase().as_str()) {
        return Some((*app_id, 1.0));
    }

    // Search Steam
    let url = format!(
        "https://store.steampowered.com/api/storesearch/?term={}&cc=us&l=en",
        urlencoding::encode(title)
    );

    let response = client.get(&url)
        .send()
        .await
        .ok()?
        .json::<SteamSearchResponse>()
        .await
        .ok()?;

    // Find best match using fuzzy matching
    let mut best_match: Option<(i64, f64)> = None;

    for item in response.items.unwrap_or_default() {
        let similarity = strsim::jaro_winkler(
            &title.to_lowercase(),
            &item.name.to_lowercase()
        );

        if similarity > best_match.map(|(_, s)| s).unwrap_or(0.0) {
            best_match = Some((item.id, similarity));
        }
    }

    // Only return if similarity is above threshold
    best_match.filter(|(_, sim)| *sim >= 0.6)
}
```

## Fetch Details

```rust
pub async fn fetch_steam_details(
    client: &reqwest::Client,
    app_id: i64,
) -> Option<SteamGameDetails> {
    let url = format!(
        "https://store.steampowered.com/api/appdetails?appids={}",
        app_id
    );

    let response = client.get(&url)
        .send()
        .await
        .ok()?
        .json::<SteamAppDetailsResponse>()
        .await
        .ok()?;

    let app_data = response.apps
        .get(&app_id.to_string())?
        .data
        .as_ref()?;

    Some(SteamGameDetails {
        name: app_data.name.clone(),
        description: app_data.short_description.clone(),
        header_image: app_data.header_image.clone(),
        background: app_data.background.clone(),
        genres: app_data.genres.as_ref().map(|g|
            g.iter().map(|genre| genre.description.clone()).collect()
        ),
        developers: app_data.developers.clone(),
        publishers: app_data.publishers.clone(),
        release_date: app_data.release_date
            .as_ref()
            .and_then(|r| r.date.clone()),
    })
}
```

## Fetch Reviews

```rust
pub async fn fetch_steam_reviews(
    client: &reqwest::Client,
    app_id: i64,
) -> Option<SteamReviews> {
    let url = format!(
        "https://store.steampowered.com/appreviews/{}?json=1&language=all&purchase_type=all",
        app_id
    );

    let response = client.get(&url)
        .send()
        .await
        .ok()?
        .json::<SteamReviewsResponse>()
        .await
        .ok()?;

    let summary = response.query_summary?;
    let total = summary.total_positive.unwrap_or(0) + summary.total_negative.unwrap_or(0);

    if total == 0 {
        return None;
    }

    let score = (summary.total_positive.unwrap_or(0) * 100) / total;

    Some(SteamReviews {
        score: score as i64,
        count: summary.total_reviews.unwrap_or(0),
        summary: summary.review_score_desc.unwrap_or_default(),
    })
}
```

## Data Types

### SteamGameDetails

```rust
pub struct SteamGameDetails {
    pub name: String,
    pub description: Option<String>,
    pub header_image: Option<String>,
    pub background: Option<String>,
    pub genres: Option<Vec<String>>,
    pub developers: Option<Vec<String>>,
    pub publishers: Option<Vec<String>>,
    pub release_date: Option<String>,
}
```

### SteamReviews

```rust
pub struct SteamReviews {
    pub score: i64,       // 0-100
    pub count: i64,       // Total reviews
    pub summary: String,  // "Very Positive", etc.
}
```

## Known Mappings

Pre-configured title-to-App-ID mappings for common games with ambiguous names:

```rust
lazy_static! {
    static ref KNOWN_MAPPINGS: HashMap<&'static str, i64> = {
        let mut m = HashMap::new();
        m.insert("doom", 379720);          // DOOM (2016)
        m.insert("prey", 480490);          // Prey (2017)
        m.insert("control", 870780);       // Control
        m.insert("inside", 304430);        // INSIDE
        m.insert("soma", 282140);          // SOMA
        // ... 200+ entries
        m
    };
}
```

## Fuzzy Matching

Uses Jaro-Winkler similarity from `strsim` crate:

```rust
let similarity = strsim::jaro_winkler(&title, &steam_name);

// Matching thresholds:
// > 0.85 - Auto-match (high confidence)
// 0.60-0.85 - Match but flag for review
// < 0.60 - No match
```

## Rate Limiting

```rust
const STEAM_API_RATE_LIMIT_MS: u64 = 500;

// Between each API call
tokio::time::sleep(Duration::from_millis(STEAM_API_RATE_LIMIT_MS)).await;
```

## Error Handling

All functions return `Option<T>`:

```rust
// If any step fails, returns None
let details = steam::fetch_steam_details(&client, app_id).await;

match details {
    Some(d) => { /* Update database */ }
    None => {
        tracing::warn!("Failed to fetch Steam details for {}", app_id);
        failed += 1;
    }
}
```
