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

fn deserialize_stringish<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Null => Ok(String::new()),
        serde_json::Value::String(s) => Ok(s),
        serde_json::Value::Number(n) => Ok(n.to_string()),
        serde_json::Value::Bool(b) => Ok(b.to_string()),
        other => Err(de::Error::custom(format!(
            "expected string-compatible scalar, got {other}"
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
    #[serde(default, deserialize_with = "deserialize_stringish")]
    pub status: String,
    #[serde(default)]
    pub service_type: RawServiceType,
    #[serde(default)]
    pub location: RawServiceLocation,
    #[serde(default)]
    pub songs: RawServiceSongList,
    #[serde(default)]
    pub plans: RawServicePlanList,
    #[allow(dead_code)]
    #[serde(default)]
    pub volunteers: RawServiceVolunteers,
}

impl RawService {
    pub fn song_ids(&self) -> Vec<&str> {
        let mut ids = std::collections::HashSet::new();
        ids.extend(self.songs.song.iter().map(|song| song.id.as_str()));

        for plan in &self.plans.plan {
            for item in &plan.items.item {
                if let Some(id) = item.song.get("id").and_then(|id| id.as_str()) {
                    ids.insert(id);
                }
            }
        }

        ids.into_iter().collect()
    }

    /// Song occurrences on this service with the key set for each (if any).
    /// Merges the service song list with plan items; plan-item keys win when
    /// both are present.
    pub fn song_uses(&self) -> Vec<ServiceSongUse> {
        let mut map: std::collections::BTreeMap<String, Option<String>> =
            std::collections::BTreeMap::new();

        for song in &self.songs.song {
            if !song.id.is_empty() {
                map.entry(song.id.clone()).or_insert_with(|| song.arrangement.key.clone());
            }
        }
        for plan in &self.plans.plan {
            for item in &plan.items.item {
                if let Some(id) = item.song.get("id").and_then(|id| id.as_str()) {
                    if id.is_empty() {
                        continue;
                    }
                    let key = item
                        .song
                        .get("arrangement")
                        .and_then(|a| a.get("key"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let entry = map.entry(id.to_string()).or_insert(None);
                    if key.is_some() {
                        *entry = key;
                    }
                }
            }
        }

        map.into_iter()
            .map(|(id, key)| ServiceSongUse { id, key })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct ServiceSongUse {
    pub id: String,
    /// Key set for the arrangement on this service (e.g. "C"), if recorded.
    pub key: Option<String>,
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

#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawServiceSongList {
    #[serde(default)]
    pub song: Vec<RawServiceSong>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawServiceSong {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub arrangement: RawServiceArrangement,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawServiceArrangement {
    /// Key set for this arrangement on this service (if any).
    #[serde(default)]
    pub key: Option<String>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawServicePlanList {
    #[serde(default)]
    pub plan: Vec<RawServicePlan>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawServicePlan {
    #[serde(default)]
    pub items: RawServiceItemList,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawServiceItemList {
    #[serde(default)]
    pub item: Vec<RawServiceItem>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawServiceItem {
    #[serde(default)]
    pub song: serde_json::Value,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ServiceInfoResponse {
    #[serde(default)]
    pub service: Vec<RawService>,
}

#[derive(Debug, Deserialize)]
pub struct PeopleResponse {
    #[serde(default)]
    pub people: PeopleList,
}

#[derive(Debug, Deserialize, Default)]
pub struct PeopleList {
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
    pub person: Vec<RawPersonRecord>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawPersonRecord {
    pub id: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub firstname: String,
    #[serde(default)]
    pub preferred_name: String,
    #[serde(default)]
    pub lastname: String,
    #[serde(default)]
    pub status: String,
    #[serde(default, deserialize_with = "deserialize_person_departments")]
    pub departments: RawPersonDepartments,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawPersonDepartments {
    #[serde(default)]
    pub department: Vec<RawPersonDepartment>,
}

/// Elvanto returns `departments: []` for people with none, and
/// `departments: { department: [...] }` for people with at least one. Normalise.
fn deserialize_person_departments<'de, D>(deserializer: D) -> Result<RawPersonDepartments, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Array(_) | serde_json::Value::Null => {
            Ok(RawPersonDepartments::default())
        }
        other => serde_json::from_value(other).map_err(de::Error::custom),
    }
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawPersonDepartment {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_person_sub_departments")]
    pub sub_departments: RawPersonSubDepartments,
}

fn deserialize_person_sub_departments<'de, D>(
    deserializer: D,
) -> Result<RawPersonSubDepartments, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Array(_) | serde_json::Value::Null => {
            Ok(RawPersonSubDepartments::default())
        }
        other => serde_json::from_value(other).map_err(de::Error::custom),
    }
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawPersonSubDepartments {
    #[serde(default)]
    pub sub_department: Vec<RawPersonSubDepartment>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawPersonSubDepartment {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_person_positions")]
    pub positions: RawPersonPositions,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawPersonPositions {
    #[serde(default)]
    pub position: Vec<RawPersonPosition>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawPersonPosition {
    pub id: String,
    #[serde(default)]
    pub name: String,
}

fn deserialize_person_positions<'de, D>(deserializer: D) -> Result<RawPersonPositions, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Array(_) | serde_json::Value::Null => Ok(RawPersonPositions::default()),
        other => serde_json::from_value(other).map_err(de::Error::custom),
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawServiceVolunteers {
    #[serde(default)]
    pub plan: Vec<RawVolunteerPlan>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawVolunteerPlan {
    #[serde(default)]
    pub positions: RawPositionList,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawPositionList {
    #[serde(default)]
    pub position: Vec<RawPosition>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawPosition {
    #[serde(default)]
    pub department_id: String,
    #[serde(default)]
    pub department_name: String,
    #[serde(default)]
    pub sub_department_id: String,
    #[serde(default)]
    pub sub_department_name: String,
    #[serde(default)]
    pub position_id: String,
    #[serde(default)]
    pub position_name: String,
    #[serde(default, deserialize_with = "deserialize_volunteers_field")]
    pub volunteers: RawVolunteersField,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawVolunteersField {
    #[serde(default)]
    pub volunteer: Vec<RawVolunteer>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawVolunteer {
    #[serde(default)]
    pub person: RawPerson,
    #[serde(default)]
    pub status: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawPerson {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub firstname: String,
    #[serde(default)]
    pub lastname: String,
    #[serde(default)]
    pub preferred_name: String,
}

impl RawPerson {
    #[allow(dead_code)]
    pub fn display_name(&self) -> String {
        let first = if self.preferred_name.is_empty() {
            &self.firstname
        } else {
            &self.preferred_name
        };
        match (first.is_empty(), self.lastname.is_empty()) {
            (true, true) => String::new(),
            (false, true) => first.clone(),
            (true, false) => self.lastname.clone(),
            (false, false) => format!("{} {}", first, self.lastname),
        }
    }
}

/// Elvanto returns `volunteers: ""` (empty string) when no one is assigned,
/// and `volunteers: { volunteer: [...] }` otherwise. Accept either.
fn deserialize_volunteers_field<'de, D>(deserializer: D) -> Result<RawVolunteersField, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::String(_) | serde_json::Value::Null => Ok(RawVolunteersField::default()),
        other => serde_json::from_value(other).map_err(de::Error::custom),
    }
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

    #[test]
    fn service_song_ids_include_sidebar_and_plan_songs() {
        let service: RawService = serde_json::from_value(json!({
            "id": "svc-1",
            "songs": {
                "song": [
                    { "id": "sidebar-song" }
                ]
            },
            "plans": {
                "plan": [
                    {
                        "items": {
                            "item": [
                                { "song": "" },
                                { "song": { "id": "plan-song" } }
                            ]
                        }
                    }
                ]
            }
        }))
        .unwrap();

        let mut ids = service.song_ids();
        ids.sort();
        assert_eq!(ids, vec!["plan-song", "sidebar-song"]);
    }

    #[test]
    fn services_response_accepts_numeric_service_status() {
        let resp: ServicesResponse = serde_json::from_value(json!({
            "status": "ok",
            "services": {
                "service": [
                    {
                        "id": "svc-1",
                        "date": "2026-05-19 09:30:00",
                        "name": "Sunday Morning",
                        "status": 1
                    }
                ]
            }
        }))
        .unwrap();

        assert_eq!(resp.services.service[0].status, "1");
    }

    #[test]
    fn position_volunteers_accepts_empty_string() {
        let pos: RawPosition = serde_json::from_value(json!({
            "department_name": "Service Teams",
            "sub_department_name": "Communion",
            "position_name": "Setup & Cleanup",
            "volunteers": ""
        }))
        .unwrap();
        assert!(pos.volunteers.volunteer.is_empty());
    }

    #[test]
    fn position_volunteers_accepts_volunteer_object() {
        let pos: RawPosition = serde_json::from_value(json!({
            "department_name": "Service Teams",
            "sub_department_name": "Service Leaders",
            "position_name": "Preaching",
            "volunteers": {
                "volunteer": [
                    {
                        "person": {
                            "id": "p-1",
                            "firstname": "Annedien",
                            "lastname": "Looyenga",
                            "preferred_name": ""
                        },
                        "status": "Confirmed"
                    }
                ]
            }
        }))
        .unwrap();
        assert_eq!(pos.volunteers.volunteer.len(), 1);
        assert_eq!(pos.volunteers.volunteer[0].person.firstname, "Annedien");
        assert_eq!(pos.volunteers.volunteer[0].status, "Confirmed");
    }

    #[test]
    fn person_display_name_prefers_preferred() {
        let p = RawPerson {
            id: "p1".into(),
            firstname: "Robert".into(),
            lastname: "Smith".into(),
            preferred_name: "Bob".into(),
        };
        assert_eq!(p.display_name(), "Bob Smith");
    }

    #[test]
    fn person_display_name_falls_back_to_firstname() {
        let p = RawPerson {
            id: "p1".into(),
            firstname: "Robert".into(),
            lastname: "Smith".into(),
            preferred_name: String::new(),
        };
        assert_eq!(p.display_name(), "Robert Smith");
    }
}
