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

#[derive(Debug, Deserialize, Default, Clone)]
pub struct CategoryList {
    #[serde(default)]
    pub category: Vec<RawCategory>,
}

#[derive(Debug, Deserialize, Clone)]
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
    #[allow(dead_code)]
    #[serde(default)]
    pub page: u32,
    #[serde(default)]
    pub per_page: u32,
    #[serde(default)]
    pub total: u32,
    #[allow(dead_code)]
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
        if self.is_active() {
            "active"
        } else {
            "archived"
        }
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

#[allow(dead_code)]
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

#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawSongDetail {
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub artist: String,
    #[serde(default)]
    pub album: String,
    #[serde(default)]
    pub number: String,
    #[serde(default)]
    pub status: serde_json::Value,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub sequence: String,
    #[serde(default)]
    pub bpm: String,
    #[serde(default)]
    pub duration: String,
    #[serde(default)]
    pub learn: serde_json::Value,
    #[serde(default)]
    pub allow_downloads: serde_json::Value,
    #[serde(default)]
    pub categories: CategoryList,
    #[serde(default)]
    pub locations: LocationList,
    #[serde(default)]
    pub arrangements: ArrangementList,
    /// Present only when `files=1` requested.
    #[serde(default)]
    pub files: serde_json::Value,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct LocationList {
    #[serde(default)]
    pub location: Vec<RawLocation>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawLocation {
    pub id: String,
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct SongInfoResponse {
    pub songs: SongInfoInner,
}

#[derive(Debug, Deserialize)]
pub struct SongInfoInner {
    #[serde(default)]
    pub song: Vec<RawSongDetail>,
}

pub fn truthy(v: &serde_json::Value) -> bool {
    match v {
        serde_json::Value::Bool(b) => *b,
        serde_json::Value::Number(n) => n.as_i64().map(|i| i != 0).unwrap_or(false),
        serde_json::Value::String(s) => s == "1" || s.eq_ignore_ascii_case("true"),
        _ => false,
    }
}

#[derive(Debug, Deserialize)]
pub struct ServicesResponse {
    #[serde(default)]
    pub services: ServiceList,
}

#[derive(Debug, Deserialize, Default)]
pub struct ServiceList {
    #[allow(dead_code)]
    #[serde(default)]
    pub page: u32,
    #[serde(default)]
    pub per_page: u32,
    #[serde(default)]
    pub total: u32,
    #[allow(dead_code)]
    #[serde(default)]
    pub on_this_page: u32,
    #[serde(default)]
    pub service: Vec<RawService>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawService {
    pub id: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub service_type: RawServiceType,
    #[serde(default)]
    pub location: RawServiceLocation,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawServiceType {
    #[allow(dead_code)]
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawServiceLocation {
    #[allow(dead_code)]
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
}
