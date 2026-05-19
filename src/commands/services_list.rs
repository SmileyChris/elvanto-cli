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
        output::text::write_services(&mut lock, &services, args.full_id)
    };
    res.map_err(|e| CliError::Io(format!("write error: {e}")))
}
