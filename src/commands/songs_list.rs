use crate::api::Client;
use crate::cli::SongsListArgs;
use crate::domain::category;
use crate::domain::song::SongSummary;
use crate::error::CliError;
use crate::output;
use std::collections::HashSet;

pub async fn run(client: &Client, args: SongsListArgs) -> Result<(), CliError> {
    let SongsListArgs {
        json,
        album,
        ccli,
        category_ids,
        full_id,
    } = args;

    let mut raws = client.list_all_songs().await?;
    if !category_ids.is_empty() {
        let wanted: HashSet<&str> = category_ids.iter().map(String::as_str).collect();
        raws.retain(|song| {
            song.categories.category.iter().any(|category| {
                wanted
                    .iter()
                    .any(|id| category::id_matches(&category.id, id))
            })
        });
    }

    let all: Vec<SongSummary> = raws.into_iter().map(Into::into).collect();

    let stdout = std::io::stdout();
    let mut lock = stdout.lock();

    let res = if json {
        output::json::write_pretty(&mut lock, &all)
    } else {
        let active: Vec<SongSummary> = all.into_iter().filter(|s| s.status == "active").collect();
        output::text::write_songs(&mut lock, &active, album, ccli, full_id)
    };
    res.map_err(|e| CliError::Io(format!("write error: {e}")))
}
