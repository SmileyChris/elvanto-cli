//! OS-keyring storage for the Elvanto API key.
//!
//! Backed by `keyring` v3 with native backends:
//!   * macOS Keychain
//!   * Linux: D-Bus Secret Service (gnome-keyring / KWallet), with kernel
//!     keyutils as a secondary backend
//!   * Windows Credential Manager
//!
//! The CLI stores at most one credential under `service="elvanto-cli"`,
//! `account="api-key"`.

use crate::error::CliError;
use keyring::Entry;

const SERVICE: &str = "elvanto-cli";
const ACCOUNT: &str = "api-key";

fn entry() -> Result<Entry, CliError> {
    Entry::new(SERVICE, ACCOUNT).map_err(|e| CliError::Io(format!("keyring: {e}")))
}

/// Returns `Ok(Some(key))` when a credential exists, `Ok(None)` when none is
/// stored, and `Err` for keyring backend failures (D-Bus down, etc.).
pub fn get() -> Result<Option<String>, CliError> {
    match entry()?.get_password() {
        Ok(s) => Ok(Some(s)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(CliError::Io(format!("keyring: {e}"))),
    }
}

pub fn set(key: &str) -> Result<(), CliError> {
    entry()?
        .set_password(key)
        .map_err(|e| CliError::Io(format!("keyring: {e}")))
}

/// Returns `Ok(true)` if a credential was deleted, `Ok(false)` if nothing was
/// stored, and `Err` for backend failures.
pub fn delete() -> Result<bool, CliError> {
    match entry()?.delete_credential() {
        Ok(()) => Ok(true),
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(e) => Err(CliError::Io(format!("keyring: {e}"))),
    }
}
