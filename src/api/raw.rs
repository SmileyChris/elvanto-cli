use serde::Deserialize;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct RawEnvelope {
    pub status: String,
    #[serde(default)]
    pub error: Option<ApiError>,
}

#[derive(Debug, Deserialize)]
pub struct ApiError {
    pub code: i64,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct CategoriesResponse {
    #[serde(default)]
    pub categories: CategoryList,
}

#[derive(Debug, Deserialize, Default)]
pub struct CategoryList {
    #[serde(default)]
    pub category: Vec<RawCategory>,
}

#[derive(Debug, Deserialize)]
pub struct RawCategory {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct SongsResponse {
    #[serde(default)]
    pub songs: SongList,
}

#[derive(Debug, Deserialize, Default)]
pub struct SongList {
    #[serde(default)]
    pub page: u32,
    #[serde(default)]
    pub per_page: u32,
    #[serde(default)]
    pub total: u32,
    #[serde(default)]
    pub on_this_page: u32,
    #[serde(default)]
    pub song: Vec<RawSong>,
}

#[derive(Debug, Deserialize)]
pub struct RawSong {
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub artist: String,
    #[serde(default)]
    pub album: String,
    /// CCLI number per Elvanto docs.
    #[serde(default)]
    pub number: String,
    /// "1" = active, "0" = inactive (Elvanto serializes booleans as numeric strings here).
    #[serde(default)]
    pub status: serde_json::Value,
}

impl RawSong {
    pub fn is_active(&self) -> bool {
        match &self.status {
            serde_json::Value::String(s) => s == "1" || s.eq_ignore_ascii_case("active"),
            serde_json::Value::Number(n) => n.as_i64() == Some(1),
            _ => false,
        }
    }

    pub fn status_label(&self) -> &'static str {
        if self.is_active() { "active" } else { "archived" }
    }
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct ArrangementList {
    #[serde(default)]
    pub arrangement: Vec<RawArrangement>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawArrangement {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub sequence: String,
    #[serde(default)]
    pub bpm: String,
    #[serde(default)]
    pub duration: String,
    /// Chord chart text. Field name varies by Elvanto endpoint version; accept both.
    #[serde(default, alias = "chord_chart")]
    pub chord_pro: String,
    #[serde(default)]
    pub lyrics: String,
    #[serde(default)]
    pub keys: KeyList,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct KeyList {
    #[serde(default)]
    pub key: Vec<RawKey>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawKey {
    pub id: String,
    #[serde(default, alias = "starting_key")]
    pub starting: String,
    #[serde(default, alias = "ending_key")]
    pub ending: String,
}

#[derive(Debug, Deserialize)]
pub struct ArrangementsResponse {
    #[serde(default)]
    pub arrangements: ArrangementList,
}

#[derive(Debug, Deserialize)]
pub struct ArrangementInfoResponse {
    #[serde(default)]
    pub arrangement: RawArrangement,
}
