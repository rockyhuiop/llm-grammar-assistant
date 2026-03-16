//! Configuration types for the grammar checker CLI.
//!
//! Config is stored at platform-specific paths via the `directories` crate:
//! - macOS:   ~/Library/Application Support/grammar-check/config.toml
//! - Linux:   ~/.config/grammar-check/config.toml
//! - Windows: %APPDATA%\grammar-check\config.toml

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Top-level configuration persisted across sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    /// Whether to use local Ollama or a cloud API.
    pub mode: OperatingMode,
    /// Local mode settings (required when mode == Local).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local: Option<LocalConfig>,
    /// Cloud mode settings (required when mode == Cloud).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloud: Option<CloudConfig>,
}

/// The operating mode for grammar checking.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OperatingMode {
    Local,
    Cloud,
}

impl std::fmt::Display for OperatingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperatingMode::Local => write!(f, "local"),
            OperatingMode::Cloud => write!(f, "cloud"),
        }
    }
}

/// Configuration for local Ollama mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalConfig {
    #[serde(default = "default_ollama_host")]
    pub ollama_host: String,
    #[serde(default = "default_local_model")]
    pub model_name: String,
}

impl Default for LocalConfig {
    fn default() -> Self {
        Self {
            ollama_host: default_ollama_host(),
            model_name: default_local_model(),
        }
    }
}

/// Configuration for cloud API mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudConfig {
    pub provider: CloudProvider,
    /// Provider-specific model name. Uses provider default if None.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_name: Option<String>,
}

/// Supported cloud LLM providers.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CloudProvider {
    Gemini,
    #[serde(rename = "openai")]
    OpenAI,
}

impl CloudProvider {
    pub fn as_str(self) -> &'static str {
        match self {
            CloudProvider::Gemini => "gemini",
            CloudProvider::OpenAI => "openai",
        }
    }

    pub fn default_model(self) -> &'static str {
        match self {
            CloudProvider::Gemini => "gemini-2.0-flash",
            CloudProvider::OpenAI => "gpt-4o-mini",
        }
    }

    /// Keyring service name for credential storage.
    pub fn keyring_service(self) -> &'static str {
        match self {
            CloudProvider::Gemini => "grammar-check-gemini",
            CloudProvider::OpenAI => "grammar-check-openai",
        }
    }
}

impl std::fmt::Display for CloudProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

fn default_ollama_host() -> String {
    "http://localhost:11434".to_string()
}

fn default_local_model() -> String {
    "llama3".to_string()
}

impl Configuration {
    /// Returns the platform-specific config file path.
    pub fn config_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("", "", "grammar-check")
            .context("Could not determine platform config directory")?;
        Ok(proj_dirs.config_dir().join("config.toml"))
    }

    /// Loads configuration from the TOML file. Returns None if no config exists yet.
    pub fn load() -> Result<Option<Self>> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config from {}", path.display()))?;
        let config: Self = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config from {}", path.display()))?;
        Ok(Some(config))
    }

    /// Persists the configuration to the TOML file, creating directories if needed.
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config dir {}", parent.display()))?;
        }
        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;
        std::fs::write(&path, content)
            .with_context(|| format!("Failed to write config to {}", path.display()))?;
        Ok(())
    }

    /// Validates that required fields are present for the current mode.
    pub fn validate(&self) -> Result<()> {
        match self.mode {
            OperatingMode::Local => {
                if self.local.is_none() {
                    anyhow::bail!("Local mode requires [local] section in config");
                }
            }
            OperatingMode::Cloud => {
                if self.cloud.is_none() {
                    anyhow::bail!("Cloud mode requires [cloud] section in config");
                }
            }
        }
        Ok(())
    }
}
