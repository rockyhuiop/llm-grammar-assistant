//! Unicode-safe position handling utilities.
//!
//! The JSON Schema uses UTF-16 code unit indices (JavaScript string positions).
//! This module provides conversions between Rust byte offsets and JS indices.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PositionError {
    #[error("Position {0} is out of bounds for text of length {1}")]
    OutOfBounds(usize, usize),
    #[error("Position {0} splits a UTF-16 surrogate pair")]
    SplitsSurrogatePair(usize),
}

/// Converts a Rust byte offset to a UTF-16 code unit index (JavaScript position).
pub fn byte_to_js_index(text: &str, byte_offset: usize) -> usize {
    let mut js_index = 0;
    for (byte_pos, c) in text.char_indices() {
        if byte_pos >= byte_offset {
            break;
        }
        // Characters above U+FFFF take 2 UTF-16 code units (surrogate pair)
        js_index += if c as u32 > 0xFFFF { 2 } else { 1 };
    }
    js_index
}

/// Converts a UTF-16 code unit index to a Rust byte offset.
pub fn js_index_to_byte(text: &str, js_index: usize) -> Result<usize, PositionError> {
    let text_len_utf16 = text.encode_utf16().count();
    if js_index > text_len_utf16 {
        return Err(PositionError::OutOfBounds(js_index, text_len_utf16));
    }

    let mut current_js = 0;
    for (byte_pos, c) in text.char_indices() {
        if current_js == js_index {
            return Ok(byte_pos);
        }
        let units = if c as u32 > 0xFFFF { 2 } else { 1 };
        if current_js + units > js_index {
            return Err(PositionError::SplitsSurrogatePair(js_index));
        }
        current_js += units;
    }
    // End of string
    Ok(text.len())
}

/// Returns the UTF-16 length of a string (number of code units).
pub fn utf16_len(text: &str) -> usize {
    text.encode_utf16().count()
}

/// Extracts the substring at the given UTF-16 code unit range.
pub fn utf16_slice(text: &str, start: usize, end: usize) -> Result<&str, PositionError> {
    let start_byte = js_index_to_byte(text, start)?;
    let end_byte = js_index_to_byte(text, end)?;
    Ok(&text[start_byte..end_byte])
}

/// Normalizes CRLF line endings to LF for consistent position handling.
pub fn normalize_line_endings(text: &str) -> std::borrow::Cow<'_, str> {
    if text.contains('\r') {
        std::borrow::Cow::Owned(text.replace("\r\n", "\n").replace('\r', "\n"))
    } else {
        std::borrow::Cow::Borrowed(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_positions() {
        let text = "Hello, world!";
        assert_eq!(byte_to_js_index(text, 7), 7);
        assert_eq!(js_index_to_byte(text, 7).unwrap(), 7);
    }

    #[test]
    fn test_emoji_surrogate_pair() {
        // "😀" is U+1F600, takes 2 UTF-16 code units
        let text = "Hi 😀!";
        // "Hi " = 3 chars, "😀" = 2 UTF-16 units, "!" = 1
        assert_eq!(utf16_len(text), 6); // 3 + 2 + 1
        assert_eq!(byte_to_js_index(text, 3), 3); // before emoji
        // After emoji: 3 JS units for "Hi ", 2 for emoji = JS index 5
        assert_eq!(byte_to_js_index(text, 7), 5); // after emoji (4-byte char)
    }
}
