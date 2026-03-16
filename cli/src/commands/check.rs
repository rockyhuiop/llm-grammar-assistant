//! The `check` subcommand: check text for grammar issues.

use std::io::Read;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::{Context, Result};

use crate::models::config::{CloudConfig, Configuration, LocalConfig, OperatingMode};
use crate::models::edit::{CheckResult, ProcessingMetadata};
use crate::providers::cloud::CloudLlmProvider;
use crate::providers::ollama::OllamaProvider;
use crate::providers::Provider;
use crate::services::chunker::{chunk_text, merge_chunk_edits, ChunkingOptions};
use crate::services::credentials::get_api_key;
use crate::services::position::normalize_line_endings;

/// Output format for the check command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum, Default)]
pub enum OutputFormat {
    /// Colored human-readable output (default)
    #[default]
    Human,
    /// Machine-readable JSON
    Json,
}

/// Runs the grammar check on text from stdin or a file and prints output.
pub async fn run(
    file: Option<&PathBuf>,
    format: OutputFormat,
    config: &Configuration,
) -> Result<()> {
    let raw_text = read_input(file)?;

    if raw_text.trim().is_empty() {
        println!("No text to check.");
        return Ok(());
    }

    let result = check_text(&raw_text, config).await?;

    if result.edits.is_empty() {
        match format {
            OutputFormat::Human => {
                use colored::Colorize;
                println!("{}", "No issues found.".green());
            }
            OutputFormat::Json => crate::output::json::print_result(&result)?,
        }
        return Ok(());
    }

    match format {
        OutputFormat::Human => crate::output::human::print_result(&result),
        OutputFormat::Json => crate::output::json::print_result(&result)?,
    }

    Ok(())
}

/// Performs the grammar check and returns the result (without printing).
/// Used by the `fix` command to get edits before applying them.
pub async fn get_check_result(file: Option<&PathBuf>, config: &Configuration) -> Result<CheckResult> {
    let raw_text = read_input(file)?;
    check_text(&raw_text, config).await
}

/// Core grammar check logic: normalizes text, selects provider, runs check.
pub async fn check_text(raw_text: &str, config: &Configuration) -> Result<CheckResult> {
    let text = normalize_line_endings(raw_text).into_owned();
    let start = Instant::now();

    match config.mode {
        OperatingMode::Local => {
            let default_local = LocalConfig::default();
            let local = config.local.as_ref().unwrap_or(&default_local);
            let provider = OllamaProvider::new(local.ollama_host.clone(), local.model_name.clone());
            provider.health_check().await?;
            run_check_with_provider(&provider, &text, config, start).await
        }
        OperatingMode::Cloud => {
            let cloud = config.cloud.as_ref().context(
                "Cloud mode requires cloud configuration. Run: grammar-check config --mode cloud",
            )?;
            let api_key = get_api_key_for_provider(cloud)?;
            let model = cloud
                .model_name
                .clone()
                .unwrap_or_else(|| cloud.provider.default_model().to_string());
            let provider = CloudLlmProvider::new(cloud.provider, model, api_key);
            run_check_with_provider(&provider, &text, config, start).await
        }
    }
}

/// Reads input text from a file path or stdin.
pub fn read_input(file: Option<&PathBuf>) -> Result<String> {
    match file {
        Some(path) => std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display())),
        None => {
            let mut text = String::new();
            std::io::stdin()
                .read_to_string(&mut text)
                .context("Failed to read from stdin")?;
            Ok(text)
        }
    }
}

async fn run_check_with_provider<P: Provider>(
    provider: &P,
    text: &str,
    config: &Configuration,
    start: Instant,
) -> Result<CheckResult> {
    let opts = ChunkingOptions::default();
    let chunks = chunk_text(text, &opts);
    let chunks_count = chunks.len();

    if chunks_count == 1 {
        let edits = provider.check(text).await.context("Grammar check failed")?;
        return Ok(CheckResult {
            original_text: text.to_string(),
            edits,
            metadata: ProcessingMetadata {
                mode: config.mode,
                provider: provider.provider_name().to_string(),
                model: provider.model_name().to_string(),
                processing_time_ms: start.elapsed().as_millis() as u64,
                chunks_processed: None,
            },
        });
    }

    // Multiple chunks: process sequentially with bounded memory
    let mut chunk_edits = Vec::with_capacity(chunks_count);
    for chunk in &chunks {
        let edits = provider
            .check(&chunk.text)
            .await
            .context("Grammar check failed on chunk")?;
        chunk_edits.push(edits);
    }

    let edits = merge_chunk_edits(&chunks, chunk_edits, text);

    Ok(CheckResult {
        original_text: text.to_string(),
        edits,
        metadata: ProcessingMetadata {
            mode: config.mode,
            provider: provider.provider_name().to_string(),
            model: provider.model_name().to_string(),
            processing_time_ms: start.elapsed().as_millis() as u64,
            chunks_processed: Some(chunks_count as u32),
        },
    })
}

fn get_api_key_for_provider(cloud: &CloudConfig) -> Result<String> {
    let service = cloud.provider.keyring_service();
    get_api_key(service)?.ok_or_else(|| {
        anyhow::anyhow!(
            "No API key found for {}. Store it with: grammar-check config --set-api-key",
            cloud.provider
        )
    })
}
