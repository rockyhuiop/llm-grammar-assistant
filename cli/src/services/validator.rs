//! JSON response validation for LLM output.
//!
//! Validates that edit positions are within document bounds and represent
//! actual changes. Filters out invalid edits rather than failing entirely.

use crate::models::edit::{Edit, LlmResponse};
use crate::services::position::utf16_len;

/// Parses and validates an LLM JSON response string.
///
/// Returns the list of valid edits, silently filtering out malformed ones.
/// This is intentional: partial results are more useful than total failure.
pub fn parse_and_validate(json: &str, original_text: &str) -> anyhow::Result<Vec<Edit>> {
    let response: LlmResponse = serde_json::from_str(json)
        .map_err(|e| anyhow::anyhow!("Invalid JSON from LLM: {e}\nResponse was: {json:.200}"))?;

    let text_len = utf16_len(original_text);
    let edits: Vec<Edit> = response
        .edits
        .into_iter()
        .filter(|edit| validate_edit(edit, original_text, text_len))
        .collect();

    Ok(edits)
}

/// Returns true if an edit passes all validation rules.
fn validate_edit(edit: &Edit, text: &str, text_utf16_len: usize) -> bool {
    // Positions must be valid range
    if edit.start_index >= edit.end_index {
        return false;
    }
    // Must be within document bounds
    if edit.end_index > text_utf16_len {
        return false;
    }
    // Must represent an actual change
    if let Ok(original) = crate::services::position::utf16_slice(text, edit.start_index, edit.end_index) {
        if original == edit.replacement {
            return false;
        }
    }
    true
}
