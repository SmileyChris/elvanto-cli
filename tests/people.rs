mod common;
use common::{bin, mock_server};
use predicates::prelude::*;
use predicates::str::contains;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

fn ok_page(people: Vec<serde_json::Value>) -> serde_json::Value {
    let n = people.len() as u64;
    serde_json::json!({
        "status": "ok",
        "people": {
            "page": 1,
            "per_page": 1000,
            "total": n,
            "on_this_page": n,
            "person": people
        }
    })
}

fn person(
    id: &str,
    first: &str,
    last: &str,
    email: &str,
    status: &str,
    depts: serde_json::Value,
) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "firstname": first,
        "lastname": last,
        "preferred_name": "",
        "email": email,
        "status": status,
        "departments": depts
    })
}

fn dept(id: &str, name: &str, subs: Vec<(&str, &str)>) -> serde_json::Value {
    if subs.is_empty() {
        serde_json::json!({
            "id": id, "name": name,
            "sub_departments": []
        })
    } else {
        let sd: Vec<serde_json::Value> = subs
            .iter()
            .map(|(sid, sname)| serde_json::json!({ "id": sid, "name": sname }))
            .collect();
        serde_json::json!({
            "id": id, "name": name,
            "sub_departments": { "sub_department": sd }
        })
    }
}

#[tokio::test]
async fn people_list_default_text_active_only_short_ids() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/people/getAll.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok_page(vec![
            person(
                "1eb01e76-aaaa-bbbb-cccc-111111111111",
                "Alice",
                "Brown",
                "alice@example.com",
                "Active",
                serde_json::json!([]),
            ),
            person(
                "deadbeef-aaaa-bbbb-cccc-222222222222",
                "Bob",
                "Carter",
                "bob@example.com",
                "Archived",
                serde_json::json!([]),
            ),
        ])))
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["people", "list"])
        .assert()
        .success()
        .stdout(
            contains("1eb01e76 | Alice Brown | alice@example.com")
                .and(contains("Bob Carter").not()),
        );
}

#[tokio::test]
async fn people_list_department_filter_or_matches_dept_or_sub() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/people/getAll.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok_page(vec![
            // Has sub-dept "Vocals"
            person(
                "11111111-aaaa-bbbb-cccc-111111111111",
                "Alice",
                "Brown",
                "alice@example.com",
                "Active",
                serde_json::json!({
                    "department": [
                        dept("d-1", "Music Team", vec![("sd-1", "Vocals")])
                    ]
                }),
            ),
            // Has top-level dept "Welcome Team"
            person(
                "22222222-aaaa-bbbb-cccc-222222222222",
                "Bob",
                "Carter",
                "bob@example.com",
                "Active",
                serde_json::json!({
                    "department": [
                        dept("d-2", "Welcome Team", vec![])
                    ]
                }),
            ),
            // Has neither
            person(
                "33333333-aaaa-bbbb-cccc-333333333333",
                "Carol",
                "Davies",
                "carol@example.com",
                "Active",
                serde_json::json!([]),
            ),
        ])))
        .mount(&server)
        .await;

    let out = bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args([
            "people",
            "list",
            "--in",
            "Vocals", // sub-dept name → resolves to sd-1
            "--in",
            "Welcome Team", // dept name → resolves to d-2
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(out).unwrap();
    assert!(text.contains("Alice Brown"));
    assert!(text.contains("Bob Carter"));
    assert!(!text.contains("Carol Davies"));
}

#[tokio::test]
async fn people_list_json_includes_inactive_and_departments() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/people/getAll.json"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(ok_page(vec![person(
                "11111111-aaaa-bbbb-cccc-111111111111",
                "Alice",
                "Brown",
                "alice@example.com",
                "Archived",
                serde_json::json!({
                    "department": [
                        dept("d-1", "Music Team", vec![("sd-1", "Vocals")])
                    ]
                }),
            )])),
        )
        .mount(&server)
        .await;

    let out = bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["people", "list", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v[0]["name"], "Alice Brown");
    assert_eq!(v[0]["status"], "archived");
    assert_eq!(v[0]["departments"][0]["department"], "Music Team");
    assert_eq!(v[0]["departments"][0]["sub_department"], "Vocals");
}

#[tokio::test]
async fn people_departments_flat_unique_with_parent() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/people/getAll.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok_page(vec![
            person(
                "11111111-aaaa-bbbb-cccc-111111111111",
                "Alice",
                "Brown",
                "",
                "Active",
                serde_json::json!({
                    "department": [
                        dept("d-1", "Music Team", vec![("sd-1", "Vocals"), ("sd-2", "Instruments")])
                    ]
                }),
            ),
            person(
                "22222222-aaaa-bbbb-cccc-222222222222",
                "Bob",
                "Carter",
                "",
                "Active",
                serde_json::json!({
                    "department": [
                        dept("d-1", "Music Team", vec![("sd-1", "Vocals")]),
                        dept("d-2", "Welcome Team", vec![])
                    ]
                }),
            ),
        ])))
        .mount(&server)
        .await;

    let out = bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["people", "departments"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(out).unwrap();
    // 4 unique entries: Music Team, Vocals, Instruments, Welcome Team
    // (the test fixtures don't include positions, so no position rows.)
    assert_eq!(text.lines().count(), 4);
    assert!(text.contains("Music Team | department | -"));
    assert!(text.contains("Vocals | sub_department | Music Team"));
    assert!(text.contains("Instruments | sub_department | Music Team"));
    assert!(text.contains("Welcome Team | department | -"));
}

#[tokio::test]
async fn people_list_ambiguous_in_value_lists_candidates() {
    let server = mock_server().await;
    // Two positions both named "Leader": disambiguates via path.
    Mock::given(method("POST"))
        .and(path("/people/getAll.json"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(ok_page(vec![serde_json::json!({
                "id": "11111111-aaaa-bbbb-cccc-111111111111",
                "firstname": "Alice", "lastname": "Brown", "preferred_name": "",
                "email": "alice@example.com", "status": "Active",
                "departments": {
                    "department": [
                        {
                            "id": "d-1", "name": "Music Team",
                            "sub_departments": { "sub_department": [{
                                "id": "sd-1", "name": "Vocals",
                                "positions": { "position": [
                                    { "id": "p-1", "name": "Leader" },
                                ]}
                            }]}
                        },
                        {
                            "id": "d-2", "name": "Welcome Team",
                            "sub_departments": { "sub_department": [{
                                "id": "sd-2", "name": "Hosts",
                                "positions": { "position": [
                                    { "id": "p-2", "name": "Leader" },
                                ]}
                            }]}
                        },
                    ]
                }
            })])),
        )
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["people", "list", "--in", "Leader"])
        .assert()
        .failure()
        .stderr(
            contains("matches 2 nodes")
                .and(contains("Music Team / Vocals / Leader"))
                .and(contains("Welcome Team / Hosts / Leader")),
        );
}

#[tokio::test]
async fn people_list_typo_in_value_suggests_did_you_mean() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/people/getAll.json"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(ok_page(vec![person(
                "11111111-aaaa-bbbb-cccc-111111111111",
                "Alice",
                "Brown",
                "",
                "Active",
                serde_json::json!({
                    "department": [
                        dept("d-1", "Music Team", vec![("sd-1", "Vocals")])
                    ]
                }),
            )])),
        )
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["people", "list", "--in", "voclas"])
        .assert()
        .failure()
        .stderr(contains("Did you mean").and(contains("Vocals")));
}

#[tokio::test]
async fn people_list_position_filter_matches_position_name() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/people/getAll.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok_page(vec![
            // Alice: Vocals / Worship Leader
            serde_json::json!({
                "id": "11111111-aaaa-bbbb-cccc-111111111111",
                "firstname": "Alice", "lastname": "Brown", "preferred_name": "",
                "email": "alice@example.com", "status": "Active",
                "departments": {
                    "department": [{
                        "id": "d-1", "name": "Music Team",
                        "sub_departments": { "sub_department": [{
                            "id": "sd-1", "name": "Vocals",
                            "positions": { "position": [{ "id": "p-wl", "name": "Worship Leader" }] }
                        }]}
                    }]
                }
            }),
            // Bob: Vocals / BV (not a worship leader)
            serde_json::json!({
                "id": "22222222-aaaa-bbbb-cccc-222222222222",
                "firstname": "Bob", "lastname": "Carter", "preferred_name": "",
                "email": "bob@example.com", "status": "Active",
                "departments": {
                    "department": [{
                        "id": "d-1", "name": "Music Team",
                        "sub_departments": { "sub_department": [{
                            "id": "sd-1", "name": "Vocals",
                            "positions": { "position": [{ "id": "p-bv", "name": "BV" }] }
                        }]}
                    }]
                }
            }),
        ])))
        .mount(&server)
        .await;

    let out = bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["people", "list", "--in", "Worship Leader"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(out).unwrap();
    assert!(text.contains("Alice Brown"));
    assert!(!text.contains("Bob Carter"));
}
