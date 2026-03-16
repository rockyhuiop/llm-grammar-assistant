//! Ollama provider for local grammar checking (no network calls to external services).

use anyhow::{Context, Result};
use serde_json::json;

use crate::models::edit::Edit;
use crate::providers::prompts::{format_user_message, GRAMMAR_CHECK_SYSTEM_PROMPT};
use crate::services::validator;

/// Provider that talks to a local Ollama instance.
pub struct OllamaProvider {
    host: String,
    model: String,
    client: reqwest::Client,
}

impl OllamaProvider {
    pub fn new(host: String, model: String) -> Self {
        Self {
            host,
            model,
            client: reqwest::Client::new(),
        }
    }

    /// Checks that Ollama is reachable before attempting grammar check.
    pub async fn health_check(&self) -> Result<()> {
        let url = format!("{}/api/version", self.host);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("Cannot reach Ollama at {}. Is it running?", self.host))?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Ollama health check failed (HTTP {}). Is it running at {}?",
                response.status(),
                self.host
            );
        }
        Ok(())
    }

    /// Calls the Ollama /api/chat endpoint.
    async fn call_api(&self, text: &str) -> Result<String> {
        let url = format!("{}/api/chat", self.host);
        let body = json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": GRAMMAR_CHECK_SYSTEM_PROMPT},
                {"role": "user", "content": format_user_message(text)}
            ],
            "stream": false,
            "options": {
                "temperature": 0.1
            }
        });

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("Failed to send request to Ollama")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Ollama returned HTTP {status}: {body:.300}");
        }

        let resp: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse Ollama response as JSON")?;

        // Ollama chat response structure: {"message": {"content": "..."}, ...}
        resp["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("Unexpected Ollama response shape: {resp}"))
    }
}

impl crate::providers::Provider for OllamaProvider {
    async fn check(&self, text: &str) -> Result<Vec<Edit>> {
        let json_response = self.call_api(text).await?;
        validator::parse_and_validate(&json_response, text)
    }

    fn provider_name(&self) -> &str {
        "ollama"
    }

    fn model_name(&self) -> &str {
        &self.model
    }
}
