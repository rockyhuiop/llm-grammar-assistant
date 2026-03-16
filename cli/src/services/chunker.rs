//! Text chunking for large documents.
//!
//! Splits documents into semantic chunks with overlap for reliable grammar checking.
//! Strategy: 3,500 char target size, 400 char overlap, paragraph/sentence boundaries.

use crate::models::edit::Edit;
use std::ops::Range;

/// Configuration for the text chunker.
#[derive(Debug, Clone)]
pub struct ChunkingOptions {
    /// Target chunk size in UTF-16 code units.
    pub target_size: usize,
    /// Overlap on each side in UTF-16 code units.
    pub overlap_size: usize,
}

impl Default for ChunkingOptions {
    fn default() -> Self {
        Self {
            target_size: 3_500,
            overlap_size: 400,
        }
    }
}

/// A text chunk with position tracking for edit offset correction.
#[derive(Debug, Clone)]
pub struct Chunk {
    /// The chunk text (including overlap regions).
    pub text: String,
    /// Start byte offset in the full document.
    pub doc_start_byte: usize,
    /// The "primary region" — UTF-16 range relative to chunk start
    /// within which reported edits are considered canonical.
    pub primary_range: Range<usize>,
}

/// Splits text into overlapping chunks at semantic boundaries.
///
/// Returns chunks that together cover the entire document. Each chunk
/// extends into overlap regions for context, but only the primary zone
/// is used for edit reporting (to avoid duplicates).
pub fn chunk_text(text: &str, opts: &ChunkingOptions) -> Vec<Chunk> {
    if text.chars().count() <= opts.target_size + opts.overlap_size * 2 {
        // Small enough to process as one chunk
        return vec![Chunk {
            text: text.to_string(),
            doc_start_byte: 0,
            primary_range: 0..text.encode_utf16().count(),
        }];
    }

    let mut chunks = Vec::new();
    let mut pos = 0usize; // current byte position

    while pos < text.len() {
        // Find the end of this chunk (target_size chars forward)
        let chunk_end = advance_chars(text, pos, opts.target_size);

        // Extend to semantic boundary (paragraph or sentence)
        let semantic_end = find_semantic_boundary(text, chunk_end);

        // Primary region: [pos, semantic_end)
        let primary_start_byte = pos;
        let primary_end_byte = semantic_end.min(text.len());

        // Extend with overlap for context
        let chunk_start_byte = retreat_bytes(text, pos, opts.overlap_size);
        let chunk_end_byte = advance_bytes(text, primary_end_byte, opts.overlap_size);

        let chunk_text = text[chunk_start_byte..chunk_end_byte].to_string();

        // Calculate primary range in UTF-16 units relative to chunk start
        let prefix_utf16 = text[chunk_start_byte..primary_start_byte]
            .encode_utf16()
            .count();
        let primary_utf16_len = text[primary_start_byte..primary_end_byte]
            .encode_utf16()
            .count();

        chunks.push(Chunk {
            text: chunk_text,
            doc_start_byte: chunk_start_byte,
            primary_range: prefix_utf16..(prefix_utf16 + primary_utf16_len),
        });

        if primary_end_byte >= text.len() {
            break;
        }
        pos = primary_end_byte;
    }

    chunks
}

/// Adjusts edit positions from chunk-local UTF-16 indices to document-level indices,
/// filtering edits outside the primary region. Then deduplicates and sorts.
pub fn merge_chunk_edits(chunks: &[Chunk], chunk_edits: Vec<Vec<Edit>>, full_text: &str) -> Vec<Edit> {
    let mut all_edits: Vec<Edit> = Vec::new();

    for (chunk, edits) in chunks.iter().zip(chunk_edits.into_iter()) {
        // UTF-16 offset of this chunk's start within the full document.
        // Edits from this chunk are in chunk-local coordinates starting at 0.
        let chunk_doc_utf16 = full_text[..chunk.doc_start_byte].encode_utf16().count();

        for edit in edits {
            // Discard edits outside the primary (non-overlap) zone
            if edit.start_index < chunk.primary_range.start
                || edit.end_index > chunk.primary_range.end
            {
                continue;
            }

            // Translate chunk-local position to document position
            all_edits.push(Edit {
                start_index: edit.start_index + chunk_doc_utf16,
                end_index: edit.end_index + chunk_doc_utf16,
                ..edit
            });
        }
    }

    // Deduplicate by (start_index, end_index) keeping first occurrence
    all_edits.sort_by_key(|e| (e.start_index, e.end_index));
    all_edits.dedup_by_key(|e| (e.start_index, e.end_index));

    all_edits
}

/// Advances byte position by approximately `char_count` characters.
fn advance_chars(text: &str, start: usize, char_count: usize) -> usize {
    let mut pos = start;
    for (count, (byte_pos, _c)) in text[start..].char_indices().enumerate() {
        if count >= char_count {
            return start + byte_pos;
        }
        pos = start + byte_pos;
    }
    text.len().min(pos + 1)
}

/// Advances by `bytes` bytes (clamped to string end).
fn advance_bytes(text: &str, pos: usize, bytes: usize) -> usize {
    let end = (pos + bytes).min(text.len());
    // Align to char boundary
    let mut aligned = end;
    while aligned < text.len() && !text.is_char_boundary(aligned) {
        aligned += 1;
    }
    aligned
}

/// Retreats by approximately `bytes` bytes (clamped to 0).
fn retreat_bytes(text: &str, pos: usize, bytes: usize) -> usize {
    let start = pos.saturating_sub(bytes);
    // Align to char boundary
    let mut aligned = start;
    while aligned < text.len() && !text.is_char_boundary(aligned) {
        aligned -= 1;
    }
    aligned
}

/// Finds the next paragraph or sentence boundary after `pos`.
fn find_semantic_boundary(text: &str, pos: usize) -> usize {
    if pos >= text.len() {
        return text.len();
    }

    let search_text = &text[pos..];
    let limit = search_text.len().min(800); // max lookahead

    // Prefer paragraph boundary (double newline)
    if let Some(idx) = search_text[..limit].find("\n\n") {
        return pos + idx + 2;
    }

    // Fall back to sentence boundary
    for (i, c) in search_text[..limit].char_indices() {
        if matches!(c, '.' | '!' | '?') {
            let next = i + c.len_utf8();
            // Ensure followed by whitespace or end
            if next >= search_text.len()
                || search_text[next..].starts_with(' ')
                || search_text[next..].starts_with('\n')
            {
                return pos + next;
            }
        }
    }

    // No boundary found, use hard limit
    pos + limit
}
