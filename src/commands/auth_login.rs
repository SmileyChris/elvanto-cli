use crate::cli::AuthLoginArgs;
use crate::error::CliError;
use crate::keyring_store;
use std::io::{BufRead, Write};

pub fn run(args: AuthLoginArgs) -> Result<(), CliError> {
    let key = if args.stdin {
        read_stdin_line()?
    } else {
        rpassword::prompt_password("Elvanto API key: ")
            .map_err(|e| CliError::Io(format!("stdin: {e}")))?
    };
    let trimmed = key.trim();
    if trimmed.is_empty() {
        return Err(CliError::Usage("API key is empty".into()));
    }
    keyring_store::set(trimmed)?;
    let stderr = std::io::stderr();
    let _ = writeln!(stderr.lock(), "stored API key in keyring");
    Ok(())
}

fn read_stdin_line() -> Result<String, CliError> {
    let mut line = String::new();
    std::io::stdin()
        .lock()
        .read_line(&mut line)
        .map_err(|e| CliError::Io(format!("stdin: {e}")))?;
    Ok(line)
}
