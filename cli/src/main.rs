//! Grammar Check CLI Entry Point

#![deny(clippy::unwrap_used)]

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

use grammar_check::commands::check::OutputFormat;
use grammar_check::commands::config::ConfigAction;
use grammar_check::models::config::{
    CloudConfig, CloudProvider, Configuration, LocalConfig, OperatingMode,
};

#[derive(Parser, Debug)]
#[command(
    name = "grammar-check",
    about = "Grammar and style checker powered by local Ollama or cloud LLMs",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Check text for grammar and style issues.
    ///
    /// Reads from stdin or a file. Outputs issues with context and suggestions.
    ///
    /// Examples:
    ///   echo "He dont like it" | grammar-check check
    ///   grammar-check check document.txt
    ///   grammar-check check --format json document.txt
    Check {
        /// File to check. If omitted, reads from stdin.
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
        /// Output format.
        #[arg(long, value_enum, default_value = "human")]
        format: OutputFormat,
        /// Override the model for this invocation.
        #[arg(long)]
        model: Option<String>,
        /// Override the base URL for this invocation (cloud mode only).
        #[arg(long)]
        base_url: Option<String>,
    },
    /// Apply all grammar corrections to text.
    ///
    /// By default, outputs the corrected text to stdout.
    ///
    /// Examples:
    ///   grammar-check fix document.txt
    ///   grammar-check fix document.txt --in-place
    ///   grammar-check fix document.txt --output corrected.txt
    Fix {
        /// File to fix. If omitted, reads from stdin.
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
        /// Write corrected text to this file instead of stdout.
        #[arg(long, value_name = "FILE")]
        output: Option<PathBuf>,
        /// Modify the input file directly (requires FILE argument).
        #[arg(long)]
        in_place: bool,
        /// Override the model for this invocation.
        #[arg(long)]
        model: Option<String>,
        /// Override the base URL for this invocation (cloud mode only).
        #[arg(long)]
        base_url: Option<String>,
    },
    /// Manage configuration (mode, provider, API keys).
    ///
    /// Examples:
    ///   grammar-check config --mode local
    ///   grammar-check config --mode cloud
    ///   grammar-check config --provider gemini
    ///   grammar-check config --set-api-key
    ///   grammar-check config --show
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Config { action } => {
            grammar_check::commands::config::run(action)?;
        }
        Commands::Check { file, format, model, base_url } => {
            let mut config = load_config_or_prompt().await?;
            apply_cli_overrides(&mut config, model.as_deref(), base_url.as_deref());
            grammar_check::commands::check::run(file.as_ref(), *format, &config).await?;
        }
        Commands::Fix {
            file,
            output,
            in_place,
            model,
            base_url,
        } => {
            let mut config = load_config_or_prompt().await?;
            apply_cli_overrides(&mut config, model.as_deref(), base_url.as_deref());
            grammar_check::commands::fix::run(
                file.as_ref(),
                output.as_ref(),
                *in_place,
                &config,
            )
            .await?;
        }
    }

    Ok(())
}

/// Loads the configuration, prompting the user to set one up if none exists.
async fn load_config_or_prompt() -> Result<Configuration> {
    match Configuration::load()? {
        Some(config) => {
            config.validate()?;
            Ok(config)
        }
        None => {
            // First-run: block until user selects a mode
            println!("{}", "Welcome to Grammar Check!".bold().green());
            println!("No configuration found. Let's set up your preferred mode.\n");
            println!("  {} {} — Use a local Ollama instance (privacy-first, no internet needed)", "1.".bold(), "Local".bold());
            println!("  {} {} — Use Gemini or OpenAI cloud APIs (requires API key)", "2.".bold(), "Cloud".bold());
            println!();

            let mode = loop {
                print!("Select mode [1/2]: ");
                use std::io::{BufRead, Write};
                std::io::stdout().flush().ok();
                let stdin = std::io::stdin();
                let line = stdin
                    .lock()
                    .lines()
                    .next()
                    .and_then(|l| l.ok())
                    .unwrap_or_default();
                match line.trim() {
                    "1" | "local" => break OperatingMode::Local,
                    "2" | "cloud" => break OperatingMode::Cloud,
                    _ => println!("Please enter 1 or 2."),
                }
            };

            let config = match mode {
                OperatingMode::Local => Configuration {
                    mode: OperatingMode::Local,
                    local: Some(LocalConfig::default()),
                    cloud: None,
                },
                OperatingMode::Cloud => {
                    println!("\nCloud provider:");
                    println!("  {} {} — Gemini 2.0 Flash (recommended)", "1.".bold(), "Gemini".bold());
                    println!("  {} {} — GPT-4o Mini", "2.".bold(), "OpenAI".bold());

                    let provider = loop {
                        print!("Select provider [1/2]: ");
                        use std::io::{BufRead, Write};
                        std::io::stdout().flush().ok();
                        let stdin = std::io::stdin();
                        let line = stdin
                            .lock()
                            .lines()
                            .next()
                            .and_then(|l| l.ok())
                            .unwrap_or_default();
                        match line.trim() {
                            "1" | "gemini" => break CloudProvider::Gemini,
                            "2" | "openai" => break CloudProvider::OpenAI,
                            _ => println!("Please enter 1 or 2."),
                        }
                    };

                    Configuration {
                        mode: OperatingMode::Cloud,
                        local: None,
                        cloud: Some(CloudConfig {
                            provider,
                            model_name: None,
                            base_url: None,
                        }),
                    }
                }
            };

            config.save()?;
            println!(
                "\n{} Configuration saved. You can change it anytime with `grammar-check config`.\n",
                "✓".green()
            );

            if config.mode == OperatingMode::Cloud {
                println!(
                    "{} Don't forget to set your API key: {}",
                    "!".yellow(),
                    "grammar-check config --set-api-key".bold()
                );
            }

            Ok(config)
        }
    }
}

/// Applies per-invocation CLI overrides to the loaded configuration.
fn apply_cli_overrides(config: &mut Configuration, model: Option<&str>, base_url: Option<&str>) {
    if model.is_none() && base_url.is_none() {
        return;
    }
    if let Some(cloud) = config.cloud.as_mut() {
        if let Some(m) = model {
            cloud.model_name = Some(m.to_string());
        }
        if let Some(u) = base_url {
            cloud.base_url = Some(u.trim_end_matches('/').to_string());
        }
    }
    if let Some(local) = config.local.as_mut() {
        if let Some(m) = model {
            local.model_name = m.to_string();
        }
    }
}
