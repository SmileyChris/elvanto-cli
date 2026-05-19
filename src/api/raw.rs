use serde::{de, Deserialize, Deserializer};

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
    #[serde(default, deserialize_with = "deserialize_u32ish")]
    pub page: u32,
    #[serde(default, deserialize_with = "deserialize_u32ish")]
    pub per_page: u32,
    #[serde(default, deserialize_with = "deserialize_u32ish")]
    pub total: u32,
    #[allow(dead_code)]
    #[serde(default, deserialize_with = "deserialize_u32ish")]
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
    #[serde(default)]
    pub categories: CategoryList,
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
        serde_json::Value::String(s) => {
            s == "1" || s.eq_ignore_ascii_case("true") || s.eq_ignore_ascii_case("active")
        }
        _ => false,
    }
}

fn deserialize_u32ish<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Null => Ok(0),
        serde_json::Value::Number(n) => {
            let value = n
                .as_u64()
                .ok_or_else(|| de::Error::custom(format!("expected unsigned integer, got {n}")))?;
            u32::try_from(value)
                .map_err(|_| de::Error::custom(format!("integer {value} is too large for u32")))
        }
        serde_json::Value::String(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                Ok(0)
            } else {
                trimmed.parse::<u32>().map_err(|_| {
                    de::Error::custom(format!("expected numeric string for u32, got {s:?}"))
                })
            }
        }
        other => Err(de::Error::custom(format!(
            "expected unsigned integer or numeric string, got {other}"
        ))),
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
    #[serde(default, deserialize_with = "deserialize_u32ish")]
    pub page: u32,
    #[serde(default, deserialize_with = "deserialize_u32ish")]
    pub per_page: u32,
    #[serde(default, deserialize_with = "deserialize_u32ish")]
    pub total: u32,
    #[allow(dead_code)]
    #[serde(default, deserialize_with = "deserialize_u32ish")]
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn songs_response_accepts_string_pagination_numbers() {
        let resp: SongsResponse = serde_json::from_value(json!({
            "status": "ok",
            "songs": {
                "page": "1",
                "per_page": "100",
                "total": "132",
                "on_this_page": "100",
                "song": []
            }
        }))
        .unwrap();

        assert_eq!(resp.songs.page, 1);
        assert_eq!(resp.songs.per_page, 100);
        assert_eq!(resp.songs.total, 132);
        assert_eq!(resp.songs.on_this_page, 100);
    }
}
