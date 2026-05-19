use crate::api::Client;
use crate::cli::SongsShowArgs;
use crate::domain::song::SongDetail;
use crate::error::CliError;
use crate::output;

pub async fn run(client: &Client, args: SongsShowArgs) -> Result<(), CliError> {
    if args.files && !args.json {
        eprintln!("warning: --files is only meaningful with --json; ignoring");
    }
    let want_files = args.files && args.json;
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
    res.map_err(|e| CliError::Io(format!("write error: {e}")))
}
