//! Human-readable terminal output with color coding.
//!
//! Grammar errors shown in red, style issues in yellow.
//! Context lines show the surrounding text with the error highlighted.

use colored::Colorize;

use crate::models::edit::{CheckResult, Edit, EditCategory};
use crate::services::position::utf16_slice;

/// Prints the grammar check result in human-readable format to stdout.
pub fn print_result(result: &CheckResult) {
    if result.edits.is_empty() {
        println!("{}", "No issues found.".green());
        return;
    }

    let count = result.edits.len();
    let issue_word = if count == 1 { "issue" } else { "issues" };
    println!(
        "{} {} found ({}ms, {} via {}):\n",
        count.to_string().bold(),
        issue_word,
        result.metadata.processing_time_ms,
        result.metadata.mode,
        result.metadata.provider
    );

    for (i, edit) in result.edits.iter().enumerate() {
        print_edit(i + 1, edit, &result.original_text);
    }
}

fn print_edit(index: usize, edit: &Edit, text: &str) {
    let text_utf16_len = text.encode_utf16().count();
    let safe_end = edit.end_index.min(text_utf16_len);
    let safe_start = edit.start_index.min(safe_end);

    let original = utf16_slice(text, safe_start, safe_end).unwrap_or("[invalid range]");

    // Build highlighted context (up to 60 chars each side)
    let context_before_start = safe_start.saturating_sub(60);
    let context_after_end = (safe_end + 60).min(text_utf16_len);

    let before = utf16_slice(text, context_before_start, safe_start).unwrap_or("");
    let after = utf16_slice(text, safe_end, context_after_end).unwrap_or("");

    let (label, colored_original) = match edit.category {
        EditCategory::Grammar => ("Grammar".red().bold(), original.red().bold()),
        EditCategory::Style => ("Style".yellow().bold(), original.yellow().bold()),
    };

    println!("{}. [{}]", index, label);

    // Context with highlighted error
    let prefix = if context_before_start > 0 { "…" } else { "" };
    let suffix = if context_after_end < text_utf16_len { "…" } else { "" };
    println!("   {}{}{}{}{}", prefix, before, colored_original, after, suffix);

    // Arrow showing replacement
    println!(
        "   {} {}  →  {}",
        format!("col {}", safe_start).dimmed(),
        original.red(),
        edit.replacement.green().bold()
    );

    if let Some(explanation) = &edit.explanation {
        println!("   {}", explanation.dimmed());
    }
    println!();
}
