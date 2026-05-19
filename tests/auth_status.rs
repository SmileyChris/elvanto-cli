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
async fn status_succeeds_with_valid_env_key() {
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
        .args(["auth", "status"])
        .assert()
        .success()
        .stdout(
            contains("source: env (ELVANTO_API_KEY)")
                .and(contains("abcd…ghij"))
                .and(contains("status: ok")),
        );
}

#[tokio::test]
async fn status_reads_api_key_from_dotenv() {
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
        .args(["auth", "status"])
        .assert()
        .success()
        .stdout(contains("dote…key1").and(contains("status: ok")));
}

#[tokio::test]
async fn status_fails_with_bad_key() {
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
        .args(["auth", "status"])
        .assert()
        .failure()
        .stdout(contains("status: invalid"));
}

#[test]
fn status_reports_none_without_api_key() {
    let dir = temp_dir("no-dotenv-auth");

    bin()
        .current_dir(dir)
        .env_remove("ELVANTO_API_KEY")
        .env_remove("ELVANTO_BASE_URL")
        .args(["auth", "status"])
        .assert()
        .failure()
        .stdout(contains("source: none").and(contains("no API key")));
}
