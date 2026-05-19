use crate::api::Client;
use crate::cli::SongsListArgs;
use crate::date_window::parse_duration_start;
use crate::domain::category;
use crate::domain::song::SongSummary;
use crate::error::CliError;
use crate::output;
use chrono::{Local, NaiveDate};
use std::collections::HashSet;

pub async fn run(client: &Client, args: SongsListArgs) -> Result<(), CliError> {
    let SongsListArgs {
        json,
        album,
        ccli,
        category_ids,
        full_id,
        used_within,
        not_used_within,
    } = args;

    let today = Local::now().date_naive();
    let used_start = used_within
        .as_deref()
        .map(|duration| parse_duration_start(duration, today, "--used-within"))
        .transpose()?;
    let not_used_start = not_used_within
        .as_deref()
        .map(|duration| parse_duration_start(duration, today, "--not-used-within"))
        .transpose()?;

    let mut raws = client.list_all_songs().await?;
    if !category_ids.is_empty() {
        let wanted: HashSet<&str> = category_ids.iter().map(String::as_str).collect();
        raws.retain(|song| {
            song.categories.category.iter().any(|category| {
                wanted
                    .iter()
                    .any(|id| category::id_matches(&category.id, id))
            })
        });
    }

    if used_start.is_some() || not_used_start.is_some() {
        let service_from = [used_start, not_used_start]
            .into_iter()
            .flatten()
            .min()
            .unwrap_or(today);
        let service_from_str = service_from.format("%Y-%m-%d").to_string();
        let today_str = today.format("%Y-%m-%d").to_string();
        let services = client
            .list_services_with_song_usage(&service_from_str, &today_str)
            .await?;

        let mut used_song_ids = HashSet::new();
        let mut recently_used_song_ids = HashSet::new();
        for service in &services {
            let Some(date) = service_date(&service.date) else {
                continue;
            };
            let song_ids = service.song_ids();
            if used_start.is_some_and(|start| date >= start) {
                used_song_ids.extend(song_ids.iter().map(|id| (*id).to_string()));
            }
            if not_used_start.is_some_and(|start| date >= start) {
                recently_used_song_ids.extend(song_ids.iter().map(|id| (*id).to_string()));
            }
        }

        if used_start.is_some() {
            raws.retain(|song| used_song_ids.contains(&song.id));
        }
        if not_used_start.is_some() {
            raws.retain(|song| !recently_used_song_ids.contains(&song.id));
        }
    }

    let all: Vec<SongSummary> = raws.into_iter().map(Into::into).collect();

    let stdout = std::io::stdout();
    let mut lock = stdout.lock();

    let res = if json {
        output::json::write_pretty(&mut lock, &all)
    } else {
        let active: Vec<SongSummary> = all.into_iter().filter(|s| s.status == "active").collect();
        output::text::write_songs(&mut lock, &active, album, ccli, full_id)
    };
    res.map_err(|e| CliError::Io(format!("write error: {e}")))
}

fn service_date(value: &str) -> Option<NaiveDate> {
    let date = value.get(..10).unwrap_or(value);
    NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()
}
