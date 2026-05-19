use crate::api::raw::{RawArrangement, RawKey};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Key {
    pub id: String,
    pub starting: String,
    pub ending: Option<String>,
}

impl From<RawKey> for Key {
    fn from(raw: RawKey) -> Self {
        let ending = if raw.ending.is_empty() { None } else { Some(raw.ending) };
        Self { id: raw.id, starting: raw.starting, ending }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Arrangement {
    pub id: String,
    pub name: String,
    pub sequence: Option<String>,
    pub bpm: Option<String>,
    pub duration: Option<String>,
    pub keys: Vec<Key>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lyrics: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chord_chart: Option<String>,
}

fn none_if_empty(s: String) -> Option<String> {
    if s.is_empty() { None } else { Some(s) }
}

impl From<RawArrangement> for Arrangement {
    fn from(raw: RawArrangement) -> Self {
        Self {
            id: raw.id,
            name: raw.name,
            sequence: none_if_empty(raw.sequence),
            bpm: none_if_empty(raw.bpm),
            duration: none_if_empty(raw.duration),
            keys: raw.keys.key.into_iter().map(Into::into).collect(),
            lyrics: none_if_empty(raw.lyrics),
            chord_chart: none_if_empty(raw.chord_pro),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::raw::KeyList;

    #[test]
    fn empty_ending_key_becomes_none() {
        let raw = RawKey { id: "k1".into(), starting: "G".into(), ending: String::new() };
        let key: Key = raw.into();
        assert_eq!(key.ending, None);
    }

    #[test]
    fn empty_lyrics_chord_chart_become_none() {
        let raw = RawArrangement {
            id: "a1".into(),
            name: "Default".into(),
            keys: KeyList { key: vec![] },
            ..Default::default()
        };
        let arr: Arrangement = raw.into();
        assert!(arr.lyrics.is_none());
        assert!(arr.chord_chart.is_none());
    }
}
