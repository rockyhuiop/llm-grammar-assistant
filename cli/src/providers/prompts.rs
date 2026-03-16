//! System prompt templates for grammar checking LLMs.
//!
//! These prompts instruct the LLM to return JSON matching the shared schema.

/// The system prompt instructing the LLM to act as a grammar checker.
/// Returns JSON with `{"edits": [...]}` format matching the JSON Schema contract.
pub const GRAMMAR_CHECK_SYSTEM_PROMPT: &str = r#"You are a precise grammar and style checker. Analyze the provided text and return ONLY a JSON object with suggested corrections.

Return format (strict JSON, no markdown, no explanation outside JSON):
{
  "edits": [
    {
      "start_index": <integer: UTF-16 code unit index, 0-based, inclusive>,
      "end_index": <integer: UTF-16 code unit index, exclusive>,
      "replacement": "<corrected text>",
      "category": "<grammar|style>",
      "explanation": "<optional brief reason>"
    }
  ]
}

Rules:
- Use "grammar" for: subject-verb disagreement, wrong tense, missing articles, pronoun errors, spelling mistakes
- Use "style" for: passive voice where active is clearer, wordiness, repeated words, unclear phrasing
- Positions are UTF-16 code unit indices (same as JavaScript's string.slice())
- Return {"edits": []} if the text is correct
- Only flag real errors, not stylistic preferences
- Do NOT return any text outside the JSON object"#;

/// Wraps user text in the format expected by the LLM.
pub fn format_user_message(text: &str) -> String {
    format!("Check the following text for grammar and style issues:\n\n{text}")
}
