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
    res.map_err(|e| CliError::Io(format!("write error: {e}")))
}
