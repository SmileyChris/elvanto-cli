use crate::api::raw::RawService;
use crate::api::Client;
use crate::cli::ServicesSongUsageArgs;
use crate::date_window::parse_date;
use crate::domain::service::volunteer_rows;
use crate::error::CliError;
use chrono::Local;
use std::collections::{BTreeMap, HashMap};

pub async fn run(client: &Client, args: ServicesSongUsageArgs) -> Result<(), CliError> {
    let today = Local::now().date_naive();
    let to = match args.to.as_deref() {
        Some(s) => parse_date(s, "--to")?,
        None => today,
    };
    let from = match args.from.as_deref() {
        Some(s) => parse_date(s, "--from")?,
        None => to
            .checked_sub_months(chrono::Months::new(12))
            .unwrap_or(to),
    };

    eprintln!(
        "Fetching services from {} to {}…",
        from.format("%Y-%m-%d"),
        to.format("%Y-%m-%d")
    );

    let from_str = from.format("%Y-%m-%d").to_string();
    let to_str = to.format("%Y-%m-%d").to_string();

    let services: Vec<RawService> = client
        .list_services_with_details(&from_str, &to_str)
        .await?;

    eprintln!("Got {} services.", services.len());

    // song_id -> Vec<(service_date, leader_name)>
    let mut song_usage: BTreeMap<String, SongUsage> = BTreeMap::new();

    for svc in &services {
        let leader = find_worship_leader(svc);

        for song_id in svc.song_ids() {
            song_usage
                .entry(song_id.to_string())
                .or_default()
                .uses
                .push(UseRecord {
                    date: svc.date.chars().take(10).collect(),
                    leader: leader.clone(),
                });
        }
    }

    // Filter to songs with max_uses or fewer
    let mut filtered: Vec<(&String, &SongUsage)> = song_usage
        .iter()
        .filter(|(_, u)| u.uses.len() as u32 <= args.max_uses)
        .collect();
    filtered.sort_by_key(|(_, u)| u.uses.len());

    if filtered.is_empty() {
        println!(
            "No songs found with ≤ {} uses in the date range.",
            args.max_uses
        );
        return Ok(());
    }

    // Fetch song details
    eprintln!(
        "Fetching details for {} songs…",
        filtered.len()
    );

    // We need to look up song titles. Let's do it in batches via the songs list
    let all_songs = client.list_all_songs().await?;
    let song_titles: HashMap<&str, &str> = all_songs
        .iter()
        .map(|s| (s.id.as_str(), s.title.as_str()))
        .collect();
    let song_artists: HashMap<&str, &str> = all_songs
        .iter()
        .map(|s| (s.id.as_str(), s.artist.as_str()))
        .collect();

    println!(
        "Songs sung ≤ {} times in the last 12 months:\n",
        args.max_uses
    );

    for (id, usage) in &filtered {
        let title = *song_titles.get(id.as_str()).unwrap_or(&"<unknown>");
        let artist = song_artists.get(id.as_str()).unwrap_or(&"");
        let artist_part = if artist.is_empty() {
            String::new()
        } else {
            format!(" by {}", artist)
        };
        let times = match usage.uses.len() {
            1 => "once".to_string(),
            2 => "twice".to_string(),
            n => format!("{} times", n),
        };

        println!("\"{}\"{} — sung {}", title, artist_part, times);
        for u in &usage.uses {
            let leader = if u.leader.is_empty() {
                "(unknown leader)".to_string()
            } else {
                format!("Led by {}", u.leader)
            };
            println!("  {} — {}", u.date, leader);
        }
        println!();
    }

    println!("{} songs total.", filtered.len());

    Ok(())
}

#[derive(Debug, Default)]
struct SongUsage {
    uses: Vec<UseRecord>,
}

#[derive(Debug)]
struct UseRecord {
    date: String,
    leader: String,
}

fn find_worship_leader(svc: &RawService) -> String {
    let rows = volunteer_rows(svc);
    for row in &rows {
        if row.position.to_lowercase().contains("worship leader") {
            return row.name.clone().unwrap_or_default();
        }
    }
    String::new()
}
