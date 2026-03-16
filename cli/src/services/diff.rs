//! Diff application service: applies Edit suggestions to text.
//!
//! Edits are applied in **reverse order** (highest start_index first) so
//! earlier positions remain valid after each substitution.

use anyhow::Result;

use crate::models::edit::Edit;
use crate::services::position::js_index_to_byte;

/// Applies a list of edits to the given text, returning the corrected text.
///
/// Edits are sorted by position (descending) before application to preserve
/// byte offsets for earlier edits.
pub fn apply_edits(text: &str, edits: &[Edit]) -> Result<String> {
    if edits.is_empty() {
        return Ok(text.to_string());
    }

    // Sort descending by start_index to apply from end to beginning
    let mut sorted: Vec<&Edit> = edits.iter().collect();
    sorted.sort_by(|a, b| b.start_index.cmp(&a.start_index));

    // Deduplicate overlapping edits (keep first in original order = last in reversed)
    let sorted = remove_overlapping(sorted);

    let mut result = text.to_string();

    for edit in sorted {
        let text_for_offsets = result.as_str();
        let start_byte = js_index_to_byte(text_for_offsets, edit.start_index)
            .map_err(|e| anyhow::anyhow!("Invalid edit start position {}: {e}", edit.start_index))?;
        let end_byte = js_index_to_byte(text_for_offsets, edit.end_index)
            .map_err(|e| anyhow::anyhow!("Invalid edit end position {}: {e}", edit.end_index))?;

        result.replace_range(start_byte..end_byte, &edit.replacement);
    }

    Ok(result)
}

/// Removes edits that overlap with higher-priority (earlier in original list) edits.
/// Input must be sorted descending by start_index.
fn remove_overlapping(edits: Vec<&Edit>) -> Vec<&Edit> {
    let mut result: Vec<&Edit> = Vec::new();
    let mut last_start: Option<usize> = None;

    for edit in edits {
        if let Some(last) = last_start {
            // If this edit ends after the start of the next edit (overlap), skip it
            if edit.end_index > last {
                continue;
            }
        }
        last_start = Some(edit.start_index);
        result.push(edit);
    }

    result
}

/// Returns true if two edits have overlapping ranges.
pub fn overlaps(a: &Edit, b: &Edit) -> bool {
    a.start_index < b.end_index && b.start_index < a.end_index
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::edit::EditCategory;

    fn make_edit(start: usize, end: usize, replacement: &str) -> Edit {
        Edit {
            start_index: start,
            end_index: end,
            replacement: replacement.to_string(),
            category: EditCategory::Grammar,
            explanation: None,
        }
    }

    #[test]
    fn test_single_edit() {
        let text = "He dont like it";
        let edits = vec![make_edit(3, 7, "doesn't")];
        let result = apply_edits(text, &edits).unwrap();
        assert_eq!(result, "He doesn't like it");
    }

    #[test]
    fn test_multiple_edits() {
        // "He dont like it and she dont either"
        //  0         1         2         3
        //  0123456789012345678901234567890123456
        // "dont" at 3-7, second "dont" at 24-28
        let text = "He dont like it and she dont either";
        let edits = vec![
            make_edit(3, 7, "doesn't"),
            make_edit(24, 28, "doesn't"),
        ];
        let result = apply_edits(text, &edits).unwrap();
        assert_eq!(result, "He doesn't like it and she doesn't either");
    }

    #[test]
    fn test_empty_edits() {
        let text = "No issues here.";
        let result = apply_edits(text, &[]).unwrap();
        assert_eq!(result, text);
    }

    #[test]
    fn test_utf16_len_ascii() {
        use crate::services::position::utf16_len;
        assert_eq!(utf16_len("hello"), 5);
    }
}
