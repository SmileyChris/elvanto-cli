use assert_cmd::Command;

/// Spawn the `elvanto` binary with all autoflag env vars stripped so that
/// a parent shell's `ELVANTO_SONGS_LIST=--json` (etc.) can't leak into tests.
#[allow(dead_code)]
pub fn bin() -> Command {
    let mut c = Command::cargo_bin("elvanto").unwrap();
    for var in [
        "ELVANTO_AUTH_CHECK",
        "ELVANTO_SERVICES_LIST",
        "ELVANTO_SERVICES_PEOPLE",
        "ELVANTO_SONGS_CATEGORIES",
        "ELVANTO_SONGS_CHART",
        "ELVANTO_SONGS_LIST",
        "ELVANTO_SONGS_LYRICS",
        "ELVANTO_SONGS_SHOW",
    ] {
        c.env_remove(var);
    }
    c
}

/// Like `bin()` but does NOT strip autoflag env vars — for testing the
/// autoflag injection itself.
#[allow(dead_code)]
pub fn raw_bin() -> Command {
    Command::cargo_bin("elvanto").unwrap()
}

#[allow(dead_code)]
pub async fn mock_server() -> wiremock::MockServer {
    wiremock::MockServer::start().await
}
