use crate::api::Client;
use crate::cli::PeopleDepartmentsArgs;
use crate::domain::person::collect_departments;
use crate::error::CliError;
use crate::output;

pub async fn run(client: &Client, args: PeopleDepartmentsArgs) -> Result<(), CliError> {
    let raws = client.list_all_people(&["departments"]).await?;
    let rows = collect_departments(&raws);

    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    let res = if args.json {
        output::json::write_pretty(&mut lock, &rows)
    } else {
        output::text::write_departments(&mut lock, &rows, args.id_mode)
    };
    res.map_err(|e| CliError::Io(format!("write error: {e}")))
}
