use crate::api::Client;
use crate::arrangement_select;
use crate::cli::SongsLyricsArgs;
use crate::domain::song::SongDetail;
use crate::error::CliError;

pub async fn run(client: &Client, args: SongsLyricsArgs) -> Result<(), CliError> {
    let raw = client.get_song_info(&args.id, false).await?;
    let detail: SongDetail = raw.into();

    let sel = arrangement_select::select(&detail.arrangements, args.arrangement.as_deref())?;

    let lyrics = sel.chosen.lyrics.as_deref().ok_or_else(|| {
        CliError::NotFound(format!("arrangement {:?} has no lyrics", sel.chosen.name))
    })?;
    println!("{lyrics}");

    if !sel.others.is_empty() {
        let names: Vec<&str> = sel.others.iter().map(|a| a.name.as_str()).collect();
        eprintln!("\n(other arrangements: {})", names.join(", "));
    }
    Ok(())
}
