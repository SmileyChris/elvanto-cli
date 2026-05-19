mod common;
use common::{bin, mock_server};
use predicates::prelude::*;
use predicates::str::contains;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn check_succeeds_with_valid_key() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/categories/getAll.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "ok",
            "categories": { "category": [] }
        })))
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["auth", "check"])
        .assert()
        .success()
        .stdout(contains("auth: ok").and(contains("abcd…ghij")));
}

#[tokio::test]
async fn check_fails_with_bad_key() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/categories/getAll.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "error",
            "error": { "code": 121, "message": "Invalid API Key." }
        })))
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "wrongkeywrong")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["auth", "check"])
        .assert()
        .failure()
        .code(1)
        .stderr(contains("Elvanto returned code 121"));
}

#[test]
fn check_fails_without_api_key() {
    bin()
        .env_remove("ELVANTO_API_KEY")
        .args(["auth", "check"])
        .assert()
        .failure()
        .code(2)
        .stderr(contains("ELVANTO_API_KEY is not set"));
}
