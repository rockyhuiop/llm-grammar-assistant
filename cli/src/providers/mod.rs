//! LLM provider implementations (Ollama, OpenAI, Gemini).

pub mod cloud;
pub mod ollama;
pub mod prompts;

use crate::models::edit::Edit;
use anyhow::Result;

/// Trait for grammar checking providers. Implemented by both Ollama and cloud providers.
///
/// Uses `async fn` in trait (stable since Rust 1.75 via RPITIT).
/// Dispatch is done with concrete types — do not use `Box<dyn Provider>`.
#[allow(async_fn_in_trait)]
pub trait Provider {
    async fn check(&self, text: &str) -> Result<Vec<Edit>>;
    fn provider_name(&self) -> &str;
    fn model_name(&self) -> &str;
}
