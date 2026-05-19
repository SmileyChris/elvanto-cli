use crate::api::raw::RawService;
use crate::api::Client;
use crate::cli::ServicesPeopleArgs;
use crate::date_window::default_window;
use crate::domain::category::short_id;
use crate::domain::service::{volunteer_rows, VolunteerRow};
use crate::error::CliError;
use crate::output;
use chrono::Local;

pub async fn run(client: &Client, args: ServicesPeopleArgs) -> Result<(), CliError> {
    let raw = fetch_with_short_id_fallback(client, &args.id).await?;
    let mut rows = volunteer_rows(&raw);
    if !args.department.is_empty() {
        rows.retain(|r| r.matches_department(&args.department));
    }
    if args.hide_unfilled {
        rows.retain(VolunteerRow::is_filled);
    }
    if args.email {
        let emails = client.list_people_emails().await?;
        for row in rows.iter_mut() {
            if let Some(id) = &row.person_id {
                row.email = emails.get(id).cloned();
            }
        }
    }

    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    let res = if args.json {
        output::json::write_pretty(&mut lock, &rows)
    } else {
        output::text::write_service_people(&mut lock, &rows, args.email, args.id_mode)
    };
    res.map_err(|e| CliError::Io(format!("write error: {e}")))
}

/// Try the API directly; if Elvanto rejects the id, look it up against the
/// last-6-month services list and match by short id (first UUID block).
async fn fetch_with_short_id_fallback(
    client: &Client,
    input: &str,
) -> Result<RawService, CliError> {
    match client.get_service_info(input, &["volunteers"]).await {
        Ok(svc) => Ok(svc),
        Err(e @ CliError::NotFound(_)) | Err(e @ CliError::Api { .. }) => {
            resolve_short_id(client, input, e).await
        }
        Err(other) => Err(other),
    }
}

async fn resolve_short_id(
    client: &Client,
    input: &str,
    original: CliError,
) -> Result<RawService, CliError> {
    let today = Local::now().date_naive();
    let from = default_window(today).0;
    let from_s = from.format("%Y-%m-%d").to_string();
    let to_s = today.format("%Y-%m-%d").to_string();
    let services = client.list_services(&from_s, &to_s).await?;
    let matches: Vec<&RawService> = services
        .iter()
        .filter(|s| short_id(&s.id) == input || s.id == input)
        .collect();
    match matches.len() {
        0 => Err(original),
        1 => {
            client
                .get_service_info(&matches[0].id, &["volunteers"])
                .await
        }
        n => Err(CliError::Usage(format!(
            "ambiguous id {input:?} matches {n} services in the last 6 months; use full id"
        ))),
    }
}
