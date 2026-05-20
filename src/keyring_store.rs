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
use std::sync::Once;

const SERVICE: &str = "elvanto-cli";
const ACCOUNT: &str = "api-key";

/// When the env var `ELVANTO_KEYRING_MOCK=1` is set, swap in keyring's
/// in-memory mock backend so tests don't touch the user's real OS keyring.
/// Production users never set this — they get the native backend.
static MOCK_INIT: Once = Once::new();

fn install_mock_if_requested() {
    MOCK_INIT.call_once(|| {
        if std::env::var("ELVANTO_KEYRING_MOCK").as_deref() == Ok("1") {
            keyring::set_default_credential_builder(keyring::mock::default_credential_builder());
        }
    });
}

fn entry() -> Result<Entry, CliError> {
    install_mock_if_requested();
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

#[cfg(test)]
mod tests {
    use super::*;

    /// `keyring`'s mock backend has `EntryOnly` persistence: every call to
    /// `Entry::new` produces a fresh, empty credential, so operations on
    /// different entries don't share state. That's exactly what we want for
    /// integration tests — each spawned bin starts with an empty keyring —
    /// but it does mean a single-process round-trip (`set` then `get`) won't
    /// see the stored value. The integration tests in `tests/auth_status.rs`
    /// drive the end-to-end behaviour; this unit test just confirms the mock
    /// installs cleanly and reports "no entry" on a fresh process.
    #[test]
    fn mock_backend_reports_empty_when_installed() {
        std::env::set_var("ELVANTO_KEYRING_MOCK", "1");
        assert_eq!(get().unwrap(), None);
        assert!(!delete().unwrap());
    }
}
