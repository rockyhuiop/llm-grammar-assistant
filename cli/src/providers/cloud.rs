//! Cloud provider implementations for OpenAI and Gemini.
//!
//! Both providers implement exponential backoff retry logic (3 attempts, 1s/2s/4s).

use anyhow::{Context, Result};
use serde_json::json;
use tokio::time::{sleep, Duration};

use crate::models::config::CloudProvider;
use crate::models::edit::Edit;
use crate::providers::prompts::{format_user_message, GRAMMAR_CHECK_SYSTEM_PROMPT};
use crate::services::validator;

/// Provider that calls cloud LLM APIs (OpenAI or Gemini).
pub struct CloudLlmProvider {
    provider: CloudProvider,
    model: String,
    api_key: String,
    client: reqwest::Client,
}

impl CloudLlmProvider {
    pub fn new(provider: CloudProvider, model: String, api_key: String) -> Self {
        Self {
            provider,
            model,
            api_key,
            client: reqwest::Client::new(),
        }
    }

    /// Executes the API call with exponential backoff retry (3 attempts).
    async fn call_with_retry(&self, text: &str) -> Result<String> {
        const MAX_ATTEMPTS: u32 = 3;
        let mut last_error = None;

        for attempt in 0..MAX_ATTEMPTS {
            if attempt > 0 {
                // Exponential backoff: 1s, 2s, 4s (+ small jitter via offset)
                let wait_ms = (1u64 << (attempt - 1)) * 1000 + 50;
                sleep(Duration::from_millis(wait_ms)).await;
            }

            match self.call_api(text).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    // Check if error is retryable
                    let err_str = e.to_string();
                    let retryable = err_str.contains("429")
                        || err_str.contains("502")
                        || err_str.contains("503")
                        || err_str.contains("timeout");

                    if !retryable {
                        return Err(e);
                    }
                    last_error = Some(e);
                    eprintln!(
                        "Request failed (attempt {}/{}), retrying...",
                        attempt + 1,
                        MAX_ATTEMPTS
                    );
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| anyhow::anyhow!("All {MAX_ATTEMPTS} attempts failed")))
    }

    async fn call_api(&self, text: &str) -> Result<String> {
        match self.provider {
            CloudProvider::OpenAI => self.call_openai(text).await,
            CloudProvider::Gemini => self.call_gemini(text).await,
        }
    }

    async fn call_openai(&self, text: &str) -> Result<String> {
        let url = "https://api.openai.com/v1/chat/completions";
        let schema = serde_json::json!({
            "type": "object",
            "required": ["edits"],
            "properties": {
                "edits": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["start_index", "end_index", "replacement", "category"],
                        "properties": {
                            "start_index": {"type": "integer", "minimum": 0},
                            "end_index": {"type": "integer", "minimum": 1},
                            "replacement": {"type": "string"},
                            "category": {"type": "string", "enum": ["grammar", "style"]},
                            "explanation": {"type": "string"}
                        },
                        "additionalProperties": false
                    }
                }
            },
            "additionalProperties": false
        });

        let body = json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": GRAMMAR_CHECK_SYSTEM_PROMPT},
                {"role": "user", "content": format_user_message(text)}
            ],
            "response_format": {
                "type": "json_schema",
                "json_schema": {
                    "name": "grammar_check",
                    "strict": true,
                    "schema": schema
                }
            }
        });

        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .context("Failed to send request to OpenAI")?;

        let status = response.status();
        if status == 401 {
            anyhow::bail!("OpenAI authentication failed. Check your API key with `grammar-check config --set-api-key`");
        }
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenAI returned HTTP {status}: {body:.300}");
        }

        let resp: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse OpenAI response")?;

        resp["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("Unexpected OpenAI response shape: {resp}"))
    }

    async fn call_gemini(&self, text: &str) -> Result<String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}/generateContent?key={}",
            self.model, self.api_key
        );

        let schema = serde_json::json!({
            "type": "OBJECT",
            "required": ["edits"],
            "properties": {
                "edits": {
                    "type": "ARRAY",
                    "items": {
                        "type": "OBJECT",
                        "required": ["start_index", "end_index", "replacement", "category"],
                        "properties": {
                            "start_index": {"type": "INTEGER"},
                            "end_index": {"type": "INTEGER"},
                            "replacement": {"type": "STRING"},
                            "category": {"type": "STRING", "enum": ["grammar", "style"]},
                            "explanation": {"type": "STRING"}
                        }
                    }
                }
            }
        });

        let body = json!({
            "contents": [{
                "parts": [
                    {"text": GRAMMAR_CHECK_SYSTEM_PROMPT},
                    {"text": format_user_message(text)}
                ]
            }],
            "generationConfig": {
                "responseMimeType": "application/json",
                "responseSchema": schema,
                "temperature": 0.1
            }
        });

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("Failed to send request to Gemini")?;

        let status = response.status();
        if status == 400 || status == 403 {
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Gemini request failed (HTTP {status}): {body:.300}");
        }
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Gemini returned HTTP {status}: {body:.300}");
        }

        let resp: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse Gemini response")?;

        resp["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("Unexpected Gemini response shape: {resp}"))
    }
}

impl crate::providers::Provider for CloudLlmProvider {
    async fn check(&self, text: &str) -> Result<Vec<Edit>> {
        let json_response = self.call_with_retry(text).await?;
        validator::parse_and_validate(&json_response, text)
    }

    fn provider_name(&self) -> &str {
        self.provider.as_str()
    }

    fn model_name(&self) -> &str {
        &self.model
    }
}
