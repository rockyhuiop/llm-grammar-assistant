//! API credential storage and retrieval using the OS keychain.
//!
//! Credentials are stored in the OS keychain (not in config file) for security.
//! Uses the `keyring` crate which supports macOS Keychain, Linux libsecret, Windows Credential Manager.

use anyhow::{Context, Result};
use keyring::Entry;

const KEYRING_USERNAME: &str = "api-key";

/// Stores an API key in the OS keychain for the given provider.
///
/// Service name format: "grammar-check-{provider}" (e.g., "grammar-check-openai")
pub fn store_api_key(service: &str, api_key: &str) -> Result<()> {
    let entry = Entry::new(service, KEYRING_USERNAME)
        .with_context(|| format!("Failed to create keyring entry for {service}"))?;
    entry
        .set_password(api_key)
        .with_context(|| format!("Failed to store API key in keychain for {service}"))?;
    Ok(())
}

/// Retrieves an API key from the OS keychain for the given provider.
///
/// Returns None if no key has been stored yet.
pub fn get_api_key(service: &str) -> Result<Option<String>> {
    let entry = Entry::new(service, KEYRING_USERNAME)
        .with_context(|| format!("Failed to create keyring entry for {service}"))?;

    match entry.get_password() {
        Ok(key) => Ok(Some(key)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(anyhow::anyhow!("Failed to retrieve API key: {e}")),
    }
}

/// Deletes an API key from the OS keychain.
pub fn delete_api_key(service: &str) -> Result<()> {
    let entry = Entry::new(service, KEYRING_USERNAME)
        .with_context(|| format!("Failed to create keyring entry for {service}"))?;
    entry
        .delete_credential()
        .with_context(|| format!("Failed to delete API key from keychain for {service}"))?;
    Ok(())
}
