mod common;
use common::{bin, mock_server};
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

#[tokio::test]
async fn paginates_and_filters_active_in_text_mode() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getAll.json"))
        .and(body_partial_json(serde_json::json!({ "page": 1 })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page(
                1,
                100,
                150,
                (0..100)
                    .map(|i| {
                        song(
                            &format!("s{i}"),
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
        .and(body_partial_json(serde_json::json!({ "page": 2 })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(page(
                2,
                50,
                150,
                (100..150)
                    .map(|i| {
                        song(
                            &format!("s{i}"),
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
async fn album_and_ccli_columns() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getAll.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page(
            1,
            1,
            1,
            vec![song("s1", "Grace", "Trad.", "1", "Hymnal", "22025")],
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
async fn json_includes_inactive_songs() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getAll.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page(
            1,
            2,
            2,
            vec![
                song("s1", "Active", "A", "1", "", ""),
                song("s2", "Archived", "B", "0", "", ""),
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
