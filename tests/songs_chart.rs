mod common;
use common::{bin, mock_server};
use predicates::str::contains;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, ResponseTemplate};

fn song_with_chart(name: &str, chord_chart: &str, starting_key: &str) -> serde_json::Value {
    serde_json::json!({
        "status": "ok",
        "songs": { "song": [ {
            "id": "s1",
            "title": "T",
            "status": "1",
            "arrangements": { "arrangement": [ {
                "id": "a1",
                "name": name,
                "chord_pro": chord_chart,
                "keys": { "key": [ { "id": "k1", "starting": starting_key, "ending": "" } ] }
            } ] }
        } ] }
    })
}

#[tokio::test]
async fn chart_without_transpose_uses_song_detail() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getInfo.json"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(song_with_chart("Default", "[G]Hello", "G")),
        )
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "chart", "s1"])
        .assert()
        .success()
        .stdout(contains("[G]Hello"));
}

#[tokio::test]
async fn chart_with_named_transpose_calls_arrangement_info() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getInfo.json"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(song_with_chart("Default", "[G]Hello", "G")),
        )
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/songs/arrangements/getInfo.json"))
        .and(body_partial_json(serde_json::json!({
            "id": "a1",
            "chord_chart_key": "A"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "ok",
            "arrangement": {
                "id": "a1",
                "name": "Default",
                "chord_pro": "[A]Hello",
                "keys": { "key": [] }
            }
        })))
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "chart", "s1", "--transpose", "A"])
        .assert()
        .success()
        .stdout(contains("[A]Hello"));
}

#[tokio::test]
async fn chart_with_offset_transpose_resolves_against_starting_key() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getInfo.json"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(song_with_chart("Default", "[G]Hello", "G")),
        )
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/songs/arrangements/getInfo.json"))
        .and(body_partial_json(serde_json::json!({
            "id": "a1",
            "chord_chart_key": "A"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "ok",
            "arrangement": {
                "id": "a1",
                "name": "Default",
                "chord_pro": "[A]Hello",
                "keys": { "key": [] }
            }
        })))
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "chart", "s1", "--transpose", "+2"])
        .assert()
        .success()
        .stdout(contains("[A]Hello"));
}

#[tokio::test]
async fn invalid_transpose_value_is_usage_error() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getInfo.json"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(song_with_chart("Default", "[G]Hello", "G")),
        )
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "chart", "s1", "--transpose", "Q"])
        .assert()
        .failure()
        .code(2)
        .stderr(contains("invalid key"));
}
