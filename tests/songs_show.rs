mod common;
use common::{bin, mock_server};
use predicates::prelude::*;
use predicates::str::contains;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, ResponseTemplate};

fn song_body(extra: serde_json::Value) -> serde_json::Value {
    let mut song = serde_json::json!({
        "id": "s-1",
        "title": "Amazing Grace",
        "artist": "Trad.",
        "album": "Hymnal",
        "number": "22025",
        "status": "1",
        "sequence": "V1 C V2 C",
        "bpm": "78",
        "duration": "180",
        "learn": "1",
        "allow_downloads": "0",
        "categories": { "category": [ { "id": "c1", "name": "Hymns" } ] },
        "locations": { "location": [ { "id": "l1", "name": "Main" } ] },
        "arrangements": {
            "arrangement": [
                {
                    "id": "a1",
                    "name": "Default",
                    "sequence": "V1 C V2 C",
                    "bpm": "78",
                    "duration": "180",
                    "lyrics": "Amazing grace how sweet the sound\nThat saved a wretch like me",
                    "chord_pro": "[G]Amazing [C]grace",
                    "keys": { "key": [ { "id": "k1", "starting": "G", "ending": "" } ] }
                }
            ]
        }
    });
    if let Some(obj) = song.as_object_mut() {
        if let Some(more) = extra.as_object() {
            for (k, v) in more {
                obj.insert(k.clone(), v.clone());
            }
        }
    }
    serde_json::json!({
        "status": "ok",
        "songs": { "song": [ song ] }
    })
}

#[tokio::test]
async fn curated_output_default() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getInfo.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(song_body(serde_json::json!({}))))
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "show", "s-1"])
        .assert()
        .success()
        .stdout(
            contains("Title:       Amazing Grace")
                .and(contains("CCLI number: 22025"))
                .and(contains("Status:      active"))
                .and(contains("First line:  Amazing grace how sweet the sound"))
                .and(contains("- Default [G]")),
        );
}

#[tokio::test]
async fn full_output() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getInfo.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(song_body(serde_json::json!({}))))
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "show", "s-1", "--full"])
        .assert()
        .success()
        .stdout(
            contains("BPM:             78")
                .and(contains("Categories:      Hymns"))
                .and(contains("Locations:       Main"))
                .and(contains("Learn:           true"))
                .and(contains("Allow downloads: false")),
        );
}

#[tokio::test]
async fn json_output_normalized() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getInfo.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(song_body(serde_json::json!({}))))
        .mount(&server)
        .await;

    let out = bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "show", "s-1", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ccli_number"], "22025");
    assert_eq!(v["status"], "active");
    assert_eq!(v["learn"], true);
    assert_eq!(v["allow_downloads"], false);
    assert_eq!(v["arrangements"][0]["name"], "Default");
    assert_eq!(v["arrangements"][0]["keys"][0]["starting"], "G");
    assert!(
        v.get("files").is_none(),
        "files omitted when --files not passed"
    );
}

#[tokio::test]
async fn files_flag_with_json_requests_files_and_includes_them() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getInfo.json"))
        .and(body_partial_json(
            serde_json::json!({ "id": "s-1", "files": 1 }),
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(song_body(serde_json::json!({
                "files": { "file": [ { "id": "f1", "filename": "lead.pdf" } ] }
            }))),
        )
        .mount(&server)
        .await;

    let out = bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "show", "s-1", "--json", "--files"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["files"]["file"][0]["filename"], "lead.pdf");
}

#[tokio::test]
async fn not_found_returns_exit_1() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getInfo.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "ok",
            "songs": { "song": [] }
        })))
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "show", "missing-id"])
        .assert()
        .failure()
        .code(1)
        .stderr(contains("not found: song missing-id"));
}
