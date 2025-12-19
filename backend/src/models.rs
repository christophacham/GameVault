use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Game {
    pub id: i64,
    /// SECURITY: Hidden from API responses - contains local filesystem path
    #[serde(skip_serializing)]
    pub folder_path: String,
    /// SECURITY: Hidden from API responses - may reveal folder naming patterns
    #[serde(skip_serializing)]
    pub folder_name: String,
    pub title: String,

    // IGDB/Steam IDs
    pub igdb_id: Option<i64>,
    pub steam_app_id: Option<i64>,

    // Basic info
    pub summary: Option<String>,
    pub release_date: Option<String>,

    // Images (CDN URLs - fallback)
    pub cover_url: Option<String>,
    pub background_url: Option<String>,

    // Local cached images (in .gamevault/ folder)
    pub local_cover_path: Option<String>,
    pub local_background_path: Option<String>,

    // Metadata (JSON strings)
    pub genres: Option<String>,
    pub developers: Option<String>,
    pub publishers: Option<String>,

    // Reviews
    pub review_score: Option<i64>,
    pub review_count: Option<i64>,
    pub review_summary: Option<String>,

    // Recent reviews (last 30 days)
    pub review_score_recent: Option<i64>,
    pub review_count_recent: Option<i64>,

    // Technical
    pub size_bytes: Option<i64>,

    // Matching
    pub match_confidence: Option<f64>,
    pub match_status: String,

    // User state
    pub user_status: Option<String>,
    pub playtime_mins: Option<i64>,
    pub match_locked: Option<i64>,

    // HLTB data (HowLongToBeat)
    pub hltb_main_mins: Option<i64>,
    pub hltb_extra_mins: Option<i64>,
    pub hltb_completionist_mins: Option<i64>,

    // Save backup pattern
    pub save_path_pattern: Option<String>,

    // Timestamps
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSummary {
    pub id: i64,
    pub title: String,
    pub cover_url: Option<String>,
    pub local_cover_path: Option<String>,
    pub genres: Option<Vec<String>>,
    pub review_score: Option<i64>,
    pub review_summary: Option<String>,
    pub match_status: String,
    pub user_status: Option<String>,
    pub hltb_main_mins: Option<i64>,
}

impl From<Game> for GameSummary {
    fn from(g: Game) -> Self {
        let genres = g.genres.and_then(|s| serde_json::from_str(&s).ok());
        GameSummary {
            id: g.id,
            title: g.title,
            cover_url: g.cover_url,
            local_cover_path: g.local_cover_path,
            genres,
            review_score: g.review_score,
            review_summary: g.review_summary,
            match_status: g.match_status,
            user_status: g.user_status,
            hltb_main_mins: g.hltb_main_mins,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        ApiResponse {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        ApiResponse {
            success: false,
            data: None,
            error: Some(msg.into()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Stats {
    pub total_games: i64,
    pub matched_games: i64,
    pub pending_games: i64,
    pub enriched_games: i64,
}

// Steam API response structures
#[derive(Debug, Deserialize)]
pub struct SteamAppDetailsResponse {
    #[serde(flatten)]
    pub apps: std::collections::HashMap<String, SteamAppResult>,
}

#[derive(Debug, Deserialize)]
pub struct SteamAppResult {
    pub success: bool,
    pub data: Option<SteamAppData>,
}

#[derive(Debug, Deserialize)]
pub struct SteamAppData {
    pub steam_appid: i64,
    pub name: String,
    pub short_description: Option<String>,
    pub header_image: Option<String>,
    pub background: Option<String>,
    pub developers: Option<Vec<String>>,
    pub publishers: Option<Vec<String>>,
    pub genres: Option<Vec<SteamGenre>>,
    pub release_date: Option<SteamReleaseDate>,
}

#[derive(Debug, Deserialize)]
pub struct SteamGenre {
    pub id: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct SteamReleaseDate {
    pub coming_soon: bool,
    pub date: Option<String>,
}

// Steam reviews
#[derive(Debug, Deserialize)]
pub struct SteamReviewsResponse {
    pub success: i32,
    pub query_summary: Option<SteamQuerySummary>,
}

#[derive(Debug, Deserialize)]
pub struct SteamQuerySummary {
    pub review_score: Option<i64>,
    pub review_score_desc: Option<String>,
    pub total_positive: Option<i64>,
    pub total_negative: Option<i64>,
    pub total_reviews: Option<i64>,
}

// Steam search
#[derive(Debug, Deserialize)]
pub struct SteamSearchResult {
    pub appid: i64,
    pub name: String,
}
