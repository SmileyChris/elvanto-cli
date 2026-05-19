mod api;
mod cli;
mod commands;
mod error;

use clap::Parser;
use cli::{Cli, Command};
use error::CliError;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();
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

async fn run(cli: Cli) -> Result<(), CliError> {
    let api_key = std::env::var("ELVANTO_API_KEY")
        .map_err(|_| CliError::Usage("ELVANTO_API_KEY is not set".into()))?;
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
        },
        Command::Songs { command: _ } => {
            Err(CliError::Usage("songs commands not implemented yet".into()))
        }
    }
}
