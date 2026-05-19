use crate::api::Client;
use crate::cli::ServicesPeopleArgs;
use crate::domain::service::{volunteer_rows, VolunteerRow};
use crate::error::CliError;
use crate::output;

pub async fn run(client: &Client, args: ServicesPeopleArgs) -> Result<(), CliError> {
    let raw = client.get_service_info(&args.id, &["volunteers"]).await?;
    let mut rows = volunteer_rows(&raw);
    if args.filled {
        rows.retain(VolunteerRow::is_filled);
    }

    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    let res = if args.json {
        output::json::write_pretty(&mut lock, &rows)
    } else {
        output::text::write_service_people(&mut lock, &rows)
    };
    res.map_err(|e| CliError::Io(format!("write error: {e}")))
}
