mod common;
use common::{bin, mock_server};
use predicates::prelude::*;
use predicates::str::contains;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

fn ok_body() -> serde_json::Value {
    serde_json::json!({
        "status": "ok",
        "categories": {
            "category": [
                { "id": "c1", "name": "Worship" },
                { "id": "c2", "name": "Hymns" }
            ]
        }
    })
}

#[tokio::test]
async fn text_output() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/categories/getAll.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok_body()))
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "categories"])
        .assert()
        .success()
        .stdout(contains("c1 | Worship").and(contains("c2 | Hymns")));
}

#[tokio::test]
async fn json_output() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/categories/getAll.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok_body()))
        .mount(&server)
        .await;

    let out = bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "categories", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let parsed: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(parsed[0]["id"], "c1");
    assert_eq!(parsed[1]["name"], "Hymns");
}
