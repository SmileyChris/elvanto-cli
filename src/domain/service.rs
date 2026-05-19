use crate::api::raw::RawService;
use crate::domain::category::id_matches;
use serde::Serialize;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct VolunteerRow {
    pub department: String,
    pub department_id: String,
    pub sub_department: String,
    pub sub_department_id: String,
    pub position: String,
    pub position_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub person_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

impl VolunteerRow {
    /// Match by id (full UUID or short first-block) against department,
    /// sub-department, or position id. Empty `filters` matches everything.
    pub fn matches_department(&self, filters: &[String]) -> bool {
        if filters.is_empty() {
            return true;
        }
        filters.iter().any(|f| {
            id_matches(&self.department_id, f)
                || id_matches(&self.sub_department_id, f)
                || id_matches(&self.position_id, f)
        })
    }

    pub fn is_filled(&self) -> bool {
        self.person_id.is_some()
    }
}

/// Flatten a service's volunteer tree into a flat list of rows.
/// One row per volunteer; positions with no volunteers produce one row with
/// `person_id`/`name`/`status` set to `None`.
#[allow(dead_code)]
pub fn volunteer_rows(raw: &RawService) -> Vec<VolunteerRow> {
    let mut out = Vec::new();
    for plan in &raw.volunteers.plan {
        for position in &plan.positions.position {
            if position.volunteers.volunteer.is_empty() {
                out.push(VolunteerRow {
                    department: position.department_name.clone(),
                    department_id: position.department_id.clone(),
                    sub_department: position.sub_department_name.clone(),
                    sub_department_id: position.sub_department_id.clone(),
                    position: position.position_name.clone(),
                    position_id: position.position_id.clone(),
                    person_id: None,
                    name: None,
                    status: None,
                    email: None,
                });
            } else {
                for v in &position.volunteers.volunteer {
                    let name = v.person.display_name();
                    out.push(VolunteerRow {
                        department: position.department_name.clone(),
                        department_id: position.department_id.clone(),
                        sub_department: position.sub_department_name.clone(),
                        sub_department_id: position.sub_department_id.clone(),
                        position: position.position_name.clone(),
                        position_id: position.position_id.clone(),
                        person_id: if v.person.id.is_empty() {
                            None
                        } else {
                            Some(v.person.id.clone())
                        },
                        name: if name.is_empty() { None } else { Some(name) },
                        status: if v.status.is_empty() {
                            None
                        } else {
                            Some(v.status.to_ascii_lowercase())
                        },
                        email: None,
                    });
                }
            }
        }
    }
    out
}

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
    match trimmed.to_ascii_lowercase().as_str() {
        "" => "unknown".to_string(),
        "1" => "published".to_string(),
        "0" => "draft".to_string(),
        status => status.to_string(),
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
            volunteers: Default::default(),
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
    fn from_raw_numeric_status_codes_become_labels() {
        let mut r = raw();
        r.status = "1".into();
        let s: Service = r.into();
        assert_eq!(s.status, "published");

        let mut r = raw();
        r.status = "0".into();
        let s: Service = r.into();
        assert_eq!(s.status, "draft");
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

    #[test]
    fn volunteer_rows_flatten_filled_and_unfilled_positions() {
        let svc: RawService = serde_json::from_value(serde_json::json!({
            "id": "svc-1",
            "volunteers": {
                "plan": [{
                    "positions": {
                        "position": [
                            {
                                "department_name": "Service Teams",
                                "sub_department_name": "Service Leaders",
                                "position_name": "Preaching",
                                "volunteers": {
                                    "volunteer": [{
                                        "person": {
                                            "id": "p-1",
                                            "firstname": "Annedien",
                                            "lastname": "Looyenga",
                                            "preferred_name": ""
                                        },
                                        "status": "Confirmed"
                                    }]
                                }
                            },
                            {
                                "department_name": "Service Teams",
                                "sub_department_name": "Communion",
                                "position_name": "Setup & Cleanup",
                                "volunteers": ""
                            }
                        ]
                    }
                }]
            }
        }))
        .unwrap();
        let rows = volunteer_rows(&svc);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].position, "Preaching");
        assert!(rows[0].is_filled());
        assert_eq!(rows[0].name.as_deref(), Some("Annedien Looyenga"));
        assert_eq!(rows[0].status.as_deref(), Some("confirmed"));
        assert_eq!(rows[1].position, "Setup & Cleanup");
        assert!(!rows[1].is_filled());
        assert_eq!(rows[1].name, None);
    }

    #[test]
    fn volunteer_rows_emits_one_row_per_volunteer() {
        let svc: RawService = serde_json::from_value(serde_json::json!({
            "id": "svc-1",
            "volunteers": {
                "plan": [{
                    "positions": {
                        "position": [{
                            "department_name": "Sound",
                            "sub_department_name": "FOH",
                            "position_name": "Engineer",
                            "volunteers": {
                                "volunteer": [
                                    {"person": {"id": "p-1", "firstname": "Alice", "lastname": "B", "preferred_name": ""}, "status": "Confirmed"},
                                    {"person": {"id": "p-2", "firstname": "Bob", "lastname": "C", "preferred_name": ""}, "status": "Unconfirmed"}
                                ]
                            }
                        }]
                    }
                }]
            }
        }))
        .unwrap();
        let rows = volunteer_rows(&svc);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].name.as_deref(), Some("Alice B"));
        assert_eq!(rows[1].name.as_deref(), Some("Bob C"));
        assert_eq!(rows[1].status.as_deref(), Some("unconfirmed"));
    }
}
