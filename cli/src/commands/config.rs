//! The `config` subcommand: manage grammar-check configuration.

use anyhow::{Context, Result};
use colored::Colorize;

use crate::models::config::{
    CloudConfig, CloudProvider, Configuration, LocalConfig, OperatingMode,
};
use crate::services::credentials::{delete_api_key, store_api_key};

/// Subcommand actions for `grammar-check config`.
#[derive(Debug, clap::Subcommand)]
pub enum ConfigAction {
    /// Set the operating mode (local or cloud).
    #[command(name = "--mode", alias = "mode")]
    SetMode {
        #[arg(value_enum)]
        mode: ModeArg,
    },
    /// Set the cloud provider (gemini or openai).
    #[command(name = "--provider", alias = "provider")]
    SetProvider {
        #[arg(value_enum)]
        provider: ProviderArg,
    },
    /// Store an API key in the OS keychain.
    #[command(name = "--set-api-key", alias = "set-api-key")]
    SetApiKey {
        /// The API key value (or omit to read from stdin).
        #[arg(long)]
        key: Option<String>,
    },
    /// Remove the stored API key from the OS keychain.
    #[command(name = "--delete-api-key", alias = "delete-api-key")]
    DeleteApiKey,
    /// Display current configuration.
    #[command(name = "--show", alias = "show")]
    Show,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum ModeArg {
    Local,
    Cloud,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum ProviderArg {
    Gemini,
    OpenAI,
}

/// Runs the config subcommand.
pub fn run(action: &ConfigAction) -> Result<()> {
    match action {
        ConfigAction::SetMode { mode } => set_mode(*mode),
        ConfigAction::SetProvider { provider } => set_provider(*provider),
        ConfigAction::SetApiKey { key } => set_api_key(key.as_deref()),
        ConfigAction::DeleteApiKey => delete_api_key_cmd(),
        ConfigAction::Show => show_config(),
    }
}

fn set_mode(mode: ModeArg) -> Result<()> {
    let mut config = load_or_default()?;
    config.mode = match mode {
        ModeArg::Local => {
            if config.local.is_none() {
                config.local = Some(LocalConfig::default());
            }
            OperatingMode::Local
        }
        ModeArg::Cloud => {
            if config.cloud.is_none() {
                config.cloud = Some(CloudConfig {
                    provider: CloudProvider::Gemini,
                    model_name: None,
                });
            }
            OperatingMode::Cloud
        }
    };
    config.save()?;
    println!(
        "{} Mode set to {}",
        "✓".green(),
        config.mode.to_string().bold()
    );
    Ok(())
}

fn set_provider(provider: ProviderArg) -> Result<()> {
    let mut config = load_or_default()?;
    let cloud_provider = match provider {
        ProviderArg::Gemini => CloudProvider::Gemini,
        ProviderArg::OpenAI => CloudProvider::OpenAI,
    };
    config.cloud = Some(CloudConfig {
        provider: cloud_provider,
        model_name: None,
    });
    config.mode = OperatingMode::Cloud;
    config.save()?;
    println!(
        "{} Provider set to {}",
        "✓".green(),
        cloud_provider.to_string().bold()
    );
    Ok(())
}

fn set_api_key(key: Option<&str>) -> Result<()> {
    let config = load_or_default()?;
    let cloud = config
        .cloud
        .as_ref()
        .context("No cloud provider configured. Set one with: grammar-check config --provider <gemini|openai>")?;

    let api_key = match key {
        Some(k) => k.to_string(),
        None => {
            print!("Enter API key for {}: ", cloud.provider);
            rpassword_read()
        }
    };

    if api_key.is_empty() {
        anyhow::bail!("API key cannot be empty");
    }

    store_api_key(cloud.provider.keyring_service(), &api_key)?;
    println!(
        "{} API key stored securely for {}",
        "✓".green(),
        cloud.provider.to_string().bold()
    );
    Ok(())
}

fn rpassword_read() -> String {
    // Simple stdin read for API key (not a TTY-password prompt, sufficient for CLI tool)
    use std::io::BufRead;
    let stdin = std::io::stdin();
    stdin.lock().lines().next()
        .and_then(|l| l.ok())
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn delete_api_key_cmd() -> Result<()> {
    let config = load_or_default()?;
    let cloud = config
        .cloud
        .as_ref()
        .context("No cloud provider configured")?;
    delete_api_key(cloud.provider.keyring_service())?;
    println!("{} API key removed for {}", "✓".green(), cloud.provider.to_string().bold());
    Ok(())
}

fn show_config() -> Result<()> {
    let path = Configuration::config_path()?;
    match Configuration::load()? {
        None => {
            println!("{}", "No configuration found.".yellow());
            println!("Config path: {}", path.display());
            println!("Run `grammar-check config --mode local` to create a configuration.");
        }
        Some(config) => {
            println!("{}", "Current configuration:".bold());
            println!("  Config file: {}", path.display());
            println!("  Mode: {}", config.mode.to_string().bold());

            if let Some(local) = &config.local {
                println!("\n  [local]");
                println!("    ollama_host: {}", local.ollama_host);
                println!("    model: {}", local.model_name);
            }
            if let Some(cloud) = &config.cloud {
                println!("\n  [cloud]");
                println!("    provider: {}", cloud.provider.to_string().bold());
                println!(
                    "    model: {}",
                    cloud
                        .model_name
                        .as_deref()
                        .unwrap_or(cloud.provider.default_model())
                );
                // Check if API key exists
                let has_key = crate::services::credentials::get_api_key(
                    cloud.provider.keyring_service(),
                )
                .map(|k| k.is_some())
                .unwrap_or(false);
                println!(
                    "    api_key: {}",
                    if has_key {
                        "stored in keychain".green().to_string()
                    } else {
                        "not set".red().to_string()
                    }
                );
            }
        }
    }
    Ok(())
}

fn load_or_default() -> Result<Configuration> {
    Ok(Configuration::load()?.unwrap_or_else(|| Configuration {
        mode: OperatingMode::Local,
        local: Some(LocalConfig::default()),
        cloud: None,
    }))
}
