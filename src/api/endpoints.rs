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
        resp.songs
            .song
            .into_iter()
            .next()
            .ok_or_else(|| CliError::NotFound(format!("song {id}")))
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
}
