use crate::api::raw::{truthy, RawSong, RawSongDetail};
use crate::domain::arrangement::Arrangement;
use crate::domain::category::Category;
use crate::domain::none_if_empty;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SongSummary {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub ccli_number: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<usize>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<String>,
}

impl From<RawSong> for SongSummary {
    fn from(raw: RawSong) -> Self {
        let status = raw.status_label().to_string();
        Self {
            id: raw.id,
            title: raw.title,
            artist: raw.artist,
            album: raw.album,
            ccli_number: raw.number,
            status,
            last_used: None,
            count: None,
            categories: raw.categories.category.iter().map(|c| c.name.clone()).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Location {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SongDetail {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub ccli_number: String,
    pub status: String,
    pub notes: Option<String>,
    pub sequence: Option<String>,
    pub bpm: Option<String>,
    pub duration: Option<String>,
    pub learn: bool,
    pub allow_downloads: bool,
    pub categories: Vec<Category>,
    pub locations: Vec<Location>,
    pub arrangements: Vec<Arrangement>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<serde_json::Value>,
}

impl From<RawSongDetail> for SongDetail {
    fn from(raw: RawSongDetail) -> Self {
        let status = if truthy(&raw.status) {
            "active"
        } else {
            "archived"
        }
        .to_string();
        let files = match raw.files {
            serde_json::Value::Null => None,
            other => Some(other),
        };
        Self {
            id: raw.id,
            title: raw.title,
            artist: raw.artist,
            album: raw.album,
            ccli_number: raw.number,
            status,
            notes: none_if_empty(raw.notes),
            sequence: none_if_empty(raw.sequence),
            bpm: none_if_empty(raw.bpm),
            duration: none_if_empty(raw.duration),
            learn: truthy(&raw.learn),
            allow_downloads: truthy(&raw.allow_downloads),
            categories: raw
                .categories
                .category
                .into_iter()
                .map(Into::into)
                .collect(),
            locations: raw
                .locations
                .location
                .into_iter()
                .map(|l| Location {
                    id: l.id,
                    name: l.name,
                })
                .collect(),
            arrangements: raw
                .arrangements
                .arrangement
                .into_iter()
                .map(Into::into)
                .collect(),
            files,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn raw(status: serde_json::Value) -> RawSong {
        RawSong {
            id: "s1".into(),
            title: "Amazing Grace".into(),
            artist: "Trad.".into(),
            album: "".into(),
            number: "22025".into(),
            status,
            categories: Default::default(),
        }
    }

    #[test]
    fn numeric_string_status_active() {
        let s: SongSummary = raw(json!("1")).into();
        assert_eq!(s.status, "active");
        assert_eq!(s.ccli_number, "22025");
    }

    #[test]
    fn numeric_string_status_inactive() {
        let s: SongSummary = raw(json!("0")).into();
        assert_eq!(s.status, "archived");
    }

    #[test]
    fn detail_status_active_string_is_active() {
        let detail: SongDetail = RawSongDetail {
            id: "s1".into(),
            title: "Amazing Grace".into(),
            status: json!("active"),
            ..Default::default()
        }
        .into();

        assert_eq!(detail.status, "active");
    }

    #[test]
    fn json_serializes_ccli_field_name() {
        let s: SongSummary = raw(json!("1")).into();
        let v = serde_json::to_value(&s).unwrap();
        assert!(v.get("ccli_number").is_some());
        assert!(v.get("number").is_none());
        assert!(v.get("last_used").is_none());
    }
}
