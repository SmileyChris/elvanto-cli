use crate::api::raw::{CategoriesResponse, RawCategory, RawSong, SongsResponse};
use crate::api::Client;
use crate::error::CliError;

const SONGS_PAGE_SIZE: u32 = 100;

impl Client {
    pub async fn list_categories(&self) -> Result<Vec<RawCategory>, CliError> {
        let resp: CategoriesResponse = self
            .post("songs/categories/getAll", &serde_json::json!({}))
            .await?;
        Ok(resp.categories.category)
    }

    pub async fn list_all_songs(&self) -> Result<Vec<RawSong>, CliError> {
        let mut out = Vec::new();
        let mut page: u32 = 1;
        loop {
            let resp: SongsResponse = self
                .post(
                    "songs/getAll",
                    &serde_json::json!({
                        "item": 0,
                        "page": page,
                        "page_size": SONGS_PAGE_SIZE,
                    }),
                )
                .await?;
            let got = resp.songs.song.len() as u32;
            out.extend(resp.songs.song);
            let per_page = if resp.songs.per_page == 0 {
                SONGS_PAGE_SIZE
            } else {
                resp.songs.per_page
            };
            if got < per_page || (resp.songs.total > 0 && out.len() as u32 >= resp.songs.total) {
                break;
            }
            page += 1;
            if page > 1000 {
                break; // safety brake
            }
        }
        Ok(out)
    }

    pub async fn get_song_info(
        &self,
        id: &str,
        with_files: bool,
    ) -> Result<crate::api::raw::RawSongDetail, CliError> {
        let body = if with_files {
            serde_json::json!({ "id": id, "files": 1 })
        } else {
            serde_json::json!({ "id": id })
        };
        let resp: crate::api::raw::SongInfoResponse = self.post("songs/getInfo", &body).await?;
        resp.song
            .into_iter()
            .next()
            .ok_or_else(|| CliError::NotFound(format!("song {id}")))
    }

    pub async fn list_arrangements_for_song(
        &self,
        song_id: &str,
    ) -> Result<Vec<crate::api::raw::RawArrangement>, CliError> {
        const PAGE_SIZE: u32 = 100;
        let mut out = Vec::new();
        let mut page: u32 = 1;
        loop {
            let resp: crate::api::raw::ArrangementsResponse = self
                .post(
                    "songs/arrangements/getAll",
                    &serde_json::json!({
                        "song_id": song_id,
                        "page": page,
                        "page_size": PAGE_SIZE,
                    }),
                )
                .await?;
            let got = resp.arrangements.arrangement.len() as u32;
            out.extend(resp.arrangements.arrangement);
            if got < PAGE_SIZE {
                break;
            }
            page += 1;
            if page > 1000 {
                break;
            }
        }
        Ok(out)
    }

    pub async fn create_arrangement_key(
        &self,
        arrangement_id: &str,
        name: &str,
        key_starting: &str,
    ) -> Result<serde_json::Value, CliError> {
        let body = serde_json::json!({
            "arrangement_id": arrangement_id,
            "name": name,
            "key_starting": key_starting,
        });
        self.post("songs/keys/create", &body).await
    }

    pub async fn list_arrangement_keys(
        &self,
        arrangement_id: &str,
    ) -> Result<Vec<crate::api::raw::RawKey>, CliError> {
        let resp: crate::api::raw::KeysResponse = self
            .post(
                "songs/keys/getAll",
                &serde_json::json!({
                    "arrangement_id": arrangement_id,
                    "page_size": 100,
                }),
            )
            .await?;
        Ok(resp.keys.key)
    }

    pub async fn get_arrangement_info(
        &self,
        arrangement_id: &str,
        chord_chart_key: Option<&str>,
    ) -> Result<crate::api::raw::RawArrangement, CliError> {
        let mut body = serde_json::json!({ "id": arrangement_id });
        if let Some(k) = chord_chart_key {
            body["chord_chart_key"] = serde_json::Value::String(k.to_string());
        }
        let resp: crate::api::raw::ArrangementInfoResponse =
            self.post("songs/arrangements/getInfo", &body).await?;
        Ok(resp.arrangement)
    }

    pub async fn list_services(
        &self,
        date_from: &str,
        date_to: &str,
    ) -> Result<Vec<crate::api::raw::RawService>, CliError> {
        const SERVICES_PAGE_SIZE: u32 = 100;
        let mut out = Vec::new();
        let mut page: u32 = 1;
        loop {
            let resp: crate::api::raw::ServicesResponse = self
                .post(
                    "services/getAll",
                    &serde_json::json!({
                        "page": page,
                        "page_size": SERVICES_PAGE_SIZE,
                        "start": date_from,
                        "end": date_to,
                    }),
                )
                .await?;
            let got = resp.services.service.len() as u32;
            out.extend(resp.services.service);
            let per_page = if resp.services.per_page == 0 {
                SERVICES_PAGE_SIZE
            } else {
                resp.services.per_page
            };
            if got < per_page
                || (resp.services.total > 0 && out.len() as u32 >= resp.services.total)
            {
                break;
            }
            page += 1;
            if page > 1000 {
                break;
            }
        }
        Ok(out)
    }

    /// Fetch all people with the requested `fields` expansions. Paginates at page_size=1000.
    pub async fn list_all_people(
        &self,
        fields: &[&str],
    ) -> Result<Vec<crate::api::raw::RawPersonRecord>, CliError> {
        const PEOPLE_PAGE_SIZE: u32 = 1000;
        let mut out = Vec::new();
        let mut page: u32 = 1;
        loop {
            let body = serde_json::json!({
                "page": page,
                "page_size": PEOPLE_PAGE_SIZE,
                "fields": fields,
            });
            let resp: crate::api::raw::PeopleResponse = self.post("people/getAll", &body).await?;
            let got = resp.people.person.len() as u32;
            out.extend(resp.people.person);
            let per_page = if resp.people.per_page == 0 {
                PEOPLE_PAGE_SIZE
            } else {
                resp.people.per_page
            };
            if got < per_page || (resp.people.total > 0 && out.len() as u32 >= resp.people.total) {
                break;
            }
            page += 1;
            if page > 1000 {
                break;
            }
        }
        Ok(out)
    }

    /// Fetch all people's id+email as a HashMap. Paginates at page_size=1000.
    pub async fn list_people_emails(
        &self,
    ) -> Result<std::collections::HashMap<String, String>, CliError> {
        const PEOPLE_PAGE_SIZE: u32 = 1000;
        let mut out = std::collections::HashMap::new();
        let mut page: u32 = 1;
        loop {
            let resp: crate::api::raw::PeopleResponse = self
                .post(
                    "people/getAll",
                    &serde_json::json!({
                        "page": page,
                        "page_size": PEOPLE_PAGE_SIZE,
                    }),
                )
                .await?;
            let got = resp.people.person.len() as u32;
            for p in resp.people.person {
                if !p.email.is_empty() {
                    out.insert(p.id, p.email);
                }
            }
            let per_page = if resp.people.per_page == 0 {
                PEOPLE_PAGE_SIZE
            } else {
                resp.people.per_page
            };
            if got < per_page || (resp.people.total > 0 && out.len() as u32 >= resp.people.total) {
                break;
            }
            page += 1;
            if page > 1000 {
                break;
            }
        }
        Ok(out)
    }

    #[allow(dead_code)]
    pub async fn get_service_info(
        &self,
        id: &str,
        fields: &[&str],
    ) -> Result<crate::api::raw::RawService, CliError> {
        let body = if fields.is_empty() {
            serde_json::json!({ "id": id })
        } else {
            serde_json::json!({ "id": id, "fields": fields })
        };
        let resp: crate::api::raw::ServiceInfoResponse =
            self.post("services/getInfo", &body).await?;
        resp.service
            .into_iter()
            .next()
            .ok_or_else(|| CliError::NotFound(format!("service {id}")))
    }

    /// Fetch services with songs, plans, and volunteers for analysis.
    pub async fn list_services_with_details(
        &self,
        date_from: &str,
        date_to: &str,
    ) -> Result<Vec<crate::api::raw::RawService>, CliError> {
        const SERVICES_PAGE_SIZE: u32 = 100;
        let mut out = Vec::new();
        let mut page: u32 = 1;
        loop {
            let resp: crate::api::raw::ServicesResponse = self
                .post(
                    "services/getAll",
                    &serde_json::json!({
                        "page": page,
                        "page_size": SERVICES_PAGE_SIZE,
                        "start": date_from,
                        "end": date_to,
                        "fields": ["songs", "plans", "volunteers"],
                    }),
                )
                .await?;
            let got = resp.services.service.len() as u32;
            out.extend(resp.services.service);
            let per_page = if resp.services.per_page == 0 {
                SERVICES_PAGE_SIZE
            } else {
                resp.services.per_page
            };
            if got < per_page
                || (resp.services.total > 0 && out.len() as u32 >= resp.services.total)
            {
                break;
            }
            page += 1;
            if page > 1000 {
                break;
            }
        }
        Ok(out)
    }
}
