//! Edit and CheckResult types matching the JSON Schema contract.
//!
//! All positions use UTF-16 code unit indices (same as JavaScript string indices).

use serde::{Deserialize, Serialize};

use crate::models::config::OperatingMode;

/// A single suggested text correction returned by the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edit {
    /// Start position in original text (UTF-16 code unit index, 0-based, inclusive).
    pub start_index: usize,
    /// End position in original text (exclusive). Must be > start_index.
    pub end_index: usize,
    /// The suggested replacement text. May be empty for deletions.
    pub replacement: String,
    /// Issue category: grammar or style.
    pub category: EditCategory,
    /// Optional explanation of why this correction is suggested.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
}

/// Category of issue identified by the grammar checker.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EditCategory {
    Grammar,
    Style,
}

impl EditCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            EditCategory::Grammar => "grammar",
            EditCategory::Style => "style",
        }
    }
}

/// The result of a grammar check operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// The original text that was checked.
    pub original_text: String,
    /// List of suggested corrections (may be empty).
    pub edits: Vec<Edit>,
    /// Processing metadata.
    pub metadata: ProcessingMetadata,
}

/// Metadata about how the grammar check was processed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingMetadata {
    /// The operating mode used.
    pub mode: OperatingMode,
    /// Provider name: "ollama", "openai", or "gemini".
    pub provider: String,
    /// Model name: "llama3", "gpt-4o-mini", etc.
    pub model: String,
    /// Total processing time in milliseconds.
    pub processing_time_ms: u64,
    /// Number of chunks processed (for large documents).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunks_processed: Option<u32>,
}

/// The deserialized wrapper around the LLM's JSON response.
#[derive(Debug, Deserialize)]
pub struct LlmResponse {
    pub edits: Vec<Edit>,
}
