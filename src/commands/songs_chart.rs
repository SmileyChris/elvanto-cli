use crate::api::Client;
use crate::arrangement_select;
use crate::cli::SongsChartArgs;
use crate::domain::arrangement::Arrangement;
use crate::domain::song::SongDetail;
use crate::error::CliError;
use crate::transpose;
use std::io::Write;

pub async fn run(client: &Client, args: SongsChartArgs) -> Result<(), CliError> {
    let raw_song = client.get_song_info(&args.id, false).await?;
    let detail: SongDetail = raw_song.into();
    let sel = arrangement_select::select(&detail.arrangements, args.arrangement.as_deref())?;
    let chosen = sel.chosen;

    let chart = match args.transpose.as_deref() {
        None => chosen.chord_chart.clone().ok_or_else(|| {
            CliError::NotFound(format!("arrangement {:?} has no chord chart", chosen.name))
        })?,
        Some(input) => {
            let req = transpose::parse(input)?;
            let target = match req {
                transpose::Request::Named(key) => key,
                offset @ transpose::Request::Offset(_) => {
                    let starting = chosen
                        .keys
                        .first()
                        .map(|k| k.starting.as_str())
                        .ok_or_else(|| {
                            CliError::NotFound(format!("arrangement {:?} has no key", chosen.name))
                        })?;
                    transpose::resolve(&offset, starting)?
                }
            };
            let raw_arr = client
                .get_arrangement_info(&chosen.id, Some(&target))
                .await?;
            let arr: Arrangement = raw_arr.into();
            arr.chord_chart.ok_or_else(|| {
                CliError::NotFound(format!(
                    "Elvanto returned no transposed chord chart for {target}"
                ))
            })?
        }
    };

    {
        let stdout = std::io::stdout();
        let mut lock = stdout.lock();
        writeln!(lock, "{chart}").map_err(|e| CliError::Io(format!("write error: {e}")))?;
    }

    if !sel.others.is_empty() {
        let names: Vec<&str> = sel.others.iter().map(|a| a.name.as_str()).collect();
        eprintln!("\n(other arrangements: {})", names.join(", "));
    }
    Ok(())
}
