use crate::models::{SteamAppDetailsResponse, SteamReviewsResponse};
use reqwest::Client;
use strsim::jaro_winkler;
use std::collections::HashMap;
use std::time::Duration;

const STEAM_STORE_API: &str = "https://store.steampowered.com/api";
const STEAM_SEARCH_URL: &str = "https://steamcommunity.com/actions/SearchApps";

/// Known game title to Steam App ID mappings
fn get_known_mappings() -> HashMap<&'static str, i64> {
    let mut m = HashMap::new();

    // Popular games with exact mappings
    m.insert("cyberpunk 2077", 1091500);
    m.insert("baldur's gate 3", 1086940);
    m.insert("elden ring", 1245620);
    m.insert("elden ring nightreign", 2622380);
    m.insert("doom eternal", 782330);
    m.insert("days gone", 1259420);
    m.insert("gta v", 271590);
    m.insert("grand theft auto v", 271590);
    m.insert("grand theft auto v enhanced", 271590);
    m.insert("snowrunner", 1465360);
    m.insert("arma 3", 107410);
    m.insert("forza horizon 5", 1551360);
    m.insert("forza motorsport", 2440510);
    m.insert("halo infinite", 1240440);
    m.insert("stalker 2 heart of chornobyl", 1643320);
    m.insert("s.t.a.l.k.e.r. 2", 1643320);
    m.insert("kingdom come deliverance ii", 1771300);
    m.insert("frostpunk", 323190);
    m.insert("frostpunk 2", 1601580);
    m.insert("cities skylines ii", 949230);
    m.insert("farming simulator 22", 1248130);
    m.insert("farming simulator 25", 2300320);
    m.insert("age of empires iv", 1466860);
    m.insert("age of empires ii definitive edition", 813780);
    m.insert("age of empires iii definitive edition", 933110);
    m.insert("age of empires definitive edition", 1017900);
    m.insert("hitman 3", 1659040);
    m.insert("hitman world of assassination", 1659040);
    m.insert("assassin's creed odyssey", 812140);
    m.insert("assassin's creed mirage", 2208920);
    m.insert("diablo 2 resurrected", 0); // Not on Steam
    m.insert("far cry 5", 552520);
    m.insert("need for speed heat", 1222680);
    m.insert("hollow knight silksong", 1030300);
    m.insert("alan wake 2", 0); // Epic exclusive
    m.insert("final fantasy vii remake intergrade", 1462040);
    m.insert("final fantasy vii rebirth", 2909400);
    m.insert("final fantasy xvi", 2515020);
    m.insert("conan exiles", 440900);
    m.insert("icarus", 1149460);
    m.insert("company of heroes 3", 1677280);
    m.insert("mechwarrior 5 clans", 1983350);
    m.insert("northgard", 466560);
    m.insert("space engineers", 244850);
    m.insert("automobilista 2", 1066890);
    m.insert("dirt rally 2.0", 690790);

    // C&C / Command & Conquer series
    m.insert("c&c - remastered collection", 1213210);
    m.insert("c&c remastered collection", 1213210);
    m.insert("command & conquer remastered collection", 1213210);
    m.insert("command and conquer remastered collection", 1213210);
    m.insert("c&c red alert 3", 17480);
    m.insert("command & conquer red alert 3", 17480);
    m.insert("c&c 3 tiberium wars", 24790);
    m.insert("command & conquer 3 tiberium wars", 24790);

    // Fallout series
    m.insert("fallout 4", 377160);
    m.insert("fallout 4 goty", 377160);
    m.insert("fallout 76", 1151340);
    m.insert("fallout new vegas", 22380);
    m.insert("fallout 3", 22300);
    m.insert("fallout 3 goty", 22300);

    // Gold Rush and other simulators
    m.insert("gold rush - the game", 451340);
    m.insert("gold rush the game", 451340);
    m.insert("euro truck simulator 2", 227300);
    m.insert("american truck simulator", 270880);
    m.insert("train sim world", 530070);
    m.insert("train sim world 2", 1282590);
    m.insert("train sim world 3", 1944790);
    m.insert("train sim world 4", 2362320);

    // Additional popular games
    m.insert("red dead redemption 2", 1174180);
    m.insert("rdr2", 1174180);
    m.insert("the witcher 3", 292030);
    m.insert("witcher 3", 292030);
    m.insert("witcher 3 wild hunt", 292030);
    m.insert("the witcher 3 wild hunt", 292030);
    m.insert("gta iv", 12210);
    m.insert("grand theft auto iv", 12210);
    m.insert("death stranding", 1190460);
    m.insert("death stranding director's cut", 1850570);
    m.insert("horizon zero dawn", 1151640);
    m.insert("horizon forbidden west", 2420110);
    m.insert("god of war", 1593500);
    m.insert("god of war ragnarok", 2322010);
    m.insert("resident evil 4", 2050650);
    m.insert("resident evil 4 remake", 2050650);
    m.insert("resident evil village", 1196590);
    m.insert("resident evil 8", 1196590);
    m.insert("sekiro", 814380);
    m.insert("sekiro shadows die twice", 814380);
    m.insert("dark souls iii", 374320);
    m.insert("dark souls 3", 374320);
    m.insert("dark souls remastered", 570940);
    m.insert("monster hunter rise", 1446780);
    m.insert("monster hunter world", 582010);
    m.insert("armored core vi", 1888160);
    m.insert("armored core 6", 1888160);
    m.insert("armored core vi fires of rubicon", 1888160);

    // Racing games
    m.insert("assetto corsa", 244210);
    m.insert("assetto corsa competizione", 805550);
    m.insert("f1 23", 2108330);
    m.insert("f1 2023", 2108330);
    m.insert("f1 24", 2488620);
    m.insert("f1 2024", 2488620);
    m.insert("need for speed unbound", 1846380);
    m.insert("need for speed most wanted", 1262540);
    m.insert("the crew motorfest", 1933490);
    m.insert("crew motorfest", 1933490);

    // Strategy games
    m.insert("total war warhammer iii", 1142710);
    m.insert("total war warhammer 3", 1142710);
    m.insert("civilization vi", 289070);
    m.insert("civilization 6", 289070);
    m.insert("civ 6", 289070);
    m.insert("crusader kings iii", 1158310);
    m.insert("crusader kings 3", 1158310);
    m.insert("europa universalis iv", 236850);
    m.insert("eu4", 236850);
    m.insert("stellaris", 281990);
    m.insert("hearts of iron iv", 394360);
    m.insert("hoi4", 394360);

    // Indie / AA titles
    m.insert("hades", 1145360);
    m.insert("hades ii", 1145350);
    m.insert("hades 2", 1145350);
    m.insert("hollow knight", 367520);
    m.insert("celeste", 504230);
    m.insert("cuphead", 268910);
    m.insert("dead cells", 588650);
    m.insert("stardew valley", 413150);
    m.insert("terraria", 105600);
    m.insert("valheim", 892970);
    m.insert("satisfactory", 526870);
    m.insert("factorio", 427520);
    m.insert("rimworld", 294100);
    m.insert("subnautica", 264710);
    m.insert("subnautica below zero", 848450);

    // Survival / Crafting
    m.insert("rust", 252490);
    m.insert("ark survival evolved", 346110);
    m.insert("ark survival ascended", 2399830);
    m.insert("the forest", 242760);
    m.insert("sons of the forest", 1326470);
    m.insert("raft", 648800);
    m.insert("grounded", 962130);
    m.insert("v rising", 1604030);
    m.insert("palworld", 1623730);

    // Horror
    m.insert("resident evil 2", 883710);
    m.insert("resident evil 2 remake", 883710);
    m.insert("resident evil 3", 952060);
    m.insert("resident evil 3 remake", 952060);
    m.insert("dead space", 1693980);
    m.insert("dead space remake", 1693980);
    m.insert("the callisto protocol", 1461830);
    m.insert("outlast", 238320);
    m.insert("amnesia rebirth", 999220);
    m.insert("amnesia the bunker", 1944430);

    // Sports
    m.insert("ea sports fc 24", 2195250);
    m.insert("fc 24", 2195250);
    m.insert("fifa 24", 2195250);
    m.insert("ea sports fc 25", 2669320);
    m.insert("fc 25", 2669320);
    m.insert("nba 2k24", 2338770);
    m.insert("nba 2k25", 2688840);

    // Other AAA
    m.insert("starfield", 1716740);
    m.insert("hogwarts legacy", 990080);
    m.insert("spider-man remastered", 1817070);
    m.insert("marvel's spider-man remastered", 1817070);
    m.insert("spider-man miles morales", 1817190);
    m.insert("marvel's spider-man miles morales", 1817190);
    m.insert("ghost of tsushima", 2215430);
    m.insert("ghost of tsushima director's cut", 2215430);
    m.insert("lies of p", 1627720);
    m.insert("lords of the fallen", 1501750);
    m.insert("wo long fallen dynasty", 1448440);
    m.insert("black myth wukong", 2358720);

    // Elder Scrolls
    m.insert("tes iv - oblivion remastered", 22330);
    m.insert("tes iv oblivion remastered", 22330);
    m.insert("oblivion remastered", 22330);
    m.insert("the elder scrolls iv oblivion", 22330);
    m.insert("tes v - skyrim", 489830);
    m.insert("skyrim special edition", 489830);
    m.insert("skyrim anniversary edition", 489830);

    // Warhammer
    m.insert("wh40k - space marine", 55150);
    m.insert("wh40k space marine", 55150);
    m.insert("warhammer 40000 space marine", 55150);
    m.insert("wh40k - space marine mce", 55150);
    m.insert("space marine 2", 2183900);
    m.insert("warhammer 40000 space marine 2", 2183900);

    // DOOM series
    m.insert("doom classic bundle", 2280);
    m.insert("doom i & ii enhanced", 2280);
    m.insert("doom 1", 2280);
    m.insert("doom 2", 2300);
    m.insert("doom 3", 9050);
    m.insert("doom 2016", 379720);
    m.insert("doom", 379720);

    // Syberia
    m.insert("syberia - remastered", 46500);
    m.insert("syberia remastered", 46500);
    m.insert("syberia", 46500);
    m.insert("syberia 2", 46510);
    m.insert("syberia 3", 464340);
    m.insert("syberia the world before", 1410680);

    // GTA Trilogy - Now on Steam!
    m.insert("gta trilogy - definitive edition", 1847330);  // GTA III DE as representative
    m.insert("gta trilogy definitive edition", 1847330);
    m.insert("grand theft auto trilogy - definitive edition", 1847330);
    m.insert("grand theft auto trilogy definitive edition", 1847330);
    m.insert("gta iii definitive edition", 1847330);
    m.insert("gta vice city definitive edition", 1546990);
    m.insert("gta san andreas definitive edition", 1547000);

    // Commandos - Released April 2025!
    m.insert("commandos - origins", 1479730);
    m.insert("commandos origins", 1479730);

    // Non-Steam (mark as 0 to skip)
    m.insert("diablo 2 - resurrected", 0);  // Battle.net exclusive
    m.insert("diablo 2 resurrected", 0);
    m.insert("diablo ii resurrected", 0);
    m.insert("pokemon legends - z-a", 0);  // Nintendo exclusive
    m.insert("pokemon legends z-a", 0);
    m.insert("super mario galaxy 1 + 2", 0);  // Nintendo exclusive
    m.insert("super mario galaxy", 0);
    m.insert("mgs delta - snake eater", 0);  // Unreleased
    m.insert("mgs delta snake eater", 0);

    // Jurassic Park
    m.insert("jurassic park cgc", 275890);
    m.insert("jurassic park the game", 275890);
    m.insert("jurassic world evolution", 648350);
    m.insert("jurassic world evolution 2", 1244460);

    m
}

/// Search for a Steam App ID using the search API
pub async fn search_steam_app(client: &Client, title: &str) -> Option<(i64, f64)> {
    // First check known mappings
    let lower_title = title.to_lowercase();
    let mappings = get_known_mappings();

    for (known_title, app_id) in &mappings {
        let similarity = jaro_winkler(&lower_title, known_title);
        if similarity > 0.85 && *app_id > 0 {
            tracing::info!("Found known mapping for '{}': {} (similarity: {:.2})", title, app_id, similarity);
            return Some((*app_id, similarity));
        }
    }

    // Search Steam
    let url = format!("{}/{}", STEAM_SEARCH_URL, urlencoding::encode(title));

    let response = match client.get(&url)
        .timeout(Duration::from_secs(10))
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("Steam search failed for '{}': {}", title, e);
            return None;
        }
    };

    let results: Vec<serde_json::Value> = match response.json().await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("Failed to parse Steam search results for '{}': {}", title, e);
            return None;
        }
    };

    // Find best match using Jaro-Winkler similarity
    let mut best_match: Option<(i64, f64)> = None;

    for result in results.iter().take(5) {
        if let (Some(appid), Some(name)) = (
            result.get("appid").and_then(|v| v.as_str()).and_then(|s| s.parse::<i64>().ok()),
            result.get("name").and_then(|v| v.as_str()),
        ) {
            let similarity = jaro_winkler(&lower_title, &name.to_lowercase());

            if similarity > best_match.map(|(_, s)| s).unwrap_or(0.0) {
                best_match = Some((appid, similarity));
            }
        }
    }

    if let Some((appid, similarity)) = best_match {
        if similarity > 0.6 {
            tracing::info!("Found Steam match for '{}': {} (similarity: {:.2})", title, appid, similarity);
            return Some((appid, similarity));
        }
    }

    tracing::info!("No Steam match found for '{}'", title);
    None
}

/// Fetch game details from Steam
pub async fn fetch_steam_details(client: &Client, app_id: i64) -> Option<SteamAppDetails> {
    let url = format!("{}/appdetails?appids={}", STEAM_STORE_API, app_id);

    let response = match client.get(&url)
        .timeout(Duration::from_secs(10))
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("Failed to fetch Steam details for {}: {}", app_id, e);
            return None;
        }
    };

    let data: SteamAppDetailsResponse = match response.json().await {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!("Failed to parse Steam details for {}: {}", app_id, e);
            return None;
        }
    };

    let app_result = data.apps.get(&app_id.to_string())?;

    if !app_result.success {
        return None;
    }

    let app_data = app_result.data.as_ref()?;

    Some(SteamAppDetails {
        app_id,
        name: app_data.name.clone(),
        description: app_data.short_description.clone(),
        header_image: app_data.header_image.clone(),
        background: app_data.background.clone(),
        developers: app_data.developers.clone(),
        publishers: app_data.publishers.clone(),
        genres: app_data.genres.as_ref().map(|g| {
            g.iter().map(|genre| genre.description.clone()).collect()
        }),
        release_date: app_data.release_date.as_ref().and_then(|r| r.date.clone()),
    })
}

/// Fetch reviews from Steam
pub async fn fetch_steam_reviews(client: &Client, app_id: i64) -> Option<SteamReviews> {
    let url = format!(
        "{}/appreviews/{}?json=1&language=all&purchase_type=all&num_per_page=0",
        STEAM_STORE_API, app_id
    );

    let response = match client.get(&url)
        .timeout(Duration::from_secs(10))
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("Failed to fetch Steam reviews for {}: {}", app_id, e);
            return None;
        }
    };

    let data: SteamReviewsResponse = match response.json().await {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!("Failed to parse Steam reviews for {}: {}", app_id, e);
            return None;
        }
    };

    if data.success != 1 {
        return None;
    }

    let summary = data.query_summary?;

    let total = summary.total_positive.unwrap_or(0) + summary.total_negative.unwrap_or(0);
    let score = if total > 0 {
        (summary.total_positive.unwrap_or(0) * 100) / total
    } else {
        0
    };

    Some(SteamReviews {
        score,
        count: summary.total_reviews.unwrap_or(0),
        summary: summary.review_score_desc.unwrap_or_default(),
    })
}

#[derive(Debug, Clone)]
pub struct SteamAppDetails {
    pub app_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub header_image: Option<String>,
    pub background: Option<String>,
    pub developers: Option<Vec<String>>,
    pub publishers: Option<Vec<String>>,
    pub genres: Option<Vec<String>>,
    pub release_date: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SteamReviews {
    pub score: i64,
    pub count: i64,
    pub summary: String,
}

// urlencoding is imported from the crate
