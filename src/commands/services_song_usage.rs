use crate::api::raw::RawService;
use crate::api::Client;
use crate::cli::ServicesSongUsageArgs;
use crate::date_window::parse_date;
use crate::domain::service::volunteer_rows;
use crate::error::CliError;
use chrono::Local;
use serde_json::json;
use std::collections::{BTreeMap, HashMap, HashSet};

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

    // song_id -> uses (date, leader, key)
    let mut song_usage: BTreeMap<String, SongUsage> = BTreeMap::new();

    for svc in &services {
        let leader = find_worship_leader(svc);

        for use_ in svc.song_uses() {
            song_usage
                .entry(use_.id)
                .or_default()
                .uses
                .push(UseRecord {
                    date: svc.date.chars().take(10).collect(),
                    leader: leader.clone(),
                    key: use_.key,
                });
        }
    }

    // We need song titles/artists for output. One batch call.
    let all_songs = client.list_all_songs().await?;
    let song_titles: HashMap<&str, &str> = all_songs
        .iter()
        .map(|s| (s.id.as_str(), s.title.as_str()))
        .collect();
    let song_artists: HashMap<&str, &str> = all_songs
        .iter()
        .map(|s| (s.id.as_str(), s.artist.as_str()))
        .collect();

    if args.json {
        // Machine-readable full dump: no max-uses / one-leader filtering.
        let mut out: Vec<serde_json::Value> = song_usage
            .iter()
            .map(|(id, usage)| {
                json!({
                    "song_id": id,
                    "title": song_titles.get(id.as_str()).copied().unwrap_or("<unknown>"),
                    "artist": song_artists.get(id.as_str()).copied().unwrap_or(""),
                    "uses": usage.uses.iter().map(|u| json!({
                        "date": u.date,
                        "leader": u.leader,
                        "key": u.key,
                    })).collect::<Vec<_>>(),
                })
            })
            .collect();
        out.sort_by(|a, b| {
            a["title"]
                .as_str()
                .unwrap_or("")
                .cmp(b["title"].as_str().unwrap_or(""))
        });
        println!("{}", serde_json::to_string_pretty(&out).map_err(|e| CliError::Io(format!("json: {e}")))?);
        return Ok(());
    }

    // Filter for the text analysis view
    let mut filtered: Vec<(&String, &SongUsage)> = song_usage
        .iter()
        .filter(|(_, u)| {
            let under_max = u.uses.len() as u32 <= args.max_uses;
            let one_leader = args.one_leader
                && {
                    let leaders: HashSet<&str> = u
                        .uses
                        .iter()
                        .filter_map(|r| {
                            if r.leader.is_empty() {
                                None
                            } else {
                                Some(r.leader.as_str())
                            }
                        })
                        .collect();
                    leaders.len() == 1
                };
            under_max || one_leader
        })
        .collect();
    filtered.sort_by_key(|(_, u)| u.uses.len());

    if filtered.is_empty() {
        println!("No songs found matching the criteria.");
        return Ok(());
    }

    let heading = if args.one_leader {
        format!(
            "Songs sung ≤ {} times or led by only one person:\n",
            args.max_uses
        )
    } else {
        format!(
            "Songs sung ≤ {} times in the last 12 months:\n",
            args.max_uses
        )
    };
    println!("{heading}");

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
            let key_part = match &u.key {
                Some(k) if !k.is_empty() => format!(" ({})", k),
                _ => String::new(),
            };
            println!("  {} — {}{}", u.date, leader, key_part);
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
    key: Option<String>,
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
