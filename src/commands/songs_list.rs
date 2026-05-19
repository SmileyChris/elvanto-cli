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
    res.map_err(|e| CliError::Io(format!("write error: {e}")))
}
