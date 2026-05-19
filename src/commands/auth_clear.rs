use crate::error::CliError;
use crate::keyring_store;
use std::io::Write;

pub fn run() -> Result<(), CliError> {
    let removed = keyring_store::delete()?;
    let stderr = std::io::stderr();
    let mut lock = stderr.lock();
    if removed {
        let _ = writeln!(lock, "removed API key from keyring");
    } else {
        let _ = writeln!(lock, "no API key stored in keyring");
    }
    Ok(())
}
