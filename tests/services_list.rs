mod common;
use common::{bin, mock_server};
use predicates::prelude::*;
use predicates::str::contains;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, ResponseTemplate};

fn ok_page(services: Vec<serde_json::Value>) -> serde_json::Value {
    let on_this_page = services.len() as u64;
    serde_json::json!({
        "status": "ok",
        "services": {
            "page": 1,
            "per_page": 100,
            "total": on_this_page,
            "on_this_page": on_this_page,
            "service": services
        }
    })
}

fn svc(
    id: &str,
    date: &str,
    name: &str,
    status: &str,
    type_name: &str,
    location: &str,
) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "date": date,
        "name": name,
        "status": status,
        "service_type": { "id": "st-1", "name": type_name },
        "location": { "id": "loc-1", "name": location },
        "description": ""
    })
}

#[tokio::test]
async fn text_output_default_window() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/services/getAll.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok_page(vec![
            svc(
                "svc-1",
                "2026-04-12 09:30:00",
                "Sunday Morning",
                "Published",
                "Sunday Service",
                "Main",
            ),
            svc(
                "svc-2",
                "2026-04-19 09:30:00",
                "Sunday Morning",
                "Draft",
                "Sunday Service",
                "Main",
            ),
        ])))
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["services", "list"])
        .assert()
        .success()
        .stdout(
            contains("svc | 2026-04-12 | Sunday Morning | Sunday Service | Main | published").and(
                contains("svc | 2026-04-19 | Sunday Morning | Sunday Service | Main | draft"),
            ),
        );
}

#[tokio::test]
async fn full_id_flag_prints_full_uuid() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/services/getAll.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok_page(vec![svc(
            "1eb01e76-7a5d-4d02-a207-d75055645f14",
            "2026-04-12 09:30:00",
            "Sunday",
            "Published",
            "Sunday Service",
            "Main",
        )])))
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["services", "list", "--id", "long"])
        .assert()
        .success()
        .stdout(contains(
            "1eb01e76-7a5d-4d02-a207-d75055645f14 | 2026-04-12",
        ));
}

#[tokio::test]
async fn default_text_output_shortens_uuid() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/services/getAll.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok_page(vec![svc(
            "1eb01e76-7a5d-4d02-a207-d75055645f14",
            "2026-04-12 09:30:00",
            "Sunday",
            "Published",
            "Sunday Service",
            "Main",
        )])))
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["services", "list"])
        .assert()
        .success()
        .stdout(contains("1eb01e76 | 2026-04-12").and(contains("Sunday")));
}

#[tokio::test]
async fn from_and_to_flags_drive_request_body() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/services/getAll.json"))
        .and(body_partial_json(serde_json::json!({
            "start": "2026-01-01",
            "end": "2026-03-31"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok_page(vec![svc(
            "svc-9",
            "2026-02-14 18:00:00",
            "Valentine Vigil",
            "Published",
            "Special",
            "Hall",
        )])))
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args([
            "services",
            "list",
            "--from",
            "2026-01-01",
            "--to",
            "2026-03-31",
        ])
        .assert()
        .success()
        .stdout(contains("svc | 2026-02-14 | Valentine Vigil"));
}

#[tokio::test]
async fn json_output_normalized() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/services/getAll.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok_page(vec![svc(
            "svc-1",
            "2026-04-12 09:30:00",
            "Sunday Morning",
            "Published",
            "Sunday Service",
            "Main",
        )])))
        .mount(&server)
        .await;

    let out = bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["services", "list", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v[0]["id"], "svc-1");
    assert_eq!(v[0]["status"], "published");
    assert_eq!(v[0]["service_type"], "Sunday Service");
    assert_eq!(v[0]["location"], "Main");
    assert_eq!(v[0]["date"], "2026-04-12 09:30:00");
}

#[tokio::test]
async fn invalid_date_format_is_usage_error() {
    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", "http://127.0.0.1:1")
        .args(["services", "list", "--from", "01/15/2026"])
        .assert()
        .failure()
        .code(2)
        .stderr(contains("invalid --from").and(contains("YYYY-MM-DD")));
}

#[tokio::test]
async fn from_after_to_is_usage_error() {
    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", "http://127.0.0.1:1")
        .args([
            "services",
            "list",
            "--from",
            "2026-05-01",
            "--to",
            "2026-04-01",
        ])
        .assert()
        .failure()
        .code(2)
        .stderr(contains("--from").and(contains("must be on or before")));
}

#[tokio::test]
async fn empty_list_succeeds_with_no_output() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/services/getAll.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok_page(vec![])))
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["services", "list"])
        .assert()
        .success()
        .stdout("");
}
