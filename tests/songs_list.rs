mod common;
use chrono::{Duration, Local, Months, NaiveDate};
use common::{bin, mock_server};
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, ResponseTemplate};

fn page(
    page: u32,
    on_this_page: u32,
    total: u32,
    songs: Vec<serde_json::Value>,
) -> serde_json::Value {
    serde_json::json!({
        "status": "ok",
        "songs": {
            "page": page,
            "per_page": 100,
            "total": total,
            "on_this_page": on_this_page,
            "song": songs
        }
    })
}

fn service_usage_page(services: Vec<serde_json::Value>) -> serde_json::Value {
    let count = services.len() as u64;
    serde_json::json!({
        "status": "ok",
        "services": {
            "page": 1,
            "per_page": 100,
            "total": count,
            "on_this_page": count,
            "service": services
        }
    })
}

fn service_with_usage(
    id: &str,
    date: NaiveDate,
    sidebar_song_ids: &[&str],
    plan_song_ids: &[&str],
) -> serde_json::Value {
    let sidebar_songs: Vec<serde_json::Value> = sidebar_song_ids
        .iter()
        .map(|id| serde_json::json!({ "id": id }))
        .collect();
    let plan_items: Vec<serde_json::Value> = plan_song_ids
        .iter()
        .map(|id| serde_json::json!({ "song": { "id": id } }))
        .collect();

    serde_json::json!({
        "id": id,
        "date": format!("{date} 09:30:00"),
        "name": "Sunday Morning",
        "status": "Published",
        "service_type": { "id": "st-1", "name": "Sunday Service" },
        "location": { "id": "loc-1", "name": "Main" },
        "description": "",
        "songs": { "song": sidebar_songs },
        "plans": {
            "plan": [
                {
                    "items": {
                        "item": plan_items
                    }
                }
            ]
        }
    })
}

fn song(
    id: &str,
    title: &str,
    artist: &str,
    status: &str,
    album: &str,
    number: &str,
) -> serde_json::Value {
    serde_json::json!({
        "id": id, "title": title, "artist": artist,
        "album": album, "number": number, "status": status,
    })
}

fn song_with_categories(
    id: &str,
    title: &str,
    artist: &str,
    status: &str,
    categories: &[(&str, &str)],
) -> serde_json::Value {
    let categories: Vec<serde_json::Value> = categories
        .iter()
        .map(|(id, name)| serde_json::json!({ "id": id, "name": name }))
        .collect();

    serde_json::json!({
        "id": id,
        "title": title,
        "artist": artist,
        "album": "",
        "number": "",
        "status": status,
        "categories": { "category": categories }
    })
}

#[tokio::test]
async fn paginates_and_filters_active_in_text_mode() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getAll.json"))
        .and(body_partial_json(serde_json::json!({
            "item": 0,
            "page": 1,
            "page_size": 100
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page(
                1,
                100,
                150,
                (0..100)
                    .map(|i| {
                        song(
                            &format!("s{i}-0000-0000-0000-000000000000"),
                            &format!("T{i}"),
                            "A",
                            "1",
                            "Al",
                            &format!("{i}"),
                        )
                    })
                    .collect(),
            )),
        )
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/songs/getAll.json"))
        .and(body_partial_json(serde_json::json!({
            "item": 0,
            "page": 2,
            "page_size": 100
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page(
                2,
                50,
                150,
                (100..150)
                    .map(|i| {
                        song(
                            &format!("s{i}-0000-0000-0000-000000000000"),
                            &format!("T{i}"),
                            "A",
                            if i % 2 == 0 { "1" } else { "0" },
                            "Al",
                            &format!("{i}"),
                        )
                    })
                    .collect(),
            )),
        )
        .mount(&server)
        .await;

    let out = bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(out).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    // 100 active on page 1 + 25 active on page 2 (even indices 100..150)
    assert_eq!(lines.len(), 125);
    assert!(lines[0].contains("s0 | T0 | A"));
}

#[tokio::test]
async fn category_id_filter_matches_any_repeated_id() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getAll.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page(
            1,
            4,
            4,
            vec![
                song_with_categories(
                    "s1-0000-0000-0000-000000000000",
                    "Praise Match",
                    "A",
                    "1",
                    &[("02b06b47-c275-11e6-aad3-0219ad55c99b", "Praise")],
                ),
                song_with_categories(
                    "s2-0000-0000-0000-000000000000",
                    "Hymn Match",
                    "B",
                    "1",
                    &[("90aee036-d5b7-11e5-aba7-06fb5fa8f77d", "Hymns")],
                ),
                song_with_categories(
                    "s3-0000-0000-0000-000000000000",
                    "No Match",
                    "C",
                    "1",
                    &[("63b140bf-acb2-49b3-921a-6a507263bf6d", "Seasonal")],
                ),
                song_with_categories(
                    "s4-0000-0000-0000-000000000000",
                    "Uncategorized",
                    "D",
                    "1",
                    &[],
                ),
            ],
        )))
        .mount(&server)
        .await;

    let out = bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args([
            "songs",
            "list",
            "--category-id",
            "02b06b47",
            "--category-id",
            "90aee036-d5b7-11e5-aba7-06fb5fa8f77d",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(out).unwrap();
    let lines: Vec<&str> = text.lines().collect();

    assert_eq!(lines.len(), 2);
    assert!(text.contains("s1 | Praise Match | A"));
    assert!(text.contains("s2 | Hymn Match | B"));
    assert!(!text.contains("No Match"));
    assert!(!text.contains("Uncategorized"));
}

#[tokio::test]
async fn category_id_filter_applies_to_json_without_active_filter() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getAll.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page(
            1,
            2,
            2,
            vec![
                song_with_categories(
                    "s1-0000-0000-0000-000000000000",
                    "Active",
                    "A",
                    "1",
                    &[("c1", "Praise")],
                ),
                song_with_categories(
                    "s2-0000-0000-0000-000000000000",
                    "Archived",
                    "B",
                    "0",
                    &[("c1", "Praise")],
                ),
            ],
        )))
        .mount(&server)
        .await;

    let out = bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "list", "--json", "--category-id", "c1"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let parsed: serde_json::Value = serde_json::from_slice(&out).unwrap();

    assert_eq!(parsed.as_array().unwrap().len(), 2);
    assert_eq!(parsed[0]["status"], "active");
    assert_eq!(parsed[1]["status"], "archived");
}

#[tokio::test]
async fn service_usage_filters_include_and_exclude_windows() {
    let server = mock_server().await;
    let today = Local::now().date_naive();
    let used_start = today - Duration::days(30);
    let recent = today - Duration::days(1);
    let older = today - Duration::days(10);

    Mock::given(method("POST"))
        .and(path("/songs/getAll.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page(
            1,
            3,
            3,
            vec![
                song("keep-0000-0000-0000-000000000000", "Keep", "A", "1", "", ""),
                song(
                    "recent-0000-0000-0000-000000000000",
                    "Recent",
                    "B",
                    "1",
                    "",
                    "",
                ),
                song(
                    "never-0000-0000-0000-000000000000",
                    "Never",
                    "C",
                    "1",
                    "",
                    "",
                ),
            ],
        )))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/services/getAll.json"))
        .and(body_partial_json(serde_json::json!({
            "start": used_start.format("%Y-%m-%d").to_string(),
            "end": today.format("%Y-%m-%d").to_string(),
            "fields": ["songs", "plans"]
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(service_usage_page(vec![
                service_with_usage("svc-old", older, &[], &["keep-0000-0000-0000-000000000000"]),
                service_with_usage(
                    "svc-recent",
                    recent,
                    &["recent-0000-0000-0000-000000000000"],
                    &[],
                ),
            ])),
        )
        .mount(&server)
        .await;

    let out = bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args([
            "songs",
            "list",
            "--used-within",
            "30d",
            "--not-used-within",
            "7d",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(out).unwrap();

    assert!(text.contains(&format!("keep | Keep | A | {}", older.format("%Y-%m-%d"))));
    assert!(!text.contains("Recent"));
    assert!(!text.contains("Never"));
}

#[tokio::test]
async fn last_used_flag_adds_column_without_filtering() {
    let server = mock_server().await;
    let today = Local::now().date_naive();
    let start = today.checked_sub_months(Months::new(12)).unwrap();
    let older = today - Duration::days(40);
    let recent = today - Duration::days(3);

    Mock::given(method("POST"))
        .and(path("/songs/getAll.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page(
            1,
            3,
            3,
            vec![
                song("old-0000-0000-0000-000000000000", "Old", "A", "1", "", ""),
                song(
                    "recent-0000-0000-0000-000000000000",
                    "Recent",
                    "B",
                    "1",
                    "",
                    "",
                ),
                song(
                    "never-0000-0000-0000-000000000000",
                    "Never",
                    "C",
                    "1",
                    "",
                    "",
                ),
            ],
        )))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/services/getAll.json"))
        .and(body_partial_json(serde_json::json!({
            "start": start.format("%Y-%m-%d").to_string(),
            "end": today.format("%Y-%m-%d").to_string(),
            "fields": ["songs", "plans"]
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(service_usage_page(vec![
                service_with_usage("svc-old", older, &["old-0000-0000-0000-000000000000"], &[]),
                service_with_usage(
                    "svc-recent",
                    recent,
                    &[],
                    &["recent-0000-0000-0000-000000000000"],
                ),
            ])),
        )
        .mount(&server)
        .await;

    let out = bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "list", "--last-used"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(out).unwrap();

    assert!(text.contains(&format!("old | Old | A | {}", older.format("%Y-%m-%d"))));
    assert!(text.contains(&format!(
        "recent | Recent | B | {}",
        recent.format("%Y-%m-%d")
    )));
    assert!(text.contains("never | Never | C | -"));

    // --last-used sorts most-recent-first; never-used goes to the end.
    let lines: Vec<&str> = text.lines().collect();
    let recent_idx = lines.iter().position(|l| l.contains("Recent")).unwrap();
    let old_idx = lines.iter().position(|l| l.contains("Old")).unwrap();
    let never_idx = lines.iter().position(|l| l.contains("Never")).unwrap();
    assert!(recent_idx < old_idx, "recent should come before old");
    assert!(old_idx < never_idx, "old should come before never");
}

#[tokio::test]
async fn invalid_service_usage_duration_is_usage_error() {
    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", "http://127.0.0.1:1")
        .args(["songs", "list", "--used-within", "soon"])
        .assert()
        .failure()
        .code(2)
        .stderr(contains("invalid --used-within").and(contains("14d")));
}

#[tokio::test]
async fn album_and_ccli_columns() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getAll.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page(
            1,
            1,
            1,
            vec![song(
                "s1-0000-0000-0000-000000000000",
                "Grace",
                "Trad.",
                "1",
                "Hymnal",
                "22025",
            )],
        )))
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "list", "--album", "--ccli"])
        .assert()
        .success()
        .stdout(contains("s1 | Grace | Trad. | Hymnal | 22025"));
}

#[tokio::test]
async fn full_id_flag_shows_full_song_ids_in_text_output() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getAll.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page(
            1,
            1,
            1,
            vec![song(
                "s1-0000-0000-0000-000000000000",
                "Grace",
                "Trad.",
                "1",
                "",
                "",
            )],
        )))
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "list", "--full-id"])
        .assert()
        .success()
        .stdout(contains("s1-0000-0000-0000-000000000000 | Grace | Trad."));
}

#[tokio::test]
async fn json_includes_inactive_songs() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getAll.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page(
            1,
            2,
            2,
            vec![
                song("s1-0000-0000-0000-000000000000", "Active", "A", "1", "", ""),
                song(
                    "s2-0000-0000-0000-000000000000",
                    "Archived",
                    "B",
                    "0",
                    "",
                    "",
                ),
            ],
        )))
        .mount(&server)
        .await;

    let out = bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "list", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let parsed: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(parsed.as_array().unwrap().len(), 2);
    assert_eq!(parsed[1]["status"], "archived");
}
