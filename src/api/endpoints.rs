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
            let per_page = if resp.songs.per_page == 0 { SONGS_PAGE_SIZE } else { resp.songs.per_page };
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
}
