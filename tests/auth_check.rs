mod common;
use common::{bin, mock_server};
use predicates::prelude::*;
use predicates::str::contains;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

fn temp_dir(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "elvanto-cli-{label}-{}-{unique}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

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
async fn check_reads_api_key_from_dotenv() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/categories/getAll.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "ok",
            "categories": { "category": [] }
        })))
        .mount(&server)
        .await;

    let dir = temp_dir("dotenv-auth");
    std::fs::write(
        dir.join(".env"),
        format!(
            "ELVANTO_API_KEY=dotenvkey1\nELVANTO_BASE_URL={}\n",
            server.uri()
        ),
    )
    .unwrap();

    bin()
        .current_dir(dir)
        .env_remove("ELVANTO_API_KEY")
        .env_remove("ELVANTO_BASE_URL")
        .args(["auth", "check"])
        .assert()
        .success()
        .stdout(contains("auth: ok").and(contains("dote…key1")));
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
    let dir = temp_dir("no-dotenv-auth");

    bin()
        .current_dir(dir)
        .env_remove("ELVANTO_API_KEY")
        .env_remove("ELVANTO_BASE_URL")
        .args(["auth", "check"])
        .assert()
        .failure()
        .code(2)
        .stderr(contains("ELVANTO_API_KEY is not set"));
}
