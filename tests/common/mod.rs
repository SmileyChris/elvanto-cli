use assert_cmd::Command;

/// Set on every test-spawned binary: swap the OS keyring for `keyring`'s
/// in-memory mock so tests never touch the user's real Secret Service /
/// Keychain / Credential Manager. Each spawned process starts with an
/// empty mock; nothing leaks between tests.
fn install_keyring_mock(c: &mut Command) {
    c.env("ELVANTO_KEYRING_MOCK", "1");
}

/// Spawn the `elvanto` binary with all autoflag env vars neutralised so that
/// a parent shell's `ELVANTO_SONGS_LIST=--json` (etc.) — or a local `.env` —
/// can't leak into tests. We set empty-string rather than `env_remove` so
/// that dotenvy doesn't repopulate the variables from `.env`.
#[allow(dead_code)]
pub fn bin() -> Command {
    let mut c = Command::cargo_bin("elvanto").unwrap();
    install_keyring_mock(&mut c);
    for var in [
        "ELVANTO_PEOPLE_LIST",
        "ELVANTO_PEOPLE_ORG",
        "ELVANTO_SERVICES_LIST",
        "ELVANTO_SERVICES_PEOPLE",
        "ELVANTO_SONGS_CATEGORIES",
        "ELVANTO_SONGS_CHART",
        "ELVANTO_SONGS_LIST",
        "ELVANTO_SONGS_LYRICS",
        "ELVANTO_SONGS_SHOW",
    ] {
        c.env(var, "");
    }
    c
}

/// Like `bin()` but does NOT strip autoflag env vars — for testing the
/// autoflag injection itself. Still uses the mock keyring.
#[allow(dead_code)]
pub fn raw_bin() -> Command {
    let mut c = Command::cargo_bin("elvanto").unwrap();
    install_keyring_mock(&mut c);
    c
}

#[allow(dead_code)]
pub async fn mock_server() -> wiremock::MockServer {
    wiremock::MockServer::start().await
}
