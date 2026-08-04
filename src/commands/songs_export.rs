use crate::api::raw::RawArrangement;
use crate::api::Client;
use crate::domain::arrangement::Arrangement;
use crate::error::CliError;
use serde::Serialize;
use std::io::Write;

/// A flattened export row — one per arrangement per song.
#[derive(Debug, Serialize)]
pub struct ExportRow {
    pub song_id: String,
    pub song_title: String,
    pub song_artist: String,
    pub arrangement_id: String,
    pub arrangement_name: String,
    pub key_male: Option<String>,
    pub key_female: Option<String>,
    /// Starting keys from all key objects (comma-separated).
    pub starting_keys: String,
}

pub async fn run(client: &Client) -> Result<(), CliError> {
    let songs = client.list_all_songs().await?;
    let total = songs.len();
    eprintln!("Exporting key data for {total} songs from Elvanto...");

    let stdout = std::io::stdout();
    let mut lock = stdout.lock();

    for (i, song) in songs.iter().enumerate() {
        if (i + 1) % 50 == 0 || i == 0 || i == total - 1 {
            eprintln!("  [{}/{}] {}", i + 1, total, song.title);
        }

        let raw_arrs: Vec<RawArrangement> =
            match client.list_arrangements_for_song(&song.id).await {
                Ok(arrs) => arrs,
                Err(e) => {
                    eprintln!(
                        "  [warn] failed to fetch arrangements for {} ({}): {e}",
                        song.id, song.title
                    );
                    continue;
                }
            };

        let arrangements: Vec<Arrangement> =
            raw_arrs.into_iter().map(Into::into).collect();

        for arr in &arrangements {
            let mut key_male = arr.key_male.clone();
            let mut key_female = arr.key_female.clone();
            let mut male_added = String::new();
            let mut female_added = String::new();
            let mut all_starting: Vec<String> = arr.keys.iter().map(|k| k.starting.clone()).collect();

            if let Ok(keys) = client.list_arrangement_keys(&arr.id).await {
                // keys/getAll order is not date-based; when duplicates exist
                // (keys/create appends rather than replaces), the newest
                // record is the authoritative one. date_added is
                // "YYYY-MM-DD HH:MM:SS", so string comparison works.
                for k in &keys {
                    match k.name.as_str() {
                        "Male" if k.date_added > male_added => {
                            male_added = k.date_added.clone();
                            key_male = Some(k.starting.clone());
                        }
                        "Female" if k.date_added > female_added => {
                            female_added = k.date_added.clone();
                            key_female = Some(k.starting.clone());
                        }
                        _ => {}
                    }
                    if !all_starting.contains(&k.starting) {
                        all_starting.push(k.starting.clone());
                    }
                }
            }

            let row = ExportRow {
                song_id: song.id.clone(),
                song_title: song.title.clone(),
                song_artist: song.artist.clone(),
                arrangement_id: arr.id.clone(),
                arrangement_name: arr.name.clone(),
                key_male,
                key_female,
                starting_keys: all_starting.join(", "),
            };
            serde_json::to_writer(&mut lock, &row)
                .map_err(|e| CliError::Io(format!("write error: {e}")))?;
            writeln!(&mut lock)
                .map_err(|e| CliError::Io(format!("write error: {e}")))?;
        }
    }

    eprintln!("Done. {total} songs exported.");
    Ok(())
}
