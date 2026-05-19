mod common;
use common::{bin, mock_server};
use predicates::str::contains;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

fn song_with_arrangements(arrs: Vec<serde_json::Value>) -> serde_json::Value {
    serde_json::json!({
        "status": "ok",
        "songs": { "song": [ {
            "id": "s1",
            "title": "Title",
            "status": "1",
            "arrangements": { "arrangement": arrs }
        } ] }
    })
}

fn arr(name: &str, lyrics: &str) -> serde_json::Value {
    serde_json::json!({
        "id": name, "name": name, "lyrics": lyrics,
        "keys": { "key": [] }
    })
}

#[tokio::test]
async fn picks_default_arrangement() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getInfo.json"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(song_with_arrangements(vec![
                arr("Acoustic", "Acoustic lyrics"),
                arr("Default", "Default lyrics"),
            ])),
        )
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "lyrics", "s1"])
        .assert()
        .success()
        .stdout(contains("Default lyrics"));
}

#[tokio::test]
async fn hints_other_arrangements_on_stderr() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getInfo.json"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(song_with_arrangements(vec![
                arr("Default", "Default lyrics"),
                arr("Acoustic", "Acoustic lyrics"),
                arr("Live", "Live lyrics"),
            ])),
        )
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "lyrics", "s1"])
        .assert()
        .success()
        .stderr(contains("other arrangements: Acoustic, Live"));
}

#[tokio::test]
async fn arrangement_override() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getInfo.json"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(song_with_arrangements(vec![
                arr("Default", "Default lyrics"),
                arr("Acoustic", "Acoustic lyrics"),
            ])),
        )
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "lyrics", "s1", "--arrangement", "Acoustic"])
        .assert()
        .success()
        .stdout(contains("Acoustic lyrics"));
}

#[tokio::test]
async fn missing_arrangement_is_usage_error() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getInfo.json"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(song_with_arrangements(vec![arr(
                "Default",
                "Default lyrics",
            )])),
        )
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "lyrics", "s1", "--arrangement", "Live"])
        .assert()
        .failure()
        .code(2)
        .stderr(contains("not found"));
}
