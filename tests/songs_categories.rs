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
                { "id": "02b06b47-c275-11e6-aad3-0219ad55c99b", "name": "Carols" },
                { "id": "90aee036-d5b7-11e5-aba7-06fb5fa8f77d", "name": "Hymn" }
            ]
        }
    })
}

#[tokio::test]
async fn text_output_uses_short_ids_by_default() {
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
        .stdout(contains("02b06b47 | Carols").and(contains("90aee036 | Hymn")));
}

#[tokio::test]
async fn text_output_can_show_full_ids() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/categories/getAll.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok_body()))
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "categories", "--id", "long"])
        .assert()
        .success()
        .stdout(
            contains("02b06b47-c275-11e6-aad3-0219ad55c99b | Carols")
                .and(contains("90aee036-d5b7-11e5-aba7-06fb5fa8f77d | Hymn")),
        );
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
    assert_eq!(parsed[0]["id"], "02b06b47-c275-11e6-aad3-0219ad55c99b");
    assert_eq!(parsed[1]["name"], "Hymn");
}
