use crate::api::Client;
use crate::cli::SongsListArgs;
use crate::date_window::parse_duration_start;
use crate::domain::category::{self, category_tree, Category};
use crate::domain::song::SongSummary;
use crate::error::CliError;
use crate::output;
use crate::resolve;
use chrono::{Local, NaiveDate};
use std::collections::{HashMap, HashSet};

pub async fn run(client: &Client, args: SongsListArgs) -> Result<(), CliError> {
    let SongsListArgs {
        json,
        album,
        ccli,
        category_ids,
        id_mode,
        last_used,
        count,
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
    let has_service_usage_filter = used_start.is_some() || not_used_start.is_some();
    let show_last_used = last_used || has_service_usage_filter;
    // Track counts whenever we fetch service data (for filters), but only
    // show the column when explicitly requested.
    let show_count = count;
    let track_counts = count || has_service_usage_filter;
    let needs_service_data = show_last_used || track_counts;

    let mut raws = client.list_all_songs().await?;
    if !category_ids.is_empty() {
        // Fetch the full categories list so the resolver can give accurate
        // ambiguous/no-match errors even for categories with no active songs.
        let cats: Vec<Category> = client
            .list_categories()
            .await?
            .into_iter()
            .map(Into::into)
            .collect();
        let tree = category_tree(&cats);
        let resolved: HashSet<String> = resolve::resolve_all(&category_ids, &tree)?
            .into_iter()
            .collect();
        raws.retain(|song| {
            song.categories
                .category
                .iter()
                .any(|c| resolved.iter().any(|id| category::id_matches(&c.id, id)))
        });
    }

    let mut last_used_by_song = HashMap::new();
    let mut count_by_song: HashMap<String, usize> = HashMap::new();
    if needs_service_data {
        let service_from = if has_service_usage_filter {
            [used_start, not_used_start]
                .into_iter()
                .flatten()
                .min()
                .unwrap_or(today)
        } else {
            parse_duration_start("1y", today, "--last-used")?
        };
        let service_from_str = service_from.format("%Y-%m-%d").to_string();
        let today_str = today.format("%Y-%m-%d").to_string();
        let services = client
            .list_services_with_details(&service_from_str, &today_str)
            .await?;

        for service in &services {
            let Some(date) = service_date(&service.date) else {
                continue;
            };
            for song_id in service.song_ids() {
                let key = song_id.to_string();
                last_used_by_song
                    .entry(key.clone())
                    .and_modify(|last: &mut NaiveDate| {
                        if date > *last {
                            *last = date;
                        }
                    })
                    .or_insert(date);
                *count_by_song.entry(key).or_insert(0) += 1;
            }
        }

        if let Some(start) = used_start {
            raws.retain(|song| {
                last_used_by_song
                    .get(&song.id)
                    .is_some_and(|last| *last >= start)
            });
        }
        if let Some(start) = not_used_start {
            raws.retain(|song| {
                last_used_by_song
                    .get(&song.id)
                    .is_none_or(|last| *last < start)
            });
        }
    }

    let mut all: Vec<SongSummary> = raws
        .into_iter()
        .map(|raw| {
            let mut song: SongSummary = raw.into();
            if show_last_used {
                song.last_used = last_used_by_song
                    .get(&song.id)
                    .map(|date| date.format("%Y-%m-%d").to_string());
            }
            if track_counts {
                song.count = count_by_song.get(&song.id).copied();
            }
            song
        })
        .collect();

    // --last-used (explicit) sorts most-recent-first; songs never used go to the end.
    // Date strings are YYYY-MM-DD so lexicographic order = chronological order.
    if last_used {
        all.sort_by(|a, b| b.last_used.cmp(&a.last_used));
    }
    // --count sorts most-used-first; songs never used go to the end.
    if count {
        all.sort_by(|a, b| match (&a.count, &b.count) {
            (None, None) => std::cmp::Ordering::Equal,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (Some(_), None) => std::cmp::Ordering::Less,
            (Some(ac), Some(bc)) => bc.cmp(ac),
        });
    }

    let stdout = std::io::stdout();
    let mut lock = stdout.lock();

    let res = if json {
        output::json::write_pretty(&mut lock, &all)
    } else {
        let active: Vec<SongSummary> = all.into_iter().filter(|s| s.status == "active").collect();
        output::text::write_songs(&mut lock, &active, album, ccli, id_mode, show_last_used, show_count)
    };
    res.map_err(|e| CliError::Io(format!("write error: {e}")))
}

fn service_date(value: &str) -> Option<NaiveDate> {
    let date = value.get(..10).unwrap_or(value);
    NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()
}
