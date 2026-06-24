# Elvanto CLI: `services list` Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `elvanto services list` showing services in a given date range, defaulting to the last 6 months. Read-only, text + `--json` output, auto-paginates.

**Architecture:** Mirrors the existing `songs list` layering. New `Service` domain type, `services/getAll` endpoint wrapper, `services_list` command, `Services` clap subcommand. A pure `date_window` helper computes the default window from an injected "now" so it stays testable. Pagination follows the same `page` / `per_page` / `total` / `on_this_page` walk used by `list_all_songs`.

**Tech Stack:** Rust 2021, existing deps (`clap`, `reqwest`, `serde`, `tokio`, `wiremock`, `assert_cmd`). One new dep: `chrono` (clock feature only) for date arithmetic.

---

## File Structure

```
elvanto-cli/
├── Cargo.toml                          # add chrono dep
├── CONTEXT.md                          # add Service terminology
├── README.md                           # add services list to the example block (optional)
├── src/
│   ├── cli.rs                          # add Services subcommand + ServicesListArgs
│   ├── main.rs                         # add Services dispatch arm
│   ├── date_window.rs                  # new: pure default_window(now) helper
│   ├── api/
│   │   ├── raw.rs                      # add RawService, ServicesResponse, ServiceList, RawServiceType
│   │   └── endpoints.rs                # add list_services(date_from, date_to)
│   ├── domain/
│   │   ├── mod.rs                      # add `pub mod service;`
│   │   └── service.rs                  # new: Service normalized type + From<RawService>
│   ├── output/
│   │   └── text.rs                     # add write_services
│   └── commands/
│       ├── mod.rs                      # add `pub mod services_list;`
│       └── services_list.rs            # new: orchestrator
└── tests/
    └── services_list.rs                # new: integration tests via wiremock + assert_cmd
```

**Boundaries follow existing conventions:**
- `api/raw.rs` mirrors Elvanto's JSON shape; no domain dependency.
- `domain/service.rs` defines the normalized type and `From<RawService>` impl.
- `output/text.rs` is a pure formatter taking `&mut dyn Write`.
- `commands/services_list.rs` orchestrates: pulls env-injected "now", computes window, calls endpoint, converts, dispatches to writer.
- `date_window.rs` is a sibling of `transpose.rs` / `arrangement_select.rs`: pure logic, unit-tested.

---

## API assumptions (Elvanto `services/getAll`)

The plan assumes Elvanto's `services/getAll` accepts these top-level body params and returns this shape. The integration tests build mocks against these shapes; if the real API differs in production, only the mocks and `list_services` body construction need adjusting — the rest of the layering is endpoint-agnostic.

**Request body (POST `https://api.elvanto.com/v1/services/getAll.json`):**
```json
{ "page": 1, "page_size": 100, "date_from": "2025-11-19", "date_to": "2026-05-19" }
```

**Response (success):**
```json
{
  "status": "ok",
  "services": {
    "page": 1,
    "per_page": 100,
    "total": 12,
    "on_this_page": 12,
    "service": [
      {
        "id": "svc-1",
        "date": "2026-04-12 09:30:00",
        "name": "Sunday Morning",
        "description": "Easter service",
        "status": "Published",
        "service_type": { "id": "st-1", "name": "Sunday Service" },
        "location": { "id": "loc-1", "name": "Main Auditorium" }
      }
    ]
  }
}
```

`status` values observed: `Published`, `Draft`, `Cancelled`. We normalize to lowercase strings.

---

## Task 1: Add `chrono` dependency and a pure `date_window` helper

**Files:**
- Modify: `/home/chris/dev/elvanto-cli/Cargo.toml`
- Create: `/home/chris/dev/elvanto-cli/src/date_window.rs`
- Modify: `/home/chris/dev/elvanto-cli/src/main.rs` (`mod date_window;`)
- Test: inline in `src/date_window.rs`

- [ ] **Step 1: Add chrono to Cargo.toml**

Find the `[dependencies]` block and add (alphabetically near `clap`):

```toml
chrono = { version = "0.4", default-features = false, features = ["clock"] }
```

- [ ] **Step 2: Write src/date_window.rs**

```rust
use crate::error::CliError;
use chrono::{Datelike, NaiveDate};

/// Returns (date_from, date_to) where date_to = `now` and date_from = `now` minus 6 months.
/// Day-of-month is clamped to the last valid day of the resulting month
/// (so e.g. Aug 31 → Feb 28 in a non-leap year).
pub fn default_window(now: NaiveDate) -> (NaiveDate, NaiveDate) {
    let from = subtract_months(now, 6);
    (from, now)
}

fn subtract_months(date: NaiveDate, months: u32) -> NaiveDate {
    let mut year = date.year();
    let mut month = date.month() as i32 - months as i32;
    while month < 1 {
        month += 12;
        year -= 1;
    }
    let month = month as u32;
    let day = date.day().min(last_day_of_month(year, month));
    NaiveDate::from_ymd_opt(year, month, day).expect("valid clamped date")
}

fn last_day_of_month(year: i32, month: u32) -> u32 {
    // Find day 28..=31 that exists this month.
    for d in (28..=31).rev() {
        if NaiveDate::from_ymd_opt(year, month, d).is_some() {
            return d;
        }
    }
    28
}

pub fn parse_date(input: &str, flag_name: &str) -> Result<NaiveDate, CliError> {
    NaiveDate::parse_from_str(input, "%Y-%m-%d").map_err(|_| {
        CliError::Usage(format!(
            "invalid {flag_name} value {input:?}; expected YYYY-MM-DD"
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn six_months_back_middle_of_month() {
        let now = d(2026, 5, 19);
        let (from, to) = default_window(now);
        assert_eq!(from, d(2025, 11, 19));
        assert_eq!(to, d(2026, 5, 19));
    }

    #[test]
    fn six_months_back_wraps_year() {
        let now = d(2026, 2, 10);
        let (from, _to) = default_window(now);
        assert_eq!(from, d(2025, 8, 10));
    }

    #[test]
    fn six_months_back_clamps_to_last_day() {
        // Aug 31 - 6 months = Feb 28 (2026 is not a leap year)
        let now = d(2026, 8, 31);
        let (from, _to) = default_window(now);
        assert_eq!(from, d(2026, 2, 28));
    }

    #[test]
    fn six_months_back_handles_leap_february() {
        // Aug 31 2024 - 6 months = Feb 29 (2024 is a leap year)
        let now = d(2024, 8, 31);
        let (from, _to) = default_window(now);
        assert_eq!(from, d(2024, 2, 29));
    }

    #[test]
    fn parse_date_ok() {
        assert_eq!(parse_date("2026-01-15", "--from").unwrap(), d(2026, 1, 15));
    }

    #[test]
    fn parse_date_rejects_bad_format() {
        let err = parse_date("01/15/2026", "--from").unwrap_err();
        assert!(matches!(err, CliError::Usage(_)));
        assert!(err.to_string().contains("--from"));
        assert!(err.to_string().contains("YYYY-MM-DD"));
    }

    #[test]
    fn parse_date_rejects_invalid_calendar_date() {
        // Feb 30 doesn't exist
        let err = parse_date("2026-02-30", "--from").unwrap_err();
        assert!(matches!(err, CliError::Usage(_)));
    }
}
```

- [ ] **Step 3: Wire module in src/main.rs**

Add `mod date_window;` alongside the other module declarations (alphabetical-ish, e.g. after `mod commands;` and before `mod domain;`).

- [ ] **Step 4: Run tests**

Run: `cargo test date_window::`
Expected: 7 passed.

Run: `cargo build`
Expected: clean. `date_window::default_window` and `parse_date` will be dead until Task 4 wires them up — add `#[allow(dead_code)]` on each public function (not the test mod). After Task 4 they'll be live; remove the attributes then.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock src/date_window.rs src/main.rs
git commit -m "feat(date): add chrono dep and default 6-month date window helper"
```

---

## Task 2: Raw + domain `Service` types and `list_services` endpoint

**Files:**
- Modify: `/home/chris/dev/elvanto-cli/src/api/raw.rs` (append types)
- Modify: `/home/chris/dev/elvanto-cli/src/api/endpoints.rs` (add `list_services`)
- Create: `/home/chris/dev/elvanto-cli/src/domain/service.rs`
- Modify: `/home/chris/dev/elvanto-cli/src/domain/mod.rs` (add `pub mod service;`)
- Test: inline in `src/domain/service.rs`

- [ ] **Step 1: Append raw types in src/api/raw.rs**

Add to the bottom of the file (after the existing Song / Arrangement / Category / Location types):

```rust
#[derive(Debug, Deserialize)]
pub struct ServicesResponse {
    #[serde(default)]
    pub services: ServiceList,
}

#[derive(Debug, Deserialize, Default)]
pub struct ServiceList {
    #[allow(dead_code)]
    #[serde(default)]
    pub page: u32,
    #[serde(default)]
    pub per_page: u32,
    #[serde(default)]
    pub total: u32,
    #[allow(dead_code)]
    #[serde(default)]
    pub on_this_page: u32,
    #[serde(default)]
    pub service: Vec<RawService>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawService {
    pub id: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub service_type: RawServiceType,
    #[serde(default)]
    pub location: RawServiceLocation,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawServiceType {
    #[allow(dead_code)]
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct RawServiceLocation {
    #[allow(dead_code)]
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
}
```

(`RawServiceLocation` is intentionally separate from the song-context `RawLocation` to avoid coupling unrelated endpoint schemas, even though the shape is identical. They live in different parts of Elvanto's API and may diverge.)

- [ ] **Step 2: Add `list_services` to src/api/endpoints.rs**

Inside the existing `impl Client { ... }` block (alongside `list_all_songs`, etc.), append:

```rust
pub async fn list_services(
    &self,
    date_from: &str,
    date_to: &str,
) -> Result<Vec<crate::api::raw::RawService>, CliError> {
    const SERVICES_PAGE_SIZE: u32 = 100;
    let mut out = Vec::new();
    let mut page: u32 = 1;
    loop {
        let resp: crate::api::raw::ServicesResponse = self
            .post(
                "services/getAll",
                &serde_json::json!({
                    "page": page,
                    "page_size": SERVICES_PAGE_SIZE,
                    "date_from": date_from,
                    "date_to": date_to,
                }),
            )
            .await?;
        let got = resp.services.service.len() as u32;
        out.extend(resp.services.service);
        let per_page = if resp.services.per_page == 0 {
            SERVICES_PAGE_SIZE
        } else {
            resp.services.per_page
        };
        if got < per_page || (resp.services.total > 0 && out.len() as u32 >= resp.services.total) {
            break;
        }
        page += 1;
        if page > 1000 {
            break;
        }
    }
    Ok(out)
}
```

(Same termination logic as `list_all_songs`; the `SERVICES_PAGE_SIZE` constant is scoped inside the method since it isn't reused elsewhere.)

- [ ] **Step 3: Write src/domain/service.rs**

```rust
use crate::api::raw::RawService;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Service {
    pub id: String,
    /// Original Elvanto timestamp, "YYYY-MM-DD HH:MM:SS".
    pub date: String,
    pub name: String,
    pub status: String,
    pub service_type: String,
    pub location: Option<String>,
    pub description: Option<String>,
}

impl Service {
    /// First 10 chars of the date string, i.e. "YYYY-MM-DD".
    pub fn date_short(&self) -> &str {
        if self.date.len() >= 10 {
            &self.date[..10]
        } else {
            &self.date
        }
    }
}

fn normalize_status(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        "unknown".to_string()
    } else {
        trimmed.to_ascii_lowercase()
    }
}

fn none_if_empty(s: String) -> Option<String> {
    if s.is_empty() { None } else { Some(s) }
}

impl From<RawService> for Service {
    fn from(raw: RawService) -> Self {
        Self {
            id: raw.id,
            date: raw.date,
            name: raw.name,
            status: normalize_status(&raw.status),
            service_type: raw.service_type.name,
            location: none_if_empty(raw.location.name),
            description: none_if_empty(raw.description),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::raw::{RawServiceLocation, RawServiceType};

    fn raw() -> RawService {
        RawService {
            id: "svc-1".into(),
            date: "2026-04-12 09:30:00".into(),
            name: "Sunday Morning".into(),
            description: "Easter".into(),
            status: "Published".into(),
            service_type: RawServiceType { id: "st-1".into(), name: "Sunday Service".into() },
            location: RawServiceLocation { id: "loc-1".into(), name: "Main".into() },
        }
    }

    #[test]
    fn from_raw_normalizes_status_to_lowercase() {
        let s: Service = raw().into();
        assert_eq!(s.status, "published");
    }

    #[test]
    fn from_raw_flattens_service_type_to_name() {
        let s: Service = raw().into();
        assert_eq!(s.service_type, "Sunday Service");
    }

    #[test]
    fn from_raw_empty_status_becomes_unknown() {
        let mut r = raw();
        r.status = "".into();
        let s: Service = r.into();
        assert_eq!(s.status, "unknown");
    }

    #[test]
    fn from_raw_empty_location_becomes_none() {
        let mut r = raw();
        r.location.name = "".into();
        let s: Service = r.into();
        assert_eq!(s.location, None);
    }

    #[test]
    fn date_short_takes_first_ten_chars() {
        let s: Service = raw().into();
        assert_eq!(s.date_short(), "2026-04-12");
    }

    #[test]
    fn date_short_handles_short_string() {
        let mut r = raw();
        r.date = "2026".into();
        let s: Service = r.into();
        assert_eq!(s.date_short(), "2026");
    }

    #[test]
    fn json_field_order_and_names() {
        let s: Service = raw().into();
        let v = serde_json::to_value(&s).unwrap();
        assert_eq!(v["id"], "svc-1");
        assert_eq!(v["service_type"], "Sunday Service");
        assert_eq!(v["status"], "published");
        assert_eq!(v["location"], "Main");
    }
}
```

- [ ] **Step 4: Update src/domain/mod.rs**

```rust
pub mod arrangement;
pub mod category;
pub mod service;
pub mod song;

pub(crate) fn none_if_empty(s: String) -> Option<String> {
    if s.is_empty() { None } else { Some(s) }
}
```

(The crate-level `none_if_empty` already exists from the Songs work. The local one inside `service.rs` is kept as a free function rather than imported because it's used by the `From` impl in this file and keeps service.rs self-contained. If clippy or a reviewer flags the duplication, replace `service.rs`'s `none_if_empty` with `use crate::domain::none_if_empty;` — both call sites would then share. Either is acceptable; choose duplication for now to keep this task self-contained.)

- [ ] **Step 5: Run tests**

Run: `cargo test domain::service::`
Expected: 7 passed.

Run: `cargo test`
Expected: 59 passed (52 existing + 7 new).

Run: `cargo build`
Expected: clean. `list_services`, `Service`, and the new raw types will emit dead-code warnings — add `#[allow(dead_code)]` on `list_services` (Task 3 wires it), on `pub struct Service` if the compiler complains about the `Service` type being unused outside tests, and on `RawService` if needed. After Task 3, remove these attributes.

- [ ] **Step 6: Commit**

```bash
git add src/api/ src/domain/ 
git commit -m "feat(services): add Service domain type and list_services endpoint"
```

---

## Task 3: `services list` command + CLI subcommand + integration tests

**Files:**
- Modify: `/home/chris/dev/elvanto-cli/src/cli.rs` (add `Services` + `ServicesListArgs`)
- Modify: `/home/chris/dev/elvanto-cli/src/output/text.rs` (add `write_services`)
- Create: `/home/chris/dev/elvanto-cli/src/commands/services_list.rs`
- Modify: `/home/chris/dev/elvanto-cli/src/commands/mod.rs` (add `pub mod services_list;`)
- Modify: `/home/chris/dev/elvanto-cli/src/main.rs` (dispatch `Services`)
- Modify: `/home/chris/dev/elvanto-cli/src/date_window.rs` (remove `#[allow(dead_code)]`)
- Modify: `/home/chris/dev/elvanto-cli/src/api/endpoints.rs` (remove `#[allow(dead_code)]` on `list_services` if added)
- Modify: `/home/chris/dev/elvanto-cli/src/domain/service.rs` (remove `#[allow(dead_code)]` if added)
- Test: `tests/services_list.rs`

- [ ] **Step 1: Add Services to src/cli.rs**

Add a `Services` variant to the top-level `Command` enum (alongside `Auth` and `Songs`):

```rust
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Authentication utilities.
    Auth {
        #[command(subcommand)]
        command: AuthCommand,
    },
    /// Service (calendar event) commands.
    Services {
        #[command(subcommand)]
        command: ServicesCommand,
    },
    /// Worship song commands.
    Songs {
        #[command(subcommand)]
        command: SongsCommand,
    },
}
```

Then add (anywhere in cli.rs after the existing subcommand enums):

```rust
#[derive(Debug, Subcommand)]
pub enum ServicesCommand {
    /// List services in a date range (defaults to the last 6 months).
    List(ServicesListArgs),
}

#[derive(Debug, Args)]
pub struct ServicesListArgs {
    /// Inclusive start date (YYYY-MM-DD). Defaults to 6 months before --to.
    #[arg(long, value_name = "YYYY-MM-DD")]
    pub from: Option<String>,
    /// Inclusive end date (YYYY-MM-DD). Defaults to today (local time).
    #[arg(long, value_name = "YYYY-MM-DD")]
    pub to: Option<String>,
    /// Emit normalized JSON instead of text.
    #[arg(long)]
    pub json: bool,
}
```

- [ ] **Step 2: Add `write_services` to src/output/text.rs**

Append (alongside `write_categories`, `write_songs`, `write_song_curated`, `write_song_full`):

```rust
use crate::domain::service::Service;

pub fn write_services<W: Write>(w: &mut W, services: &[Service]) -> io::Result<()> {
    for s in services {
        let location = s.location.as_deref().unwrap_or("-");
        writeln!(
            w,
            "{} | {} | {} | {} | {} | {}",
            s.id,
            s.date_short(),
            s.name,
            s.service_type,
            location,
            s.status,
        )?;
    }
    Ok(())
}
```

(The `use crate::domain::service::Service;` line goes near the existing `use crate::domain::...` imports at the top of `text.rs`.)

- [ ] **Step 3: Write src/commands/services_list.rs**

```rust
use crate::api::Client;
use crate::cli::ServicesListArgs;
use crate::date_window::{default_window, parse_date};
use crate::domain::service::Service;
use crate::error::CliError;
use crate::output;
use chrono::Local;

pub async fn run(client: &Client, args: ServicesListArgs) -> Result<(), CliError> {
    let today = Local::now().date_naive();
    let to = match args.to.as_deref() {
        Some(s) => parse_date(s, "--to")?,
        None => today,
    };
    let from = match args.from.as_deref() {
        Some(s) => parse_date(s, "--from")?,
        None => default_window(to).0,
    };
    if from > to {
        return Err(CliError::Usage(format!(
            "--from ({from}) must be on or before --to ({to})"
        )));
    }

    let from_str = from.format("%Y-%m-%d").to_string();
    let to_str = to.format("%Y-%m-%d").to_string();

    let raws = client.list_services(&from_str, &to_str).await?;
    let services: Vec<Service> = raws.into_iter().map(Into::into).collect();

    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    let res = if args.json {
        output::json::write_pretty(&mut lock, &services)
    } else {
        output::text::write_services(&mut lock, &services)
    };
    res.map_err(|e| CliError::Io(format!("write error: {e}")))
}
```

- [ ] **Step 4: Update src/commands/mod.rs**

Final content:
```rust
pub mod auth_check;
pub mod services_list;
pub mod songs_categories;
pub mod songs_chart;
pub mod songs_list;
pub mod songs_lyrics;
pub mod songs_show;
```

- [ ] **Step 5: Dispatch in src/main.rs**

Inside the existing `match cli.command` block, add a new arm for `Command::Services`:

```rust
Command::Services { command } => match command {
    cli::ServicesCommand::List(args) => commands::services_list::run(&client, args).await,
},
```

- [ ] **Step 6: Remove `#[allow(dead_code)]` from items now consumed**

- `src/date_window.rs`: remove the attributes on `default_window` and `parse_date`.
- `src/api/endpoints.rs`: remove `#[allow(dead_code)]` on `list_services` if it was added in Task 2.
- `src/domain/service.rs`: remove `#[allow(dead_code)]` from `Service` if it was added.

Verify with `cargo build` — should be zero warnings.

- [ ] **Step 7: Write tests/services_list.rs**

```rust
mod common;
use common::{bin, mock_server};
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

fn svc(id: &str, date: &str, name: &str, status: &str, type_name: &str, location: &str) -> serde_json::Value {
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
    // No --from / --to: command should call services/getAll with some date range.
    // We don't pin the exact dates here (they depend on Local::now() at test runtime);
    // we only assert that *some* date_from + date_to keys are present.
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/services/getAll.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok_page(vec![
            svc("svc-1", "2026-04-12 09:30:00", "Sunday Morning", "Published", "Sunday Service", "Main"),
            svc("svc-2", "2026-04-19 09:30:00", "Sunday Morning", "Draft", "Sunday Service", "Main"),
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
            contains("svc-1 | 2026-04-12 | Sunday Morning | Sunday Service | Main | published"),
        );
}

#[tokio::test]
async fn from_and_to_flags_drive_request_body() {
    let server = mock_server().await;
    Mock::given(method("POST"))
        .and(path("/services/getAll.json"))
        .and(body_partial_json(serde_json::json!({
            "date_from": "2026-01-01",
            "date_to": "2026-03-31"
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
        .stdout(contains("svc-9 | 2026-02-14 | Valentine Vigil"));
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
    // No mock needed; we exit before any HTTP call.
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
        .args(["services", "list", "--from", "2026-05-01", "--to", "2026-04-01"])
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
```

The `contains` + `.and()` combinator requires `predicates::prelude::*`. Add it at the top:

```rust
use predicates::prelude::*;
```

- [ ] **Step 8: Run**

Run: `cargo test --test services_list`
Expected: 6 passed.

Run: `cargo test`
Expected: 65 passed (59 from end of Task 2 + 6 new).

Run: `cargo clippy --all-targets -- -D warnings`
Expected: no warnings.

Run: `cargo fmt --all`
Then: `cargo fmt --all -- --check`
Expected: clean.

Run: `cargo run --quiet -- services --help 2>&1 | head -15`
Expected: lists `list` subcommand with `--from`, `--to`, `--json` flags.

- [ ] **Step 9: Commit**

```bash
git add src/ tests/services_list.rs
git commit -m "feat(services): add services list command with 6-month default window"
```

---

## Task 4: Documentation

**Files:**
- Modify: `/home/chris/dev/elvanto-cli/CONTEXT.md`
- Modify: `/home/chris/dev/elvanto-cli/README.md`

- [ ] **Step 1: Update CONTEXT.md — add Service terminology**

In `CONTEXT.md`, find the `## Language` section. After the `**Category**:` entry (and before `**CCLI number**:`), insert:

```markdown
**Service**:
A scheduled service or event (e.g., "Sunday Morning") with a date, name, type, status, and location. Distinct from `Item`, which is one of the elements that make up a service.
_Avoid_: Event (Elvanto reserves "event" for non-service calendar entries), gathering
```

And right after the `**Item**:` entry, add a cross-reference line:

```markdown
_See also_: `Service` — a collection of Items.
```

Then in the `## V1 commands (read-only)` section, append:

```
elvanto services list               [--json] [--from YYYY-MM-DD] [--to YYYY-MM-DD]
```

And add a new sub-section near the bottom of the file, just before `## Relationships`:

```markdown
### `services list`

Lists services in a date range. Defaults to the last 6 months (today minus 6 months → today, local time). Text columns: `id | date | name | service_type | location | status`. `--json` returns the normalized array. Auto-paginates.

```sh
elvanto services list
elvanto services list --from 2026-01-01 --to 2026-03-31
elvanto services list --json
```
```

Finally update `## Relationships` to include:

```markdown
- A **Service** has zero or more **Items** (an Item is _not_ a Song)
- A **Service** belongs to one **ServiceType** (text + JSON output expose the type name, not the id)
```

- [ ] **Step 2: Update README.md**

In the `## Building & running` section, replace the second `./target/release/elvanto` line with:

```sh
./target/release/elvanto auth check
./target/release/elvanto songs list --album --ccli
./target/release/elvanto services list
```

In the `## Songs Workflow Priority` section (or wherever fits in the existing README), no change is required for V2; the README's "Proposed Command Shape" already lists songs as the focus. If you want to acknowledge services, add a single line under the V1 status paragraph that reads: "Services V2 adds `elvanto services list`."

- [ ] **Step 3: Verify lint passes after doc edits**

Run: `cargo test`
Expected: 65 passed.

(Docs alone shouldn't affect tests, but run anyway as a smoke check.)

- [ ] **Step 4: Commit**

```bash
git add CONTEXT.md README.md
git commit -m "docs(services): add Service terminology and services list usage"
```

---

## Self-review

**Spec coverage vs. user request ("services list, default last 6 months"):**

| Requirement | Task |
|---|---|
| `elvanto services list` subcommand exists | Task 3 |
| Default time window = last 6 months | Tasks 1 + 3 (`default_window`) |
| `--from` / `--to` override the window | Task 3 (`ServicesListArgs`) |
| Text mode columns | Task 3 (`write_services`) |
| `--json` normalized output | Tasks 2 + 3 (`Service` Serialize + `services_list::run`) |
| Pagination | Task 2 (`list_services` loop) |
| Status normalized to lowercase | Task 2 (`From<RawService>`) |
| Invalid date input → exit 2 | Task 3 (`parse_date` → `CliError::Usage`) |
| `--from > --to` → exit 2 | Task 3 (`run` guard) |
| Documentation updated | Task 4 |

**Type consistency check:** `Service` field names, signatures, and the `RawService` field set are stable across Tasks 2 and 3. `default_window(now: NaiveDate) -> (NaiveDate, NaiveDate)` and `parse_date(input, flag) -> Result<NaiveDate, CliError>` signatures used in Task 3 match Task 1. `list_services(date_from: &str, date_to: &str)` matches between Task 2 (define) and Task 3 (call).

**No placeholders.** Every code step contains real code. Every test step contains real assertions.

**Risks worth verifying during execution:**
- Real Elvanto `services/getAll` may use a slightly different body shape (e.g., `fields[date_from]` keyed under a `fields` object). Task 2's body construction is the only thing that needs adjusting in that case; the rest of the pipeline is endpoint-agnostic. If you want to harden this, add a one-off live probe against a real API key before merging Task 4.
- The default-window tests pin specific dates (e.g. 2026-05-19 → 2025-11-19). Those tests use injected `NaiveDate`, so they don't depend on the system clock — but the `text_output_default_window` integration test in Task 3 doesn't pin the request body's dates either; it only checks that the response renders. That's intentional to avoid clock-dependent flakiness.
