mod common;
use common::{mock_server, raw_bin};
use predicates::prelude::*;
use predicates::str::contains;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

fn ok_services_page() -> serde_json::Value {
    serde_json::json!({
        "status": "ok",
        "services": {
            "page": 1, "per_page": 100, "total": 1, "on_this_page": 1,
            "service": [{
                "id": "1eb01e76-7a5d-4d02-a207-d75055645f14",
                "date": "2026-05-23 22:00:00",
                "name": "Sunday",
                "status": "Published",
                "service_type": { "id": "st-1", "name": "Sunday Service" },
                "location": { "id": "", "name": "" }
            }]
        }
    })
}

#[tokio::test]
async fn env_injects_flags_for_services_list() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/services/getAll.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok_services_page()))
        .mount(&server)
        .await;

    // No CLI flag: env should inject --id long, and stderr should note it.
    raw_bin()
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .env("ELVANTO_SERVICES_LIST", "--id long")
        .args(["services", "list"])
        .assert()
        .success()
        .stdout(contains(
            "1eb01e76-7a5d-4d02-a207-d75055645f14 | 2026-05-23",
        ))
        .stderr(contains("applied ELVANTO_SERVICES_LIST defaults").and(contains("--id long")));
}

#[tokio::test]
async fn manual_flag_suppresses_env_and_emits_note() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/services/getAll.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok_services_page()))
        .mount(&server)
        .await;

    // Manual --json suppresses the env --id long; stderr should say so.
    raw_bin()
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .env("ELVANTO_SERVICES_LIST", "--id long")
        .args(["services", "list", "--json"])
        .assert()
        .success()
        .stderr(
            contains("ELVANTO_SERVICES_LIST defaults suppressed by manual flag")
                .and(contains("would have applied: --id long")),
        );
}

#[tokio::test]
async fn no_env_silences_note() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/services/getAll.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok_services_page()))
        .mount(&server)
        .await;

    let assert = raw_bin()
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .env("ELVANTO_SERVICES_LIST", "--id long")
        .args(["--no-env", "services", "list"])
        .assert()
        .success();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        !stderr.contains("ELVANTO_SERVICES_LIST"),
        "stderr should not mention env var when --no-env is set; got: {stderr:?}"
    );
}

#[tokio::test]
async fn no_env_global_flag_disables_injection() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/services/getAll.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok_services_page()))
        .mount(&server)
        .await;

    raw_bin()
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .env("ELVANTO_SERVICES_LIST", "--id long")
        .args(["--no-env", "services", "list"])
        .assert()
        .success()
        // Short id, NOT full UUID — env was ignored.
        .stdout(contains("1eb01e76 | 2026-05-23").and(contains("1eb01e76-7a5d-4d02").not()));
}

#[tokio::test]
async fn user_flag_disables_injection() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/services/getAll.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok_services_page()))
        .mount(&server)
        .await;

    // --json on the CLI; env (--id long) should be ignored.
    let out = raw_bin()
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .env("ELVANTO_SERVICES_LIST", "--id long")
        .args(["services", "list", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v[0]["id"], "1eb01e76-7a5d-4d02-a207-d75055645f14");
}
