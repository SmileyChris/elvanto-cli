use crate::api::raw::RawSong;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SongSummary {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub ccli_number: String,
    pub status: String,
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
    fn json_serializes_ccli_field_name() {
        let s: SongSummary = raw(json!("1")).into();
        let v = serde_json::to_value(&s).unwrap();
        assert!(v.get("ccli_number").is_some());
        assert!(v.get("number").is_none());
    }
}
