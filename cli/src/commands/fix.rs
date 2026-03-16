//! The `fix` subcommand: automatically apply grammar corrections to text.

use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};
use colored::Colorize;

use crate::commands::check::{read_input, check_text};
use crate::models::config::Configuration;
use crate::services::diff::apply_edits;

/// Runs the grammar fix, applying all suggested corrections.
///
/// Output options (mutually exclusive):
/// - Default: write corrected text to stdout
/// - `--output <path>`: write to a specific file
/// - `--in-place`: modify the source file directly (requires file argument)
pub async fn run(
    file: Option<&PathBuf>,
    output: Option<&PathBuf>,
    in_place: bool,
    config: &Configuration,
) -> Result<()> {
    let raw_text = read_input(file)?;

    if raw_text.trim().is_empty() {
        println!("No text to fix.");
        return Ok(());
    }

    let result = check_text(&raw_text, config).await?;

    if result.edits.is_empty() {
        println!("{}", "No issues found — no changes made.".green());
        return Ok(());
    }

    let fixed_text = apply_edits(&result.original_text, &result.edits)
        .context("Failed to apply edits")?;

    let edit_count = result.edits.len();

    match (in_place, output) {
        (true, _) => {
            // --in-place: overwrite the source file
            let path = file.context("--in-place requires a file argument (not stdin)")?;
            std::fs::write(path, &fixed_text)
                .with_context(|| format!("Failed to write to {}", path.display()))?;
            eprintln!(
                "{} Applied {} correction(s) to {}",
                "✓".green(),
                edit_count,
                path.display()
            );
        }
        (false, Some(out_path)) => {
            // --output <path>: write to specified file
            std::fs::write(out_path, &fixed_text)
                .with_context(|| format!("Failed to write to {}", out_path.display()))?;
            eprintln!(
                "{} Applied {} correction(s), written to {}",
                "✓".green(),
                edit_count,
                out_path.display()
            );
        }
        (false, None) => {
            // Default: write corrected text to stdout
            std::io::stdout()
                .write_all(fixed_text.as_bytes())
                .context("Failed to write to stdout")?;
        }
    }

    Ok(())
}
