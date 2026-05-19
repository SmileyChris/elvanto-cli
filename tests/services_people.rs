mod common;
use common::{bin, mock_server};
use predicates::prelude::*;
use predicates::str::contains;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, ResponseTemplate};

fn ok_body(positions: Vec<serde_json::Value>) -> serde_json::Value {
    serde_json::json!({
        "status": "ok",
        "service": [{
            "id": "svc-1",
            "name": "Sunday",
            "date": "2026-05-23 22:00:00",
            "volunteers": {
                "plan": [{
                    "positions": {
                        "position": positions
                    }
                }]
            }
        }]
    })
}

fn pos_filled(dept: &str, sub: &str, name: &str, person: &str, status: &str) -> serde_json::Value {
    let (first, last) = person.split_once(' ').unwrap_or((person, ""));
    serde_json::json!({
        "department_name": dept,
        "sub_department_name": sub,
        "position_name": name,
        "volunteers": {
            "volunteer": [{
                "person": {
                    "id": format!("p-{}", first.to_ascii_lowercase()),
                    "firstname": first,
                    "lastname": last,
                    "preferred_name": ""
                },
                "status": status
            }]
        }
    })
}

fn pos_empty(dept: &str, sub: &str, name: &str) -> serde_json::Value {
    serde_json::json!({
        "department_name": dept,
        "sub_department_name": sub,
        "position_name": name,
        "volunteers": ""
    })
}

#[tokio::test]
async fn text_output_default_shows_filled_and_unfilled() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/services/getInfo.json"))
        .and(body_partial_json(serde_json::json!({
            "id": "svc-1",
            "fields": ["volunteers"]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok_body(vec![
            pos_filled(
                "Service Teams",
                "Service Leaders",
                "Preaching",
                "Annedien Looyenga",
                "Confirmed",
            ),
            pos_empty("Service Teams", "Communion", "Setup & Cleanup"),
        ])))
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["services", "people", "svc-1"])
        .assert()
        .success()
        .stdout(
            contains("Service Leaders | Preaching | Annedien Looyenga | confirmed")
                .and(contains("Communion | Setup & Cleanup | (unfilled) | -")),
        );
}

#[tokio::test]
async fn filled_flag_hides_unfilled() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/services/getInfo.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok_body(vec![
            pos_filled("Sound", "FOH", "Engineer", "Alice B", "Confirmed"),
            pos_empty("Sound", "Stage", "Monitors"),
        ])))
        .mount(&server)
        .await;

    let out = bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["services", "people", "svc-1", "--filled"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(out).unwrap();
    assert!(text.contains("Alice B"));
    assert!(!text.contains("Monitors"));
    assert!(!text.contains("unfilled"));
}

#[tokio::test]
async fn json_output_flat_array() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/services/getInfo.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok_body(vec![
            pos_filled("Sound", "FOH", "Engineer", "Alice B", "Confirmed"),
            pos_empty("Sound", "Stage", "Monitors"),
        ])))
        .mount(&server)
        .await;

    let out = bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["services", "people", "svc-1", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    let arr = v.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["position"], "Engineer");
    assert_eq!(arr[0]["sub_department"], "FOH");
    assert_eq!(arr[0]["status"], "confirmed");
    assert_eq!(arr[1]["position"], "Monitors");
    assert!(arr[1].get("status").is_none());
    assert!(arr[1].get("name").is_none());
}

#[tokio::test]
async fn empty_volunteers_string_does_not_crash_deserializer() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/services/getInfo.json"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(ok_body(vec![pos_empty("X", "Y", "Z")])),
        )
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["services", "people", "svc-1"])
        .assert()
        .success()
        .stdout(contains("Y | Z | (unfilled) | -"));
}

#[tokio::test]
async fn short_id_resolves_via_list_when_direct_lookup_fails() {
    let server = mock_server().await;
    // 1) Direct getInfo with the short id returns not-found.
    Mock::given(method("POST"))
        .and(path("/services/getInfo.json"))
        .and(body_partial_json(serde_json::json!({ "id": "1eb01e76" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "ok",
            "service": []
        })))
        .mount(&server)
        .await;
    // 2) list_services returns a service whose short id matches.
    Mock::given(method("POST"))
        .and(path("/services/getAll.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "ok",
            "services": {
                "page": 1, "per_page": 100, "total": 1, "on_this_page": 1,
                "service": [{
                    "id": "1eb01e76-7a5d-4d02-a207-d75055645f14",
                    "date": "2026-05-23 22:00:00",
                    "name": "Sunday",
                    "status": "Published"
                }]
            }
        })))
        .mount(&server)
        .await;
    // 3) getInfo with the full id returns the volunteers payload.
    Mock::given(method("POST"))
        .and(path("/services/getInfo.json"))
        .and(body_partial_json(serde_json::json!({
            "id": "1eb01e76-7a5d-4d02-a207-d75055645f14"
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(ok_body(vec![pos_filled(
                "Sound",
                "FOH",
                "Engineer",
                "Alice B",
                "Confirmed",
            )])),
        )
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["services", "people", "1eb01e76"])
        .assert()
        .success()
        .stdout(contains("FOH | Engineer | Alice B | confirmed"));
}

#[tokio::test]
async fn service_not_found_exits_1() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/services/getInfo.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "ok",
            "service": []
        })))
        .mount(&server)
        .await;
    // Fallback list_services returns nothing → original NotFound propagates.
    Mock::given(method("POST"))
        .and(path("/services/getAll.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "ok",
            "services": {
                "page": 1, "per_page": 100, "total": 0, "on_this_page": 0,
                "service": []
            }
        })))
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["services", "people", "missing"])
        .assert()
        .failure()
        .code(1)
        .stderr(contains("not found: service missing"));
}
