use crate::api::raw::RawService;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Service {
    pub id: String,
    /// Original Elvanto timestamp, "YYYY-MM-DD HH:MM:SS".
    pub date: String,
    pub name: String,
    pub status: String,
    pub service_type: String,
    pub location: Option<String>,
    pub description: Option<String>,
}

impl Service {
    /// First 10 chars of the date string, i.e. "YYYY-MM-DD".
    pub fn date_short(&self) -> &str {
        if self.date.len() >= 10 {
            &self.date[..10]
        } else {
            &self.date
        }
    }
}

fn normalize_status(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        "unknown".to_string()
    } else {
        trimmed.to_ascii_lowercase()
    }
}

fn none_if_empty(s: String) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

impl From<RawService> for Service {
    fn from(raw: RawService) -> Self {
        Self {
            id: raw.id,
            date: raw.date,
            name: raw.name,
            status: normalize_status(&raw.status),
            service_type: raw.service_type.name,
            location: none_if_empty(raw.location.name),
            description: none_if_empty(raw.description),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::raw::{
        RawServiceLocation, RawServicePlanList, RawServiceSongList, RawServiceType,
    };

    fn raw() -> RawService {
        RawService {
            id: "svc-1".into(),
            date: "2026-04-12 09:30:00".into(),
            name: "Sunday Morning".into(),
            description: "Easter".into(),
            status: "Published".into(),
            service_type: RawServiceType {
                id: "st-1".into(),
                name: "Sunday Service".into(),
            },
            location: RawServiceLocation {
                id: "loc-1".into(),
                name: "Main".into(),
            },
            songs: RawServiceSongList::default(),
            plans: RawServicePlanList::default(),
        }
    }

    #[test]
    fn from_raw_normalizes_status_to_lowercase() {
        let s: Service = raw().into();
        assert_eq!(s.status, "published");
    }

    #[test]
    fn from_raw_flattens_service_type_to_name() {
        let s: Service = raw().into();
        assert_eq!(s.service_type, "Sunday Service");
    }

    #[test]
    fn from_raw_empty_status_becomes_unknown() {
        let mut r = raw();
        r.status = "".into();
        let s: Service = r.into();
        assert_eq!(s.status, "unknown");
    }

    #[test]
    fn from_raw_empty_location_becomes_none() {
        let mut r = raw();
        r.location.name = "".into();
        let s: Service = r.into();
        assert_eq!(s.location, None);
    }

    #[test]
    fn date_short_takes_first_ten_chars() {
        let s: Service = raw().into();
        assert_eq!(s.date_short(), "2026-04-12");
    }

    #[test]
    fn date_short_handles_short_string() {
        let mut r = raw();
        r.date = "2026".into();
        let s: Service = r.into();
        assert_eq!(s.date_short(), "2026");
    }

    #[test]
    fn json_field_order_and_names() {
        let s: Service = raw().into();
        let v = serde_json::to_value(&s).unwrap();
        assert_eq!(v["id"], "svc-1");
        assert_eq!(v["service_type"], "Sunday Service");
        assert_eq!(v["status"], "published");
        assert_eq!(v["location"], "Main");
    }
}
