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
mod transpose;

use clap::Parser;
use cli::{Cli, Command};
use error::CliError;
use std::process::ExitCode;

fn main() -> ExitCode {
    dotenvy::dotenv().ok();

    let args = auto_flags::apply(std::env::args().collect(), |k| std::env::var(k).ok());
    let cli = Cli::parse_from(args);
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");

    let result = rt.block_on(run(cli));
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(err.exit_code())
        }
    }
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
    // auth login/clear/status manage their own credential resolution, so
    // handle them before the global resolve step.
    if let Command::Auth { command } = &cli.command {
        match command {
            cli::AuthCommand::Login(args) => return commands::auth_login::run(args.clone()),
            cli::AuthCommand::Clear => return commands::auth_clear::run(),
            cli::AuthCommand::Status => return commands::auth_status::run().await,
            cli::AuthCommand::Check => {}
        }
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
        Command::Auth { command } => match command {
            cli::AuthCommand::Check => commands::auth_check::run(&client).await,
            // Handled above.
            cli::AuthCommand::Login(_) | cli::AuthCommand::Clear | cli::AuthCommand::Status => {
                unreachable!()
            }
        },
        Command::People { command } => match command {
            cli::PeopleCommand::List(args) => commands::people_list::run(&client, args).await,
            cli::PeopleCommand::Departments(args) => {
                commands::people_departments::run(&client, args).await
            }
        },
        Command::Services { command } => match command {
            cli::ServicesCommand::List(args) => commands::services_list::run(&client, args).await,
            cli::ServicesCommand::People(args) => {
                commands::services_people::run(&client, args).await
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
        },
    }
}
