# Elvanto CLI V1 (Songs, Read-Only) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a working `elvanto` Rust binary covering the V1 read-only Songs surface: `auth check`, `songs categories`, `songs list`, `songs show`, `songs chart`, `songs lyrics`.

**Architecture:** A single Cargo binary crate (`elvanto-cli`, binary name `elvanto`). Clap-derived CLI dispatches to per-command handlers. A thin async `reqwest` client posts JSON to `https://api.elvanto.com/v1/<endpoint>.json` using HTTP Basic auth (`API_KEY:x`). Raw Elvanto response types are deserialized internally; conversion to a stable normalized shape happens at the output boundary. Tokio multi-thread runtime is overkill; use `tokio::main(flavor = "current_thread")`. HTTP is mocked with `wiremock` in tests; CLI flow is exercised with `assert_cmd`.

**Tech Stack:** Rust 2021 edition. `clap` (derive), `reqwest` (rustls, json), `serde`/`serde_json`, `tokio` (current_thread), `thiserror`, `anyhow` (binary-level only), `wiremock` (dev), `assert_cmd` + `predicates` (dev). No DB. No config file in V1.

---

## File Structure

```
elvanto-cli/
├── Cargo.toml
├── Cargo.lock
├── .gitignore
├── src/
│   ├── main.rs                  # entry point; tokio runtime; calls cli::run
│   ├── cli.rs                   # clap definitions + dispatcher
│   ├── error.rs                 # CliError, ExitCode mapping
│   ├── api/
│   │   ├── mod.rs               # Client struct, auth, request helper
│   │   ├── raw.rs               # raw response types (mirror Elvanto JSON)
│   │   └── endpoints.rs         # typed wrappers per endpoint
│   ├── domain/
│   │   ├── mod.rs               # re-exports
│   │   ├── song.rs              # normalized Song type + From<raw::Song>
│   │   ├── arrangement.rs       # normalized Arrangement + Key
│   │   └── category.rs          # normalized Category
│   ├── commands/
│   │   ├── mod.rs               # re-exports
│   │   ├── auth_check.rs
│   │   ├── songs_categories.rs
│   │   ├── songs_list.rs
│   │   ├── songs_show.rs
│   │   ├── songs_chart.rs
│   │   └── songs_lyrics.rs
│   ├── output/
│   │   ├── mod.rs               # re-exports + tiny helpers
│   │   ├── text.rs              # text/table renderers
│   │   └── json.rs              # serialize normalized types
│   ├── transpose.rs             # transpose key parser + chord_chart_key map
│   └── arrangement_select.rs    # default/first/by-name selection
└── tests/
    ├── auth_check.rs
    ├── songs_categories.rs
    ├── songs_list.rs
    ├── songs_show.rs
    ├── songs_chart.rs
    ├── songs_lyrics.rs
    └── common/
        └── mod.rs               # spawn binary against wiremock server
```

**Boundaries:**
- `api/` only knows raw Elvanto types. It never imports `domain/` or `output/`.
- `domain/` only knows normalized types and `From<raw::*>` impls. It never imports `output/` or `api/` (other than the raw types module).
- `commands/` is the orchestrator: takes parsed args + `Client`, returns a `Result<(), CliError>`.
- `output/` formats normalized types. Pure functions. No I/O beyond writing to a `&mut dyn Write`.

---

## Task 1: Project scaffold + git init

**Files:**
- Create: `/home/chris/dev/elvanto-cli/Cargo.toml`
- Create: `/home/chris/dev/elvanto-cli/.gitignore`
- Create: `/home/chris/dev/elvanto-cli/src/main.rs`

- [ ] **Step 1: Initialize git repo (the working directory is not yet a repo)**

Run:
```bash
cd /home/chris/dev/elvanto-cli && git init && git add -A && git commit -m "docs: import existing design docs"
```
Expected: `Initialized empty Git repository` and a commit listing CONTEXT.md, README.md, docs/.

- [ ] **Step 2: Write Cargo.toml**

```toml
[package]
name = "elvanto-cli"
version = "0.1.0"
edition = "2021"
description = "CLI for the Elvanto API (songs read-only)"
license = "MIT OR Apache-2.0"
publish = false

[[bin]]
name = "elvanto"
path = "src/main.rs"

[dependencies]
clap = { version = "4", features = ["derive"] }
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["macros", "rt"] }
thiserror = "1"
anyhow = "1"

[dev-dependencies]
wiremock = "0.6"
assert_cmd = "2"
predicates = "3"
tokio = { version = "1", features = ["macros", "rt", "rt-multi-thread"] }
```

- [ ] **Step 3: Write .gitignore**

```
/target
Cargo.lock.bak
*.swp
.idea/
.vscode/
```

Note: `Cargo.lock` IS committed (this is a binary crate).

- [ ] **Step 4: Write minimal src/main.rs**

```rust
fn main() {
    println!("elvanto-cli v{}", env!("CARGO_PKG_VERSION"));
}
```

- [ ] **Step 5: Verify it builds and runs**

Run: `cargo build`
Expected: builds clean.

Run: `cargo run --quiet`
Expected: `elvanto-cli v0.1.0`.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock .gitignore src/
git commit -m "feat: scaffold binary crate with dependency set"
```

---

## Task 2: Error type + exit code mapping

**Files:**
- Create: `/home/chris/dev/elvanto-cli/src/error.rs`
- Modify: `/home/chris/dev/elvanto-cli/src/main.rs`
- Test: inline `#[cfg(test)]` in `src/error.rs`

- [ ] **Step 1: Write the failing test**

Add to `src/error.rs`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("Elvanto returned code {code}: {message}")]
    Api { code: i64, message: String },

    #[error("network: {0}")]
    Network(String),

    #[error("{0}")]
    Usage(String),
}

impl CliError {
    pub fn exit_code(&self) -> i32 {
        match self {
            CliError::Api { .. } | CliError::Network(_) => 1,
            CliError::Usage(_) => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_error_exits_1() {
        let err = CliError::Api { code: 250, message: "No search parameters provided.".into() };
        assert_eq!(err.exit_code(), 1);
        assert_eq!(err.to_string(), "Elvanto returned code 250: No search parameters provided.");
    }

    #[test]
    fn network_error_exits_1() {
        let err = CliError::Network("connection timed out".into());
        assert_eq!(err.exit_code(), 1);
        assert_eq!(err.to_string(), "network: connection timed out");
    }

    #[test]
    fn usage_error_exits_2() {
        let err = CliError::Usage("ELVANTO_API_KEY is not set".into());
        assert_eq!(err.exit_code(), 2);
    }
}
```

- [ ] **Step 2: Wire into main.rs**

Replace `src/main.rs` with:

```rust
mod error;

use error::CliError;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(err.exit_code() as u8)
        }
    }
}

fn run() -> Result<(), CliError> {
    Ok(())
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test --lib error::`
Expected: 3 passed.

- [ ] **Step 4: Commit**

```bash
git add src/error.rs src/main.rs
git commit -m "feat(error): add CliError with exit code mapping"
```

---

## Task 3: API client core (auth + request helper) — happy path

**Files:**
- Create: `/home/chris/dev/elvanto-cli/src/api/mod.rs`
- Create: `/home/chris/dev/elvanto-cli/src/api/raw.rs`
- Modify: `/home/chris/dev/elvanto-cli/src/main.rs` (add `mod api;`)
- Test: inline in `src/api/mod.rs` against `wiremock`

Elvanto endpoint convention: POST `https://api.elvanto.com/v1/<endpoint>.json` with JSON body. Auth = HTTP Basic with API key as username, any non-empty password (use `"x"`). Successful envelope: `{"status":"ok", ...payload}`. Failure envelope: `{"status":"error","error":{"code":<int>,"message":"..."}}`.

- [ ] **Step 1: Write src/api/raw.rs**

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum Envelope<T> {
    Ok(T),
    Error { error: ApiError },
}

#[derive(Debug, Deserialize)]
pub struct ApiError {
    pub code: i64,
    pub message: String,
}
```

Note: serde's internally-tagged enum needs an `Ok` variant that flattens the payload. Because `T` may itself be a struct with arbitrary fields, we deserialize manually instead. Replace the above with:

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RawEnvelope {
    pub status: String,
    #[serde(default)]
    pub error: Option<ApiError>,
}

#[derive(Debug, Deserialize)]
pub struct ApiError {
    pub code: i64,
    pub message: String,
}
```

The client reads the body as `serde_json::Value`, inspects `status`, then either returns the `ApiError` or deserializes the whole `Value` into the caller's payload type `T` (which is expected to use `#[serde(default)]` and ignore the `status` field).

- [ ] **Step 2: Write src/api/mod.rs (the Client)**

```rust
pub mod raw;

use crate::error::CliError;
use reqwest::Client as Http;
use serde::de::DeserializeOwned;
use serde::Serialize;

const DEFAULT_BASE_URL: &str = "https://api.elvanto.com/v1";

pub struct Client {
    http: Http,
    base_url: String,
    api_key: String,
}

impl Client {
    pub fn new(api_key: String) -> Result<Self, CliError> {
        Self::with_base_url(api_key, DEFAULT_BASE_URL.to_string())
    }

    pub fn with_base_url(api_key: String, base_url: String) -> Result<Self, CliError> {
        if api_key.is_empty() {
            return Err(CliError::Usage("ELVANTO_API_KEY is empty".into()));
        }
        let http = Http::builder()
            .user_agent(concat!("elvanto-cli/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| CliError::Network(e.to_string()))?;
        Ok(Self { http, base_url, api_key })
    }

    pub fn redacted_key(&self) -> String {
        let k = &self.api_key;
        if k.len() <= 8 {
            "*".repeat(k.len())
        } else {
            format!("{}…{}", &k[..4], &k[k.len() - 4..])
        }
    }

    pub async fn post<B, T>(&self, endpoint: &str, body: &B) -> Result<T, CliError>
    where
        B: Serialize,
        T: DeserializeOwned,
    {
        let url = format!("{}/{}.json", self.base_url, endpoint);
        let resp = self
            .http
            .post(&url)
            .basic_auth(&self.api_key, Some("x"))
            .json(body)
            .send()
            .await
            .map_err(|e| CliError::Network(e.to_string()))?;

        let value: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| CliError::Network(format!("invalid response body: {e}")))?;

        match value.get("status").and_then(|v| v.as_str()) {
            Some("ok") => serde_json::from_value(value)
                .map_err(|e| CliError::Network(format!("decode error: {e}"))),
            Some("error") => {
                let err: raw::ApiError = serde_json::from_value(
                    value.get("error").cloned().unwrap_or(serde_json::Value::Null),
                )
                .map_err(|e| CliError::Network(format!("decode error: {e}")))?;
                Err(CliError::Api { code: err.code, message: err.message })
            }
            other => Err(CliError::Network(format!(
                "unexpected status: {:?}",
                other
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[derive(Debug, Deserialize)]
    struct Pong {
        #[allow(dead_code)]
        status: String,
        pong: String,
    }

    #[tokio::test]
    async fn ok_envelope_decodes_payload() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/ping.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "ok",
                "pong": "hi"
            })))
            .mount(&server)
            .await;

        let client = Client::with_base_url("key123abc".into(), server.uri()).unwrap();
        let out: Pong = client.post("ping", &serde_json::json!({})).await.unwrap();
        assert_eq!(out.pong, "hi");
    }

    #[tokio::test]
    async fn error_envelope_becomes_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/songs/getAll.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "error",
                "error": { "code": 250, "message": "No search parameters provided." }
            })))
            .mount(&server)
            .await;

        let client = Client::with_base_url("k".repeat(10), server.uri()).unwrap();
        let res: Result<serde_json::Value, _> =
            client.post("songs/getAll", &serde_json::json!({})).await;
        match res {
            Err(CliError::Api { code, message }) => {
                assert_eq!(code, 250);
                assert_eq!(message, "No search parameters provided.");
            }
            other => panic!("expected Api error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn sends_basic_auth_header() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/auth/probe.json"))
            // "abcd1234:x" base64 == "YWJjZDEyMzQ6eA=="
            .and(header("authorization", "Basic YWJjZDEyMzQ6eA=="))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "ok"
            })))
            .mount(&server)
            .await;

        let client = Client::with_base_url("abcd1234".into(), server.uri()).unwrap();
        let _: serde_json::Value = client
            .post("auth/probe", &serde_json::json!({}))
            .await
            .unwrap();
    }

    #[test]
    fn redact_short_key() {
        let c = Client::with_base_url("abcdefgh".into(), "http://x".into()).unwrap();
        assert_eq!(c.redacted_key(), "********");
    }

    #[test]
    fn redact_long_key() {
        let c = Client::with_base_url("abcdefghijkl".into(), "http://x".into()).unwrap();
        assert_eq!(c.redacted_key(), "abcd…ijkl");
    }
}
```

- [ ] **Step 3: Wire module into main.rs**

Edit `src/main.rs`, add `mod api;` after `mod error;`. Add `#[allow(dead_code)]` above `mod api;` to avoid warnings until consumed.

```rust
mod error;
#[allow(dead_code)]
mod api;
```

- [ ] **Step 4: Run tests**

Run: `cargo test --lib api::`
Expected: 5 passed.

- [ ] **Step 5: Commit**

```bash
git add src/api/ src/main.rs
git commit -m "feat(api): add HTTP client with basic auth and envelope handling"
```

---

## Task 4: CLI skeleton with clap + dispatcher

**Files:**
- Create: `/home/chris/dev/elvanto-cli/src/cli.rs`
- Create: `/home/chris/dev/elvanto-cli/src/commands/mod.rs`
- Modify: `/home/chris/dev/elvanto-cli/src/main.rs`
- Test: `tests/help.rs`
- Test: `tests/common/mod.rs`

- [ ] **Step 1: Write src/commands/mod.rs (empty stubs for now)**

```rust
// command modules added per-task
```

- [ ] **Step 2: Write src/cli.rs**

```rust
use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "elvanto", version, about = "CLI for the Elvanto API", long_about = None)]
pub struct Cli {
    /// Print extra diagnostic information to stderr (credentials are redacted).
    #[arg(long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Authentication utilities.
    Auth {
        #[command(subcommand)]
        command: AuthCommand,
    },
    /// Worship song commands.
    Songs {
        #[command(subcommand)]
        command: SongsCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum AuthCommand {
    /// Verify the configured API key works.
    Check,
}

#[derive(Debug, Subcommand)]
pub enum SongsCommand {
    /// List song categories.
    Categories(JsonOnly),
    /// List active songs (all pages).
    List(SongsListArgs),
    /// Show a song by id.
    Show(SongsShowArgs),
    /// Print the chord chart for a song's default arrangement.
    Chart(SongsChartArgs),
    /// Print the lyrics for a song's default arrangement.
    Lyrics(SongsLyricsArgs),
}

#[derive(Debug, Args)]
pub struct JsonOnly {
    /// Emit normalized JSON instead of text.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct SongsListArgs {
    /// Emit normalized JSON; includes non-active songs.
    #[arg(long)]
    pub json: bool,
    /// Include the album column in text output.
    #[arg(long)]
    pub album: bool,
    /// Include the CCLI number column in text output.
    #[arg(long)]
    pub ccli: bool,
}

#[derive(Debug, Args)]
pub struct SongsShowArgs {
    pub id: String,
    /// Emit normalized JSON of the full song object.
    #[arg(long)]
    pub json: bool,
    /// Expand all fields in text output (excluding lyrics/chord chart).
    #[arg(long)]
    pub full: bool,
    /// Include attachment data (only meaningful with --json).
    #[arg(long)]
    pub files: bool,
}

#[derive(Debug, Args)]
pub struct SongsChartArgs {
    pub id: String,
    /// Transpose to a named key (C, F#, Bb) or a relative offset (-2, +3).
    #[arg(long)]
    pub transpose: Option<String>,
    /// Use this arrangement instead of the default.
    #[arg(long)]
    pub arrangement: Option<String>,
}

#[derive(Debug, Args)]
pub struct SongsLyricsArgs {
    pub id: String,
    /// Use this arrangement instead of the default.
    #[arg(long)]
    pub arrangement: Option<String>,
}
```

- [ ] **Step 3: Update src/main.rs to dispatch**

```rust
mod api;
mod cli;
mod commands;
mod error;

use clap::Parser;
use cli::{Cli, Command};
use error::CliError;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");

    let result = rt.block_on(run(cli));
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(err.exit_code() as u8)
        }
    }
}

async fn run(cli: Cli) -> Result<(), CliError> {
    match cli.command {
        Command::Auth { command: _ } => Err(CliError::Usage("auth check not implemented yet".into())),
        Command::Songs { command: _ } => Err(CliError::Usage("songs commands not implemented yet".into())),
    }
}
```

Remove the previous `#[allow(dead_code)] mod api;` line — the module is still unused at this point because `run` doesn't construct a Client. Add `#[allow(dead_code)]` on the `Client` struct in `src/api/mod.rs` until Task 5 wires it in.

Edit `src/api/mod.rs` line near `pub struct Client {`:

```rust
#[allow(dead_code)]
pub struct Client {
```

- [ ] **Step 4: Write tests/common/mod.rs (shared harness)**

```rust
use assert_cmd::Command;
use std::path::PathBuf;
use wiremock::MockServer;

pub fn bin() -> Command {
    Command::cargo_bin(env!("CARGO_PKG_NAME").replace("-cli", ""))
        .or_else(|_| Command::cargo_bin("elvanto"))
        .unwrap_or_else(|_| Command::cargo_bin("elvanto-cli").unwrap())
}

pub async fn mock_server() -> MockServer {
    MockServer::start().await
}

pub fn fixture_path(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures");
    p.push(name);
    p
}
```

Note: `Command::cargo_bin("elvanto")` matches the `[[bin]] name = "elvanto"` in Cargo.toml, so the `or_else` ladder is defensive only. Replace the body with simply:

```rust
pub fn bin() -> Command {
    Command::cargo_bin("elvanto").unwrap()
}
```

- [ ] **Step 5: Write tests/help.rs**

```rust
mod common;
use common::bin;
use predicates::str::contains;

#[test]
fn shows_top_level_help() {
    bin()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("auth").and(contains("songs")));
}

#[test]
fn songs_subcommands_listed() {
    bin()
        .args(["songs", "--help"])
        .assert()
        .success()
        .stdout(
            contains("categories")
                .and(contains("list"))
                .and(contains("show"))
                .and(contains("chart"))
                .and(contains("lyrics")),
        );
}
```

- [ ] **Step 6: Run**

Run: `cargo test --test help`
Expected: 2 passed.

- [ ] **Step 7: Commit**

```bash
git add src/cli.rs src/commands/ src/main.rs src/api/mod.rs tests/common/ tests/help.rs
git commit -m "feat(cli): add clap structure for all V1 commands"
```

---

## Task 5: `auth check` command

**Files:**
- Create: `/home/chris/dev/elvanto-cli/src/commands/auth_check.rs`
- Modify: `/home/chris/dev/elvanto-cli/src/commands/mod.rs`
- Modify: `/home/chris/dev/elvanto-cli/src/main.rs`
- Test: `tests/auth_check.rs`

Elvanto offers no "account info" endpoint for API-key callers. We probe with the cheapest read: `songs/categories/getAll` with no body. Status `ok` = key valid. Status `error` with auth code = key bad.

- [ ] **Step 1: Write src/commands/auth_check.rs**

```rust
use crate::api::Client;
use crate::error::CliError;

pub async fn run(client: &Client) -> Result<(), CliError> {
    // Probe with a cheap read. Any "ok" envelope means the key authenticated.
    let _: serde_json::Value = client
        .post("songs/categories/getAll", &serde_json::json!({}))
        .await?;

    println!("auth: ok ({})", client.redacted_key());
    Ok(())
}
```

- [ ] **Step 2: Update src/commands/mod.rs**

```rust
pub mod auth_check;
```

- [ ] **Step 3: Wire dispatcher in src/main.rs**

Replace the `run` function:

```rust
async fn run(cli: Cli) -> Result<(), CliError> {
    let api_key = std::env::var("ELVANTO_API_KEY")
        .map_err(|_| CliError::Usage("ELVANTO_API_KEY is not set".into()))?;
    let base_url = std::env::var("ELVANTO_BASE_URL")
        .unwrap_or_else(|_| "https://api.elvanto.com/v1".to_string());
    let client = api::Client::with_base_url(api_key, base_url)?;

    if cli.verbose {
        eprintln!("verbose: api_key={}", client.redacted_key());
    }

    match cli.command {
        Command::Auth { command } => match command {
            cli::AuthCommand::Check => commands::auth_check::run(&client).await,
        },
        Command::Songs { command: _ } => {
            Err(CliError::Usage("songs commands not implemented yet".into()))
        }
    }
}
```

Add `use cli::Command;` is already there; ensure `use cli;` (module) is reachable; if needed, change matches to use the fully-qualified `cli::AuthCommand::Check`.

Remove `#[allow(dead_code)]` from `Client` in `src/api/mod.rs` — it is now used.

- [ ] **Step 4: Write tests/auth_check.rs**

```rust
mod common;
use common::{bin, mock_server};
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
```

- [ ] **Step 5: Run**

Run: `cargo test --test auth_check`
Expected: 3 passed.

- [ ] **Step 6: Commit**

```bash
git add src/commands/ src/main.rs src/api/mod.rs tests/auth_check.rs
git commit -m "feat(auth): implement auth check via songs/categories probe"
```

---

## Task 6: Domain types + raw decoders for Category

**Files:**
- Create: `/home/chris/dev/elvanto-cli/src/domain/mod.rs`
- Create: `/home/chris/dev/elvanto-cli/src/domain/category.rs`
- Create: `/home/chris/dev/elvanto-cli/src/api/endpoints.rs`
- Modify: `/home/chris/dev/elvanto-cli/src/api/mod.rs` (re-export `endpoints`)
- Modify: `/home/chris/dev/elvanto-cli/src/api/raw.rs` (add Category raw type)
- Modify: `/home/chris/dev/elvanto-cli/src/main.rs` (`mod domain;`)
- Test: inline in `src/domain/category.rs`

Elvanto wraps lists in `{outer: {inner: [...]}}`. Categories come as `{"categories": {"category": [{"id":..., "name":...}]}}` on top of the envelope.

- [ ] **Step 1: Add raw types in src/api/raw.rs**

Append:

```rust
#[derive(Debug, Deserialize)]
pub struct CategoriesResponse {
    #[serde(default)]
    pub categories: CategoryList,
}

#[derive(Debug, Deserialize, Default)]
pub struct CategoryList {
    #[serde(default)]
    pub category: Vec<RawCategory>,
}

#[derive(Debug, Deserialize)]
pub struct RawCategory {
    pub id: String,
    pub name: String,
}
```

- [ ] **Step 2: Write src/domain/category.rs**

```rust
use crate::api::raw::RawCategory;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Category {
    pub id: String,
    pub name: String,
}

impl From<RawCategory> for Category {
    fn from(raw: RawCategory) -> Self {
        Self { id: raw.id, name: raw.name }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_raw_preserves_fields() {
        let raw = RawCategory { id: "cat-1".into(), name: "Worship".into() };
        let cat: Category = raw.into();
        assert_eq!(cat, Category { id: "cat-1".into(), name: "Worship".into() });
    }
}
```

- [ ] **Step 3: Write src/domain/mod.rs**

```rust
pub mod category;
```

- [ ] **Step 4: Write src/api/endpoints.rs**

```rust
use crate::api::raw::{CategoriesResponse, RawCategory};
use crate::api::Client;
use crate::error::CliError;

impl Client {
    pub async fn list_categories(&self) -> Result<Vec<RawCategory>, CliError> {
        let resp: CategoriesResponse = self
            .post("songs/categories/getAll", &serde_json::json!({}))
            .await?;
        Ok(resp.categories.category)
    }
}
```

- [ ] **Step 5: Wire endpoints module in src/api/mod.rs**

Add to top of `src/api/mod.rs` after `pub mod raw;`:

```rust
mod endpoints;
```

- [ ] **Step 6: Wire `mod domain;` in src/main.rs**

Add after `mod commands;`:

```rust
mod domain;
```

- [ ] **Step 7: Run**

Run: `cargo test --lib domain::category::`
Expected: 1 passed.

Run: `cargo build`
Expected: builds clean.

- [ ] **Step 8: Commit**

```bash
git add src/domain/ src/api/ src/main.rs
git commit -m "feat(domain): add Category type and categories endpoint"
```

---

## Task 7: `songs categories` command (text + JSON output)

**Files:**
- Create: `/home/chris/dev/elvanto-cli/src/output/mod.rs`
- Create: `/home/chris/dev/elvanto-cli/src/output/text.rs`
- Create: `/home/chris/dev/elvanto-cli/src/output/json.rs`
- Create: `/home/chris/dev/elvanto-cli/src/commands/songs_categories.rs`
- Modify: `/home/chris/dev/elvanto-cli/src/commands/mod.rs`
- Modify: `/home/chris/dev/elvanto-cli/src/main.rs` (`mod output;` + dispatch)
- Test: `tests/songs_categories.rs`

- [ ] **Step 1: Write src/output/mod.rs**

```rust
pub mod json;
pub mod text;
```

- [ ] **Step 2: Write src/output/text.rs**

```rust
use crate::domain::category::Category;
use std::io::{self, Write};

pub fn write_categories<W: Write>(w: &mut W, cats: &[Category]) -> io::Result<()> {
    for c in cats {
        writeln!(w, "{} | {}", c.id, c.name)?;
    }
    Ok(())
}
```

- [ ] **Step 3: Write src/output/json.rs**

```rust
use serde::Serialize;
use std::io::{self, Write};

pub fn write_pretty<W: Write, T: Serialize>(w: &mut W, value: &T) -> io::Result<()> {
    serde_json::to_writer_pretty(&mut *w, value)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    writeln!(w)
}
```

- [ ] **Step 4: Write src/commands/songs_categories.rs**

```rust
use crate::api::Client;
use crate::cli::JsonOnly;
use crate::domain::category::Category;
use crate::error::CliError;
use crate::output;

pub async fn run(client: &Client, args: JsonOnly) -> Result<(), CliError> {
    let raw = client.list_categories().await?;
    let cats: Vec<Category> = raw.into_iter().map(Into::into).collect();

    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    let res = if args.json {
        output::json::write_pretty(&mut lock, &cats)
    } else {
        output::text::write_categories(&mut lock, &cats)
    };
    res.map_err(|e| CliError::Network(format!("write error: {e}")))
}
```

- [ ] **Step 5: Update src/commands/mod.rs**

```rust
pub mod auth_check;
pub mod songs_categories;
```

- [ ] **Step 6: Wire in main.rs**

Add `mod output;` near the other module declarations. Update the `Command::Songs` arm:

```rust
Command::Songs { command } => match command {
    cli::SongsCommand::Categories(args) => commands::songs_categories::run(&client, args).await,
    _ => Err(CliError::Usage("not implemented yet".into())),
},
```

- [ ] **Step 7: Write tests/songs_categories.rs**

```rust
mod common;
use common::{bin, mock_server};
use predicates::str::contains;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

fn ok_body() -> serde_json::Value {
    serde_json::json!({
        "status": "ok",
        "categories": {
            "category": [
                { "id": "c1", "name": "Worship" },
                { "id": "c2", "name": "Hymns" }
            ]
        }
    })
}

#[tokio::test]
async fn text_output() {
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
        .stdout(contains("c1 | Worship").and(contains("c2 | Hymns")));
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
    assert_eq!(parsed[0]["id"], "c1");
    assert_eq!(parsed[1]["name"], "Hymns");
}
```

- [ ] **Step 8: Run**

Run: `cargo test --test songs_categories`
Expected: 2 passed.

- [ ] **Step 9: Commit**

```bash
git add src/output/ src/commands/ src/main.rs tests/songs_categories.rs
git commit -m "feat(songs): add songs categories command with text and JSON output"
```

---

## Task 8: Song domain type + `songs list` (pagination + columns)

**Files:**
- Create: `/home/chris/dev/elvanto-cli/src/domain/song.rs`
- Create: `/home/chris/dev/elvanto-cli/src/commands/songs_list.rs`
- Modify: `/home/chris/dev/elvanto-cli/src/api/raw.rs`
- Modify: `/home/chris/dev/elvanto-cli/src/api/endpoints.rs`
- Modify: `/home/chris/dev/elvanto-cli/src/domain/mod.rs`
- Modify: `/home/chris/dev/elvanto-cli/src/output/text.rs`
- Modify: `/home/chris/dev/elvanto-cli/src/commands/mod.rs`
- Modify: `/home/chris/dev/elvanto-cli/src/main.rs`
- Test: `tests/songs_list.rs`
- Test: inline in `src/domain/song.rs`

`songs/getAll` returns `{songs:{song:[...], page, per_page, total, on_this_page}}`. Status `1` means active, `0` means inactive. `number` is the CCLI number. Auto-paginate by walking pages until `on_this_page < per_page` or until accumulated count >= `total`.

- [ ] **Step 1: Add raw types in src/api/raw.rs**

Append:

```rust
#[derive(Debug, Deserialize)]
pub struct SongsResponse {
    #[serde(default)]
    pub songs: SongList,
}

#[derive(Debug, Deserialize, Default)]
pub struct SongList {
    #[serde(default)]
    pub page: u32,
    #[serde(default)]
    pub per_page: u32,
    #[serde(default)]
    pub total: u32,
    #[serde(default)]
    pub on_this_page: u32,
    #[serde(default)]
    pub song: Vec<RawSong>,
}

#[derive(Debug, Deserialize)]
pub struct RawSong {
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub artist: String,
    #[serde(default)]
    pub album: String,
    /// CCLI number per Elvanto docs.
    #[serde(default)]
    pub number: String,
    /// "1" = active, "0" = inactive (Elvanto serializes booleans as numeric strings here).
    #[serde(default)]
    pub status: serde_json::Value,
}

impl RawSong {
    pub fn is_active(&self) -> bool {
        match &self.status {
            serde_json::Value::String(s) => s == "1" || s.eq_ignore_ascii_case("active"),
            serde_json::Value::Number(n) => n.as_i64() == Some(1),
            _ => false,
        }
    }

    pub fn status_label(&self) -> &'static str {
        if self.is_active() { "active" } else { "archived" }
    }
}
```

- [ ] **Step 2: Write src/domain/song.rs**

```rust
use crate::api::raw::RawSong;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SongSummary {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub ccli_number: String,
    pub status: String,
}

impl From<RawSong> for SongSummary {
    fn from(raw: RawSong) -> Self {
        let status = raw.status_label().to_string();
        Self {
            id: raw.id,
            title: raw.title,
            artist: raw.artist,
            album: raw.album,
            ccli_number: raw.number,
            status,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn raw(status: serde_json::Value) -> RawSong {
        RawSong {
            id: "s1".into(),
            title: "Amazing Grace".into(),
            artist: "Trad.".into(),
            album: "".into(),
            number: "22025".into(),
            status,
        }
    }

    #[test]
    fn numeric_string_status_active() {
        let s: SongSummary = raw(json!("1")).into();
        assert_eq!(s.status, "active");
        assert_eq!(s.ccli_number, "22025");
    }

    #[test]
    fn numeric_string_status_inactive() {
        let s: SongSummary = raw(json!("0")).into();
        assert_eq!(s.status, "archived");
    }

    #[test]
    fn json_serializes_ccli_field_name() {
        let s: SongSummary = raw(json!("1")).into();
        let v = serde_json::to_value(&s).unwrap();
        assert!(v.get("ccli_number").is_some());
        assert!(v.get("number").is_none());
    }
}
```

- [ ] **Step 3: Update src/domain/mod.rs**

```rust
pub mod category;
pub mod song;
```

- [ ] **Step 4: Add list_songs to src/api/endpoints.rs**

```rust
use crate::api::raw::{CategoriesResponse, RawCategory, RawSong, SongsResponse};
use crate::api::Client;
use crate::error::CliError;

const SONGS_PAGE_SIZE: u32 = 100;

impl Client {
    pub async fn list_categories(&self) -> Result<Vec<RawCategory>, CliError> {
        let resp: CategoriesResponse = self
            .post("songs/categories/getAll", &serde_json::json!({}))
            .await?;
        Ok(resp.categories.category)
    }

    pub async fn list_all_songs(&self) -> Result<Vec<RawSong>, CliError> {
        let mut out = Vec::new();
        let mut page: u32 = 1;
        loop {
            let resp: SongsResponse = self
                .post(
                    "songs/getAll",
                    &serde_json::json!({
                        "page": page,
                        "page_size": SONGS_PAGE_SIZE,
                    }),
                )
                .await?;
            let got = resp.songs.song.len() as u32;
            out.extend(resp.songs.song);
            let per_page = if resp.songs.per_page == 0 { SONGS_PAGE_SIZE } else { resp.songs.per_page };
            if got < per_page || (resp.songs.total > 0 && out.len() as u32 >= resp.songs.total) {
                break;
            }
            page += 1;
            if page > 1000 {
                break; // safety brake
            }
        }
        Ok(out)
    }
}
```

- [ ] **Step 5: Add text renderer to src/output/text.rs**

Append:

```rust
use crate::domain::song::SongSummary;

pub fn write_songs<W: Write>(
    w: &mut W,
    songs: &[SongSummary],
    show_album: bool,
    show_ccli: bool,
) -> io::Result<()> {
    for s in songs {
        write!(w, "{} | {} | {}", s.id, s.title, s.artist)?;
        if show_album {
            write!(w, " | {}", s.album)?;
        }
        if show_ccli {
            write!(w, " | {}", s.ccli_number)?;
        }
        writeln!(w)?;
    }
    Ok(())
}
```

Note: keep both `write_categories` and `write_songs` in the same file. The `use crate::domain::song::SongSummary;` line goes near the existing `use crate::domain::category::Category;`.

- [ ] **Step 6: Write src/commands/songs_list.rs**

```rust
use crate::api::Client;
use crate::cli::SongsListArgs;
use crate::domain::song::SongSummary;
use crate::error::CliError;
use crate::output;

pub async fn run(client: &Client, args: SongsListArgs) -> Result<(), CliError> {
    let raws = client.list_all_songs().await?;
    let all: Vec<SongSummary> = raws.into_iter().map(Into::into).collect();

    let stdout = std::io::stdout();
    let mut lock = stdout.lock();

    let res = if args.json {
        output::json::write_pretty(&mut lock, &all)
    } else {
        let active: Vec<SongSummary> =
            all.into_iter().filter(|s| s.status == "active").collect();
        output::text::write_songs(&mut lock, &active, args.album, args.ccli)
    };
    res.map_err(|e| CliError::Network(format!("write error: {e}")))
}
```

- [ ] **Step 7: Update src/commands/mod.rs**

```rust
pub mod auth_check;
pub mod songs_categories;
pub mod songs_list;
```

- [ ] **Step 8: Dispatch in src/main.rs**

Add to the songs match:

```rust
cli::SongsCommand::List(args) => commands::songs_list::run(&client, args).await,
```

- [ ] **Step 9: Write tests/songs_list.rs**

```rust
mod common;
use common::{bin, mock_server};
use predicates::str::contains;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, ResponseTemplate};

fn page(page: u32, on_this_page: u32, total: u32, songs: Vec<serde_json::Value>) -> serde_json::Value {
    serde_json::json!({
        "status": "ok",
        "songs": {
            "page": page,
            "per_page": 100,
            "total": total,
            "on_this_page": on_this_page,
            "song": songs
        }
    })
}

fn song(id: &str, title: &str, artist: &str, status: &str, album: &str, number: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id, "title": title, "artist": artist,
        "album": album, "number": number, "status": status,
    })
}

#[tokio::test]
async fn paginates_and_filters_active_in_text_mode() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getAll.json"))
        .and(body_partial_json(serde_json::json!({ "page": 1 })))
        .respond_with(ResponseTemplate::new(200).set_body_json(page(
            1,
            100,
            150,
            (0..100)
                .map(|i| song(&format!("s{i}"), &format!("T{i}"), "A", "1", "Al", &format!("{i}")))
                .collect(),
        )))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/songs/getAll.json"))
        .and(body_partial_json(serde_json::json!({ "page": 2 })))
        .respond_with(ResponseTemplate::new(200).set_body_json(page(
            2,
            50,
            150,
            (100..150)
                .map(|i| song(
                    &format!("s{i}"),
                    &format!("T{i}"),
                    "A",
                    if i % 2 == 0 { "1" } else { "0" },
                    "Al",
                    &format!("{i}"),
                ))
                .collect(),
        )))
        .mount(&server)
        .await;

    let out = bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(out).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    // 100 active on page 1 + 25 active on page 2 (even indices 100..150)
    assert_eq!(lines.len(), 125);
    assert!(lines[0].contains("s0 | T0 | A"));
}

#[tokio::test]
async fn album_and_ccli_columns() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getAll.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page(
            1,
            1,
            1,
            vec![song("s1", "Grace", "Trad.", "1", "Hymnal", "22025")],
        )))
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "list", "--album", "--ccli"])
        .assert()
        .success()
        .stdout(contains("s1 | Grace | Trad. | Hymnal | 22025"));
}

#[tokio::test]
async fn json_includes_inactive_songs() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getAll.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page(
            1,
            2,
            2,
            vec![
                song("s1", "Active", "A", "1", "", ""),
                song("s2", "Archived", "B", "0", "", ""),
            ],
        )))
        .mount(&server)
        .await;

    let out = bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "list", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let parsed: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(parsed.as_array().unwrap().len(), 2);
    assert_eq!(parsed[1]["status"], "archived");
}
```

- [ ] **Step 10: Run**

Run: `cargo test --test songs_list`
Expected: 3 passed.

Run: `cargo test`
Expected: all tests across suites pass.

- [ ] **Step 11: Commit**

```bash
git add src/ tests/songs_list.rs
git commit -m "feat(songs): add songs list with pagination and column flags"
```

---

## Task 9: Arrangement domain + raw types

**Files:**
- Create: `/home/chris/dev/elvanto-cli/src/domain/arrangement.rs`
- Modify: `/home/chris/dev/elvanto-cli/src/api/raw.rs`
- Modify: `/home/chris/dev/elvanto-cli/src/domain/mod.rs`
- Test: inline in `src/domain/arrangement.rs`

`songs/arrangements/getAll` returns `{arrangements:{arrangement:[{id,name,sequence,bpm,duration,chord_pro,lyrics,keys:{key:[{id,starting,ending,...}]}}]}}`. Single-song `songs/getInfo` embeds arrangements inline on the song object.

- [ ] **Step 1: Add raw types to src/api/raw.rs**

Append:

```rust
#[derive(Debug, Deserialize, Default, Clone)]
pub struct ArrangementList {
    #[serde(default)]
    pub arrangement: Vec<RawArrangement>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawArrangement {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub sequence: String,
    #[serde(default)]
    pub bpm: String,
    #[serde(default)]
    pub duration: String,
    /// Chord chart text. Field name varies by Elvanto endpoint version; accept both.
    #[serde(default, alias = "chord_chart")]
    pub chord_pro: String,
    #[serde(default)]
    pub lyrics: String,
    #[serde(default)]
    pub keys: KeyList,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct KeyList {
    #[serde(default)]
    pub key: Vec<RawKey>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawKey {
    pub id: String,
    #[serde(default, alias = "starting_key")]
    pub starting: String,
    #[serde(default, alias = "ending_key")]
    pub ending: String,
}

#[derive(Debug, Deserialize)]
pub struct ArrangementsResponse {
    #[serde(default)]
    pub arrangements: ArrangementList,
}

#[derive(Debug, Deserialize)]
pub struct ArrangementInfoResponse {
    #[serde(default)]
    pub arrangement: RawArrangement,
}
```

- [ ] **Step 2: Write src/domain/arrangement.rs**

```rust
use crate::api::raw::{RawArrangement, RawKey};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Key {
    pub id: String,
    pub starting: String,
    pub ending: Option<String>,
}

impl From<RawKey> for Key {
    fn from(raw: RawKey) -> Self {
        let ending = if raw.ending.is_empty() { None } else { Some(raw.ending) };
        Self { id: raw.id, starting: raw.starting, ending }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Arrangement {
    pub id: String,
    pub name: String,
    pub sequence: Option<String>,
    pub bpm: Option<String>,
    pub duration: Option<String>,
    pub keys: Vec<Key>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lyrics: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chord_chart: Option<String>,
}

fn none_if_empty(s: String) -> Option<String> {
    if s.is_empty() { None } else { Some(s) }
}

impl From<RawArrangement> for Arrangement {
    fn from(raw: RawArrangement) -> Self {
        Self {
            id: raw.id,
            name: raw.name,
            sequence: none_if_empty(raw.sequence),
            bpm: none_if_empty(raw.bpm),
            duration: none_if_empty(raw.duration),
            keys: raw.keys.key.into_iter().map(Into::into).collect(),
            lyrics: none_if_empty(raw.lyrics),
            chord_chart: none_if_empty(raw.chord_pro),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::raw::KeyList;

    #[test]
    fn empty_ending_key_becomes_none() {
        let raw = RawKey { id: "k1".into(), starting: "G".into(), ending: String::new() };
        let key: Key = raw.into();
        assert_eq!(key.ending, None);
    }

    #[test]
    fn empty_lyrics_chord_chart_become_none() {
        let raw = RawArrangement {
            id: "a1".into(),
            name: "Default".into(),
            keys: KeyList { key: vec![] },
            ..Default::default()
        };
        let arr: Arrangement = raw.into();
        assert!(arr.lyrics.is_none());
        assert!(arr.chord_chart.is_none());
    }
}
```

- [ ] **Step 3: Update src/domain/mod.rs**

```rust
pub mod arrangement;
pub mod category;
pub mod song;
```

- [ ] **Step 4: Run**

Run: `cargo test --lib domain::arrangement::`
Expected: 2 passed.

Run: `cargo build`
Expected: clean.

- [ ] **Step 5: Commit**

```bash
git add src/domain/ src/api/raw.rs
git commit -m "feat(domain): add Arrangement and Key normalized types"
```

---

## Task 10: `songs show` — curated, --full, --json, --files

**Files:**
- Modify: `/home/chris/dev/elvanto-cli/src/api/raw.rs` (add `RawSongDetail`)
- Modify: `/home/chris/dev/elvanto-cli/src/api/endpoints.rs` (add `get_song_info`)
- Create: `/home/chris/dev/elvanto-cli/src/commands/songs_show.rs`
- Modify: `/home/chris/dev/elvanto-cli/src/output/text.rs`
- Modify: `/home/chris/dev/elvanto-cli/src/domain/song.rs` (add `SongDetail` + From impl)
- Modify: `/home/chris/dev/elvanto-cli/src/commands/mod.rs`
- Modify: `/home/chris/dev/elvanto-cli/src/main.rs` (dispatch)
- Test: `tests/songs_show.rs`

`songs/getInfo` returns `{songs:{song:[{...full song with arrangements...}]}}`. Even for a single id Elvanto wraps in a list; we read the first element.

- [ ] **Step 1: Append raw types in src/api/raw.rs**

```rust
#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawSongDetail {
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub artist: String,
    #[serde(default)]
    pub album: String,
    #[serde(default)]
    pub number: String,
    #[serde(default)]
    pub status: serde_json::Value,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub sequence: String,
    #[serde(default)]
    pub bpm: String,
    #[serde(default)]
    pub duration: String,
    #[serde(default)]
    pub learn: serde_json::Value,
    #[serde(default)]
    pub allow_downloads: serde_json::Value,
    #[serde(default)]
    pub categories: CategoryList,
    #[serde(default)]
    pub locations: LocationList,
    #[serde(default)]
    pub arrangements: ArrangementList,
    /// Present only when `files=1` requested.
    #[serde(default)]
    pub files: serde_json::Value,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct LocationList {
    #[serde(default)]
    pub location: Vec<RawLocation>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawLocation {
    pub id: String,
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct SongInfoResponse {
    pub songs: SongInfoInner,
}

#[derive(Debug, Deserialize)]
pub struct SongInfoInner {
    #[serde(default)]
    pub song: Vec<RawSongDetail>,
}
```

Helper for numeric-boolean fields. Append:

```rust
pub fn truthy(v: &serde_json::Value) -> bool {
    match v {
        serde_json::Value::Bool(b) => *b,
        serde_json::Value::Number(n) => n.as_i64().map(|i| i != 0).unwrap_or(false),
        serde_json::Value::String(s) => s == "1" || s.eq_ignore_ascii_case("true"),
        _ => false,
    }
}
```

- [ ] **Step 2: Add endpoint method to src/api/endpoints.rs**

Append inside `impl Client { ... }`:

```rust
pub async fn get_song_info(&self, id: &str, with_files: bool) -> Result<crate::api::raw::RawSongDetail, CliError> {
    let body = if with_files {
        serde_json::json!({ "id": id, "files": 1 })
    } else {
        serde_json::json!({ "id": id })
    };
    let resp: crate::api::raw::SongInfoResponse =
        self.post("songs/getInfo", &body).await?;
    resp.songs
        .song
        .into_iter()
        .next()
        .ok_or_else(|| CliError::Api { code: 404, message: format!("song {id} not found") })
}
```

- [ ] **Step 3: Append SongDetail to src/domain/song.rs**

```rust
use crate::api::raw::{truthy, RawSongDetail};
use crate::domain::arrangement::Arrangement;
use crate::domain::category::Category;

#[derive(Debug, Clone, Serialize)]
pub struct Location {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SongDetail {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub ccli_number: String,
    pub status: String,
    pub notes: Option<String>,
    pub sequence: Option<String>,
    pub bpm: Option<String>,
    pub duration: Option<String>,
    pub learn: bool,
    pub allow_downloads: bool,
    pub categories: Vec<Category>,
    pub locations: Vec<Location>,
    pub arrangements: Vec<Arrangement>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<serde_json::Value>,
}

fn none_if_empty(s: String) -> Option<String> {
    if s.is_empty() { None } else { Some(s) }
}

impl From<RawSongDetail> for SongDetail {
    fn from(raw: RawSongDetail) -> Self {
        let status = if matches!(&raw.status, v if truthy(v)) { "active" } else { "archived" }.to_string();
        let files = match raw.files {
            serde_json::Value::Null => None,
            other => Some(other),
        };
        Self {
            id: raw.id,
            title: raw.title,
            artist: raw.artist,
            album: raw.album,
            ccli_number: raw.number,
            status,
            notes: none_if_empty(raw.notes),
            sequence: none_if_empty(raw.sequence),
            bpm: none_if_empty(raw.bpm),
            duration: none_if_empty(raw.duration),
            learn: truthy(&raw.learn),
            allow_downloads: truthy(&raw.allow_downloads),
            categories: raw.categories.category.into_iter().map(Into::into).collect(),
            locations: raw.locations.location.into_iter().map(|l| Location { id: l.id, name: l.name }).collect(),
            arrangements: raw.arrangements.arrangement.into_iter().map(Into::into).collect(),
            files,
        }
    }
}
```

Note: there is already a `fn none_if_empty` in `domain/arrangement.rs`. Avoid duplication: lift it to `src/domain/mod.rs` as `pub(crate) fn none_if_empty(s: String) -> Option<String>` and have both files use `use crate::domain::none_if_empty;`. Apply that refactor in this step.

Final `src/domain/mod.rs`:

```rust
pub mod arrangement;
pub mod category;
pub mod song;

pub(crate) fn none_if_empty(s: String) -> Option<String> {
    if s.is_empty() { None } else { Some(s) }
}
```

Delete the local `none_if_empty` from `arrangement.rs` and `song.rs`; replace their callsites with `crate::domain::none_if_empty(...)`.

- [ ] **Step 4: Add text renderers in src/output/text.rs**

Append:

```rust
use crate::domain::song::SongDetail;

pub fn write_song_curated<W: Write>(w: &mut W, song: &SongDetail) -> io::Result<()> {
    writeln!(w, "Title:       {}", song.title)?;
    writeln!(w, "Artist:      {}", song.artist)?;
    writeln!(w, "CCLI number: {}", song.ccli_number)?;
    writeln!(w, "Status:      {}", song.status)?;

    let first_line = song
        .arrangements
        .iter()
        .find_map(|a| a.lyrics.as_deref())
        .and_then(|l| l.lines().find(|l| !l.trim().is_empty()))
        .unwrap_or("");
    writeln!(w, "First line:  {first_line}")?;

    writeln!(w, "Arrangements:")?;
    for arr in &song.arrangements {
        let keys: Vec<String> = arr
            .keys
            .iter()
            .map(|k| match &k.ending {
                Some(e) => format!("{}→{}", k.starting, e),
                None => k.starting.clone(),
            })
            .collect();
        let keys_str = if keys.is_empty() { "—".into() } else { keys.join(", ") };
        writeln!(w, "  - {} [{}]", arr.name, keys_str)?;
    }
    Ok(())
}

pub fn write_song_full<W: Write>(w: &mut W, song: &SongDetail) -> io::Result<()> {
    writeln!(w, "Title:           {}", song.title)?;
    writeln!(w, "Artist:          {}", song.artist)?;
    writeln!(w, "Album:           {}", song.album)?;
    writeln!(w, "CCLI number:     {}", song.ccli_number)?;
    writeln!(w, "Status:          {}", song.status)?;
    if let Some(v) = &song.sequence {
        writeln!(w, "Sequence:        {v}")?;
    }
    if let Some(v) = &song.bpm {
        writeln!(w, "BPM:             {v}")?;
    }
    if let Some(v) = &song.duration {
        writeln!(w, "Duration:        {v}")?;
    }
    writeln!(w, "Learn:           {}", song.learn)?;
    writeln!(w, "Allow downloads: {}", song.allow_downloads)?;
    if !song.categories.is_empty() {
        let names: Vec<&str> = song.categories.iter().map(|c| c.name.as_str()).collect();
        writeln!(w, "Categories:      {}", names.join(", "))?;
    }
    if !song.locations.is_empty() {
        let names: Vec<&str> = song.locations.iter().map(|c| c.name.as_str()).collect();
        writeln!(w, "Locations:       {}", names.join(", "))?;
    }
    if let Some(n) = &song.notes {
        writeln!(w, "Notes:           {n}")?;
    }
    writeln!(w, "Arrangements:")?;
    for arr in &song.arrangements {
        let keys: Vec<String> = arr.keys.iter().map(|k| k.starting.clone()).collect();
        writeln!(w, "  - {} [{}]", arr.name, keys.join(", "))?;
    }
    Ok(())
}
```

- [ ] **Step 5: Write src/commands/songs_show.rs**

```rust
use crate::api::Client;
use crate::cli::SongsShowArgs;
use crate::domain::song::SongDetail;
use crate::error::CliError;
use crate::output;

pub async fn run(client: &Client, args: SongsShowArgs) -> Result<(), CliError> {
    let want_files = args.files && args.json; // --files only meaningful with --json in V1
    let raw = client.get_song_info(&args.id, want_files).await?;
    let detail: SongDetail = raw.into();

    let stdout = std::io::stdout();
    let mut lock = stdout.lock();

    let res = if args.json {
        output::json::write_pretty(&mut lock, &detail)
    } else if args.full {
        output::text::write_song_full(&mut lock, &detail)
    } else {
        output::text::write_song_curated(&mut lock, &detail)
    };
    res.map_err(|e| CliError::Network(format!("write error: {e}")))
}
```

- [ ] **Step 6: Update src/commands/mod.rs**

```rust
pub mod auth_check;
pub mod songs_categories;
pub mod songs_list;
pub mod songs_show;
```

- [ ] **Step 7: Dispatch in src/main.rs**

```rust
cli::SongsCommand::Show(args) => commands::songs_show::run(&client, args).await,
```

- [ ] **Step 8: Write tests/songs_show.rs**

```rust
mod common;
use common::{bin, mock_server};
use predicates::str::contains;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, ResponseTemplate};

fn song_body(extra: serde_json::Value) -> serde_json::Value {
    let mut song = serde_json::json!({
        "id": "s-1",
        "title": "Amazing Grace",
        "artist": "Trad.",
        "album": "Hymnal",
        "number": "22025",
        "status": "1",
        "sequence": "V1 C V2 C",
        "bpm": "78",
        "duration": "180",
        "learn": "1",
        "allow_downloads": "0",
        "categories": { "category": [ { "id": "c1", "name": "Hymns" } ] },
        "locations": { "location": [ { "id": "l1", "name": "Main" } ] },
        "arrangements": {
            "arrangement": [
                {
                    "id": "a1",
                    "name": "Default",
                    "sequence": "V1 C V2 C",
                    "bpm": "78",
                    "duration": "180",
                    "lyrics": "Amazing grace how sweet the sound\nThat saved a wretch like me",
                    "chord_pro": "[G]Amazing [C]grace",
                    "keys": { "key": [ { "id": "k1", "starting": "G", "ending": "" } ] }
                }
            ]
        }
    });
    if let Some(obj) = song.as_object_mut() {
        if let Some(more) = extra.as_object() {
            for (k, v) in more {
                obj.insert(k.clone(), v.clone());
            }
        }
    }
    serde_json::json!({
        "status": "ok",
        "songs": { "song": [ song ] }
    })
}

#[tokio::test]
async fn curated_output_default() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getInfo.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(song_body(serde_json::json!({}))))
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "show", "s-1"])
        .assert()
        .success()
        .stdout(
            contains("Title:       Amazing Grace")
                .and(contains("CCLI number: 22025"))
                .and(contains("Status:      active"))
                .and(contains("First line:  Amazing grace how sweet the sound"))
                .and(contains("- Default [G]")),
        );
}

#[tokio::test]
async fn full_output() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getInfo.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(song_body(serde_json::json!({}))))
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "show", "s-1", "--full"])
        .assert()
        .success()
        .stdout(
            contains("BPM:             78")
                .and(contains("Categories:      Hymns"))
                .and(contains("Locations:       Main"))
                .and(contains("Learn:           true"))
                .and(contains("Allow downloads: false")),
        );
}

#[tokio::test]
async fn json_output_normalized() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getInfo.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(song_body(serde_json::json!({}))))
        .mount(&server)
        .await;

    let out = bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "show", "s-1", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ccli_number"], "22025");
    assert_eq!(v["status"], "active");
    assert_eq!(v["learn"], true);
    assert_eq!(v["allow_downloads"], false);
    assert_eq!(v["arrangements"][0]["name"], "Default");
    assert_eq!(v["arrangements"][0]["keys"][0]["starting"], "G");
    assert!(v.get("files").is_none(), "files omitted when --files not passed");
}

#[tokio::test]
async fn files_flag_with_json_requests_files_and_includes_them() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getInfo.json"))
        .and(body_partial_json(serde_json::json!({ "id": "s-1", "files": 1 })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(song_body(serde_json::json!({
                "files": { "file": [ { "id": "f1", "filename": "lead.pdf" } ] }
            }))),
        )
        .mount(&server)
        .await;

    let out = bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "show", "s-1", "--json", "--files"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["files"]["file"][0]["filename"], "lead.pdf");
}

#[tokio::test]
async fn not_found_returns_exit_1() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getInfo.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "ok",
            "songs": { "song": [] }
        })))
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "show", "missing-id"])
        .assert()
        .failure()
        .code(1)
        .stderr(contains("song missing-id not found"));
}
```

- [ ] **Step 9: Run**

Run: `cargo test --test songs_show`
Expected: 5 passed.

Run: `cargo test`
Expected: all suites pass.

- [ ] **Step 10: Commit**

```bash
git add src/ tests/songs_show.rs
git commit -m "feat(songs): add songs show with curated, full, json and files modes"
```

---

## Task 11: Arrangement selection helper

**Files:**
- Create: `/home/chris/dev/elvanto-cli/src/arrangement_select.rs`
- Modify: `/home/chris/dev/elvanto-cli/src/main.rs` (`mod arrangement_select;`)
- Test: inline

Rule per CONTEXT.md: when `--arrangement` is missing, pick the one named "Default" (case-insensitive); otherwise the first; if none, error. When `--arrangement <name>` is given, pick exactly by case-insensitive match; if missing, error and list the available names.

- [ ] **Step 1: Write src/arrangement_select.rs**

```rust
use crate::domain::arrangement::Arrangement;
use crate::error::CliError;

pub struct Selection<'a> {
    pub chosen: &'a Arrangement,
    pub others: Vec<&'a Arrangement>,
}

pub fn select<'a>(
    arrangements: &'a [Arrangement],
    requested: Option<&str>,
) -> Result<Selection<'a>, CliError> {
    if arrangements.is_empty() {
        return Err(CliError::Api { code: 0, message: "song has no arrangements".into() });
    }

    let chosen_idx = match requested {
        Some(name) => arrangements
            .iter()
            .position(|a| a.name.eq_ignore_ascii_case(name))
            .ok_or_else(|| {
                let available: Vec<&str> = arrangements.iter().map(|a| a.name.as_str()).collect();
                CliError::Usage(format!(
                    "arrangement {:?} not found; available: {}",
                    name,
                    available.join(", ")
                ))
            })?,
        None => arrangements
            .iter()
            .position(|a| a.name.eq_ignore_ascii_case("Default"))
            .unwrap_or(0),
    };

    let chosen = &arrangements[chosen_idx];
    let others = arrangements
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != chosen_idx)
        .map(|(_, a)| a)
        .collect();
    Ok(Selection { chosen, others })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::arrangement::Arrangement;

    fn arr(name: &str) -> Arrangement {
        Arrangement {
            id: name.into(),
            name: name.into(),
            sequence: None,
            bpm: None,
            duration: None,
            keys: vec![],
            lyrics: None,
            chord_chart: None,
        }
    }

    #[test]
    fn empty_list_errors() {
        assert!(matches!(select(&[], None), Err(CliError::Api { .. })));
    }

    #[test]
    fn default_is_picked_when_no_request() {
        let list = vec![arr("Acoustic"), arr("Default"), arr("Live")];
        let sel = select(&list, None).unwrap();
        assert_eq!(sel.chosen.name, "Default");
        assert_eq!(sel.others.len(), 2);
    }

    #[test]
    fn falls_back_to_first_without_default() {
        let list = vec![arr("Acoustic"), arr("Live")];
        let sel = select(&list, None).unwrap();
        assert_eq!(sel.chosen.name, "Acoustic");
    }

    #[test]
    fn requested_match_case_insensitive() {
        let list = vec![arr("Acoustic"), arr("Default")];
        let sel = select(&list, Some("acoustic")).unwrap();
        assert_eq!(sel.chosen.name, "Acoustic");
    }

    #[test]
    fn missing_request_errors_with_usage() {
        let list = vec![arr("Default")];
        match select(&list, Some("Live")) {
            Err(CliError::Usage(msg)) => {
                assert!(msg.contains("not found"));
                assert!(msg.contains("Default"));
            }
            other => panic!("expected Usage error, got {other:?}"),
        }
    }
}
```

- [ ] **Step 2: Wire in src/main.rs**

```rust
mod arrangement_select;
```

- [ ] **Step 3: Run**

Run: `cargo test --lib arrangement_select::`
Expected: 5 passed.

- [ ] **Step 4: Commit**

```bash
git add src/arrangement_select.rs src/main.rs
git commit -m "feat(arrangement): add Default-or-first selection helper"
```

---

## Task 12: `songs lyrics` command

**Files:**
- Create: `/home/chris/dev/elvanto-cli/src/commands/songs_lyrics.rs`
- Modify: `/home/chris/dev/elvanto-cli/src/commands/mod.rs`
- Modify: `/home/chris/dev/elvanto-cli/src/main.rs`
- Test: `tests/songs_lyrics.rs`

We already fetch full arrangements as part of `songs/getInfo`. Re-use it: `songs lyrics` and `songs chart` both call `get_song_info(id, false)` and read the chosen arrangement's `lyrics` / `chord_chart` directly. No separate `arrangements/getInfo` round trip needed for V1.

- [ ] **Step 1: Write src/commands/songs_lyrics.rs**

```rust
use crate::api::Client;
use crate::arrangement_select;
use crate::cli::SongsLyricsArgs;
use crate::domain::song::SongDetail;
use crate::error::CliError;

pub async fn run(client: &Client, args: SongsLyricsArgs) -> Result<(), CliError> {
    let raw = client.get_song_info(&args.id, false).await?;
    let detail: SongDetail = raw.into();

    let sel = arrangement_select::select(&detail.arrangements, args.arrangement.as_deref())?;

    let lyrics = sel
        .chosen
        .lyrics
        .as_deref()
        .ok_or_else(|| CliError::Api {
            code: 0,
            message: format!("arrangement {:?} has no lyrics", sel.chosen.name),
        })?;
    println!("{lyrics}");

    if !sel.others.is_empty() {
        let names: Vec<&str> = sel.others.iter().map(|a| a.name.as_str()).collect();
        eprintln!("\n(other arrangements: {})", names.join(", "));
    }
    Ok(())
}
```

- [ ] **Step 2: Update src/commands/mod.rs**

```rust
pub mod auth_check;
pub mod songs_categories;
pub mod songs_list;
pub mod songs_lyrics;
pub mod songs_show;
```

- [ ] **Step 3: Dispatch in src/main.rs**

```rust
cli::SongsCommand::Lyrics(args) => commands::songs_lyrics::run(&client, args).await,
```

- [ ] **Step 4: Write tests/songs_lyrics.rs**

```rust
mod common;
use common::{bin, mock_server};
use predicates::str::contains;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

fn song_with_arrangements(arrs: Vec<serde_json::Value>) -> serde_json::Value {
    serde_json::json!({
        "status": "ok",
        "songs": { "song": [ {
            "id": "s1",
            "title": "Title",
            "status": "1",
            "arrangements": { "arrangement": arrs }
        } ] }
    })
}

fn arr(name: &str, lyrics: &str) -> serde_json::Value {
    serde_json::json!({
        "id": name, "name": name, "lyrics": lyrics,
        "keys": { "key": [] }
    })
}

#[tokio::test]
async fn picks_default_arrangement() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getInfo.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(song_with_arrangements(vec![
            arr("Acoustic", "Acoustic lyrics"),
            arr("Default", "Default lyrics"),
        ])))
        .mount(&server)
        .await;

    let assert = bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "lyrics", "s1"])
        .assert()
        .success();
    assert.stdout(contains("Default lyrics"));
}

#[tokio::test]
async fn hints_other_arrangements_on_stderr() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getInfo.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(song_with_arrangements(vec![
            arr("Default", "Default lyrics"),
            arr("Acoustic", "Acoustic lyrics"),
            arr("Live", "Live lyrics"),
        ])))
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "lyrics", "s1"])
        .assert()
        .success()
        .stderr(contains("other arrangements: Acoustic, Live"));
}

#[tokio::test]
async fn arrangement_override() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getInfo.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(song_with_arrangements(vec![
            arr("Default", "Default lyrics"),
            arr("Acoustic", "Acoustic lyrics"),
        ])))
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "lyrics", "s1", "--arrangement", "Acoustic"])
        .assert()
        .success()
        .stdout(contains("Acoustic lyrics"));
}

#[tokio::test]
async fn missing_arrangement_is_usage_error() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getInfo.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(song_with_arrangements(vec![
            arr("Default", "Default lyrics"),
        ])))
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "lyrics", "s1", "--arrangement", "Live"])
        .assert()
        .failure()
        .code(2)
        .stderr(contains("not found"));
}
```

- [ ] **Step 5: Run**

Run: `cargo test --test songs_lyrics`
Expected: 4 passed.

- [ ] **Step 6: Commit**

```bash
git add src/ tests/songs_lyrics.rs
git commit -m "feat(songs): add songs lyrics command with arrangement selection"
```

---

## Task 13: Transposition module (no API call)

**Files:**
- Create: `/home/chris/dev/elvanto-cli/src/transpose.rs`
- Modify: `/home/chris/dev/elvanto-cli/src/main.rs` (`mod transpose;`)
- Test: inline

Elvanto's `songs/arrangements/getInfo` accepts a `chord_chart_key` parameter naming the target key (e.g. `G`, `F#`, `Bb`). The CLI accepts both named keys and relative semitone offsets like `-2` or `+3`. We resolve offsets against the arrangement's starting key (from `keys[0].starting`).

- [ ] **Step 1: Write src/transpose.rs**

```rust
use crate::error::CliError;

const KEYS_SHARP: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
const KEYS_FLAT: [&str; 12] = ["C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B"];

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Request {
    Named(String),
    Offset(i32),
}

pub fn parse(input: &str) -> Result<Request, CliError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(CliError::Usage("--transpose value is empty".into()));
    }
    let first = trimmed.chars().next().unwrap();
    if first == '+' || first == '-' || first.is_ascii_digit() {
        let n: i32 = trimmed
            .parse()
            .map_err(|_| CliError::Usage(format!("invalid transpose offset {trimmed:?}")))?;
        return Ok(Request::Offset(n));
    }
    let normalized = normalize_key(trimmed)
        .ok_or_else(|| CliError::Usage(format!("invalid key {trimmed:?}")))?;
    Ok(Request::Named(normalized))
}

pub fn resolve(req: &Request, starting: &str) -> Result<String, CliError> {
    match req {
        Request::Named(k) => Ok(k.clone()),
        Request::Offset(n) => {
            let base = key_index(starting)
                .ok_or_else(|| CliError::Api {
                    code: 0,
                    message: format!("cannot transpose: unknown starting key {starting:?}"),
                })?;
            let prefer_flats = starting.contains('b');
            let idx = ((base as i32 + *n).rem_euclid(12)) as usize;
            let table = if prefer_flats { KEYS_FLAT } else { KEYS_SHARP };
            Ok(table[idx].to_string())
        }
    }
}

fn normalize_key(s: &str) -> Option<String> {
    let upper = capitalize(s);
    if KEYS_SHARP.contains(&upper.as_str()) || KEYS_FLAT.contains(&upper.as_str()) {
        Some(upper)
    } else {
        None
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_ascii_uppercase().to_string() + chars.as_str(),
    }
}

fn key_index(k: &str) -> Option<usize> {
    KEYS_SHARP
        .iter()
        .position(|x| x.eq_ignore_ascii_case(k))
        .or_else(|| KEYS_FLAT.iter().position(|x| x.eq_ignore_ascii_case(k)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_named() {
        assert_eq!(parse("G").unwrap(), Request::Named("G".into()));
        assert_eq!(parse("f#").unwrap(), Request::Named("F#".into()));
        assert_eq!(parse("Bb").unwrap(), Request::Named("Bb".into()));
    }

    #[test]
    fn parse_offset() {
        assert_eq!(parse("+3").unwrap(), Request::Offset(3));
        assert_eq!(parse("-2").unwrap(), Request::Offset(-2));
        assert_eq!(parse("5").unwrap(), Request::Offset(5));
    }

    #[test]
    fn parse_invalid() {
        assert!(matches!(parse("Q"), Err(CliError::Usage(_))));
        assert!(matches!(parse(""), Err(CliError::Usage(_))));
    }

    #[test]
    fn resolve_offset_uses_sharps() {
        let r = resolve(&Request::Offset(2), "G").unwrap();
        assert_eq!(r, "A");
    }

    #[test]
    fn resolve_offset_wraps() {
        let r = resolve(&Request::Offset(13), "C").unwrap();
        assert_eq!(r, "C#");
        let r2 = resolve(&Request::Offset(-1), "C").unwrap();
        assert_eq!(r2, "B");
    }

    #[test]
    fn resolve_offset_uses_flats_when_starting_has_flat() {
        let r = resolve(&Request::Offset(2), "Bb").unwrap();
        assert_eq!(r, "C");
        let r2 = resolve(&Request::Offset(1), "Bb").unwrap();
        assert_eq!(r2, "B");
    }

    #[test]
    fn resolve_named_passes_through() {
        let r = resolve(&Request::Named("F#".into()), "G").unwrap();
        assert_eq!(r, "F#");
    }
}
```

- [ ] **Step 2: Wire in src/main.rs**

```rust
mod transpose;
```

- [ ] **Step 3: Run**

Run: `cargo test --lib transpose::`
Expected: 7 passed.

- [ ] **Step 4: Commit**

```bash
git add src/transpose.rs src/main.rs
git commit -m "feat(transpose): add key + offset transpose parser and resolver"
```

---

## Task 14: `songs chart` command (with optional transpose)

**Files:**
- Modify: `/home/chris/dev/elvanto-cli/src/api/endpoints.rs` (add `get_arrangement_info`)
- Create: `/home/chris/dev/elvanto-cli/src/commands/songs_chart.rs`
- Modify: `/home/chris/dev/elvanto-cli/src/commands/mod.rs`
- Modify: `/home/chris/dev/elvanto-cli/src/main.rs`
- Test: `tests/songs_chart.rs`

When `--transpose` is supplied, call `songs/arrangements/getInfo` with `id=<arrangement-id>` and `chord_chart_key=<resolved>` to get the transposed chord chart from Elvanto. When `--transpose` is absent, the chord chart already on the song detail's arrangement is used (no extra request).

- [ ] **Step 1: Append endpoint method to src/api/endpoints.rs**

Inside the same `impl Client { ... }`:

```rust
pub async fn get_arrangement_info(
    &self,
    arrangement_id: &str,
    chord_chart_key: Option<&str>,
) -> Result<crate::api::raw::RawArrangement, CliError> {
    let mut body = serde_json::json!({ "id": arrangement_id });
    if let Some(k) = chord_chart_key {
        body["chord_chart_key"] = serde_json::Value::String(k.to_string());
    }
    let resp: crate::api::raw::ArrangementInfoResponse = self
        .post("songs/arrangements/getInfo", &body)
        .await?;
    Ok(resp.arrangement)
}
```

- [ ] **Step 2: Write src/commands/songs_chart.rs**

```rust
use crate::api::Client;
use crate::arrangement_select;
use crate::cli::SongsChartArgs;
use crate::domain::arrangement::Arrangement;
use crate::domain::song::SongDetail;
use crate::error::CliError;
use crate::transpose;

pub async fn run(client: &Client, args: SongsChartArgs) -> Result<(), CliError> {
    let raw_song = client.get_song_info(&args.id, false).await?;
    let detail: SongDetail = raw_song.into();
    let sel = arrangement_select::select(&detail.arrangements, args.arrangement.as_deref())?;
    let chosen = sel.chosen;

    let chart = match args.transpose.as_deref() {
        None => chosen
            .chord_chart
            .clone()
            .ok_or_else(|| CliError::Api {
                code: 0,
                message: format!("arrangement {:?} has no chord chart", chosen.name),
            })?,
        Some(input) => {
            let req = transpose::parse(input)?;
            let starting = chosen
                .keys
                .first()
                .map(|k| k.starting.as_str())
                .ok_or_else(|| CliError::Api {
                    code: 0,
                    message: format!("arrangement {:?} has no key", chosen.name),
                })?;
            let target = transpose::resolve(&req, starting)?;
            let raw_arr = client.get_arrangement_info(&chosen.id, Some(&target)).await?;
            let arr: Arrangement = raw_arr.into();
            arr.chord_chart.ok_or_else(|| CliError::Api {
                code: 0,
                message: format!("Elvanto returned no transposed chord chart for {target}"),
            })?
        }
    };

    println!("{chart}");

    if !sel.others.is_empty() {
        let names: Vec<&str> = sel.others.iter().map(|a| a.name.as_str()).collect();
        eprintln!("\n(other arrangements: {})", names.join(", "));
    }
    Ok(())
}
```

- [ ] **Step 3: Update src/commands/mod.rs**

```rust
pub mod auth_check;
pub mod songs_categories;
pub mod songs_chart;
pub mod songs_list;
pub mod songs_lyrics;
pub mod songs_show;
```

- [ ] **Step 4: Dispatch in src/main.rs**

```rust
cli::SongsCommand::Chart(args) => commands::songs_chart::run(&client, args).await,
```

At this point all songs subcommands are wired. Remove the catch-all `_ => Err(CliError::Usage("not implemented yet".into())),` from the songs match block.

- [ ] **Step 5: Write tests/songs_chart.rs**

```rust
mod common;
use common::{bin, mock_server};
use predicates::str::contains;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, ResponseTemplate};

fn song_with_chart(name: &str, chord_chart: &str, starting_key: &str) -> serde_json::Value {
    serde_json::json!({
        "status": "ok",
        "songs": { "song": [ {
            "id": "s1",
            "title": "T",
            "status": "1",
            "arrangements": { "arrangement": [ {
                "id": "a1",
                "name": name,
                "chord_pro": chord_chart,
                "keys": { "key": [ { "id": "k1", "starting": starting_key, "ending": "" } ] }
            } ] }
        } ] }
    })
}

#[tokio::test]
async fn chart_without_transpose_uses_song_detail() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getInfo.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(song_with_chart(
            "Default",
            "[G]Hello",
            "G",
        )))
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "chart", "s1"])
        .assert()
        .success()
        .stdout(contains("[G]Hello"));
}

#[tokio::test]
async fn chart_with_named_transpose_calls_arrangement_info() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getInfo.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(song_with_chart(
            "Default",
            "[G]Hello",
            "G",
        )))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/songs/arrangements/getInfo.json"))
        .and(body_partial_json(serde_json::json!({
            "id": "a1",
            "chord_chart_key": "A"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "ok",
            "arrangement": {
                "id": "a1",
                "name": "Default",
                "chord_pro": "[A]Hello",
                "keys": { "key": [] }
            }
        })))
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "chart", "s1", "--transpose", "A"])
        .assert()
        .success()
        .stdout(contains("[A]Hello"));
}

#[tokio::test]
async fn chart_with_offset_transpose_resolves_against_starting_key() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getInfo.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(song_with_chart(
            "Default",
            "[G]Hello",
            "G",
        )))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/songs/arrangements/getInfo.json"))
        .and(body_partial_json(serde_json::json!({
            "id": "a1",
            "chord_chart_key": "A"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "ok",
            "arrangement": {
                "id": "a1",
                "name": "Default",
                "chord_pro": "[A]Hello",
                "keys": { "key": [] }
            }
        })))
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "chart", "s1", "--transpose", "+2"])
        .assert()
        .success()
        .stdout(contains("[A]Hello"));
}

#[tokio::test]
async fn invalid_transpose_value_is_usage_error() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/songs/getInfo.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(song_with_chart(
            "Default",
            "[G]Hello",
            "G",
        )))
        .mount(&server)
        .await;

    bin()
        .env("ELVANTO_API_KEY", "abcdefghij")
        .env("ELVANTO_BASE_URL", server.uri())
        .args(["songs", "chart", "s1", "--transpose", "Q"])
        .assert()
        .failure()
        .code(2)
        .stderr(contains("invalid key"));
}
```

- [ ] **Step 6: Run**

Run: `cargo test --test songs_chart`
Expected: 4 passed.

Run: `cargo test`
Expected: every test suite green.

- [ ] **Step 7: Commit**

```bash
git add src/ tests/songs_chart.rs
git commit -m "feat(songs): add songs chart with named and offset transposition"
```

---

## Task 15: End-to-end polish — README run instructions, version flag check, lint pass

**Files:**
- Modify: `/home/chris/dev/elvanto-cli/README.md` (add a small "Building" section above "References")
- Modify: any module flagged by clippy

- [ ] **Step 1: Add a "Building & running" section to README.md before "References"**

```markdown
## Building & running

The crate ships as a binary called `elvanto`.

```sh
cargo build --release
export ELVANTO_API_KEY="your-key"
./target/release/elvanto auth check
./target/release/elvanto songs list --album --ccli
```

For local development, point at a stub:

```sh
ELVANTO_BASE_URL=http://localhost:8080 cargo run -- songs list
```
```

- [ ] **Step 2: Run the full test suite**

Run: `cargo test`
Expected: all suites pass.

- [ ] **Step 3: Run clippy and fmt**

Run: `cargo clippy --all-targets -- -D warnings`
Expected: no warnings.

Run: `cargo fmt --all -- --check`
Expected: no diffs. If diffs, run `cargo fmt --all` and re-run.

- [ ] **Step 4: Smoke-test help locally**

Run: `cargo run --quiet -- --help`
Expected: subcommands `auth`, `songs` printed.

Run: `cargo run --quiet -- songs --help`
Expected: subcommands `categories`, `list`, `show`, `chart`, `lyrics`.

- [ ] **Step 5: Commit**

```bash
git add README.md src/
git commit -m "docs: add build instructions; clippy and fmt clean"
```

---

## Self-review

**Spec coverage check vs CONTEXT.md / README / docs/songs.md:**

| Spec requirement | Task |
|---|---|
| `elvanto auth check` | Task 5 |
| `elvanto songs categories [--json]` | Task 7 |
| `elvanto songs list [--json] [--album] [--ccli]` | Task 8 |
| `elvanto songs show <id> [--json] [--full] [--files]` | Task 10 |
| `elvanto songs chart <id> [--transpose] [--arrangement]` | Tasks 13–14 |
| `elvanto songs lyrics <id> [--arrangement]` | Task 12 |
| Global `--verbose` | Task 4 (defined) + Task 5 (emits redacted key) |
| `ELVANTO_API_KEY` env var, no flag | Task 5 dispatcher |
| HTTP Basic with API key + dummy password | Task 3 |
| Exit codes 0 / 1 / 2 | Task 2 |
| Elvanto error code + message in stderr | Task 2 + Task 5 |
| JSON normalization (flatten wrappers, `ccli_number`, true/false, "active") | Tasks 6, 8, 9, 10 |
| Curated vs `--full` vs `--json` tiers for `songs show` | Task 10 |
| Default arrangement = "Default" or first; other names hinted | Task 11 used by Tasks 12 + 14 |
| Transpose accepts named keys or relative offsets | Task 13 + Task 14 |
| `songs list` auto-paginates and text mode filters active | Task 8 |
| `--files` only meaningful with `--json` | Task 10 |
| Redacted key in `auth check` | Task 5 (uses Task 3 helper) |

No gaps. No placeholders. Type names consistent (`SongSummary` vs `SongDetail` chosen deliberately; `Arrangement.chord_chart` is the field, populated from raw `chord_pro` or `chord_chart` via alias). The `Selection<'a>` helper type from Task 11 is used unchanged in Tasks 12 and 14.

**Risks / known gaps to verify during execution:**

- Elvanto field name for chord chart varies (`chord_pro` vs `chord_chart`); the alias handles both, but actual response shapes should be confirmed during Task 10 against a real API call before merging.
- `status` field encoding (numeric string `"1"`/`"0"` vs string `"active"`) — the `truthy` helper covers both; verify against a live call when first credentials are wired in.
- `ELVANTO_BASE_URL` is intentionally undocumented (test/override only); not surfaced in `--help` to keep the V1 CLI surface stable.
