mod error;
#[allow(dead_code)]
mod api;

use error::CliError;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(err.exit_code())
        }
    }
}

fn run() -> Result<(), CliError> {
    Ok(())
}
