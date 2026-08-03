mod api;
mod arrangement_select;
mod auto_flags;
mod cli;
mod commands;
mod date_window;
mod domain;
mod error;
mod keyring_store;
mod output;
mod resolve;
mod transpose;

use clap::Parser;
use cli::{Cli, Command};
use error::CliError;
use std::process::ExitCode;

fn main() -> ExitCode {
    dotenvy::dotenv().ok();

    let applied = auto_flags::apply(std::env::args().collect(), |k| std::env::var(k).ok());
    let cli = Cli::parse_from(applied.argv);
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");

    let result = rt.block_on(run(cli));
    let exit = match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(ref err) => {
            eprintln!("error: {err}");
            ExitCode::from(err.exit_code())
        }
    };
    if let Some(note) = applied.note {
        eprintln!("{}", note.render());
    }
    exit
}

fn resolve_api_key() -> Result<String, CliError> {
    if let Ok(k) = std::env::var("ELVANTO_API_KEY") {
        if !k.is_empty() {
            return Ok(k);
        }
    }
    match keyring_store::get()? {
        Some(k) if !k.is_empty() => Ok(k),
        _ => Err(CliError::Usage(
            "no API key found; set ELVANTO_API_KEY or run `elvanto auth login`".into(),
        )),
    }
}

async fn run(cli: Cli) -> Result<(), CliError> {
    // auth subcommands all manage their own credential resolution.
    if let Command::Auth { command } = cli.command {
        return match command {
            cli::AuthCommand::Login(args) => commands::auth_login::run(args),
            cli::AuthCommand::Clear => commands::auth_clear::run(),
            cli::AuthCommand::Status => commands::auth_status::run().await,
        };
    }

    let api_key = resolve_api_key()?;
    let client = match std::env::var("ELVANTO_BASE_URL") {
        Ok(url) => api::Client::with_base_url(api_key, url)?,
        Err(_) => api::Client::new(api_key)?,
    };

    if cli.verbose {
        eprintln!("verbose: api_key={}", client.redacted_key());
    }

    match cli.command {
        Command::Auth { .. } => unreachable!("handled above"),
        Command::People { command } => match command {
            cli::PeopleCommand::List(args) => commands::people_list::run(&client, args).await,
            cli::PeopleCommand::Org(args) => commands::people_org::run(&client, args).await,
        },
        Command::Services { command } => match command {
            cli::ServicesCommand::List(args) => commands::services_list::run(&client, args).await,
            cli::ServicesCommand::People(args) => {
                commands::services_people::run(&client, args).await
            }
            cli::ServicesCommand::SongUsage(args) => {
                commands::services_song_usage::run(&client, args).await
            }
        },
        Command::Songs { command } => match command {
            cli::SongsCommand::Categories(args) => {
                commands::songs_categories::run(&client, args).await
            }
            cli::SongsCommand::List(args) => commands::songs_list::run(&client, args).await,
            cli::SongsCommand::Show(args) => commands::songs_show::run(&client, args).await,
            cli::SongsCommand::Chart(args) => commands::songs_chart::run(&client, args).await,
            cli::SongsCommand::Lyrics(args) => commands::songs_lyrics::run(&client, args).await,
            cli::SongsCommand::Export => commands::songs_export::run(&client).await,
            cli::SongsCommand::ArrangementEdit(args) => commands::songs_arrangement_edit::run(&client, args).await,
        },
    }
}
