//! JSON output formatter for machine-readable output.

use crate::models::edit::CheckResult;

/// Serializes the check result to JSON and prints to stdout.
pub fn print_result(result: &CheckResult) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(result)
        .map_err(|e| anyhow::anyhow!("Failed to serialize result as JSON: {e}"))?;
    println!("{json}");
    Ok(())
}
