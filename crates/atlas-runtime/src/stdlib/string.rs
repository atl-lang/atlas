//! String manipulation functions
//!
//! Complete string API with Unicode support

use crate::span::Span;
use crate::value::{RuntimeError, Value};

/// Maximum repeat count to prevent memory abuse
const MAX_REPEAT_COUNT: i64 = 1_000_000;

// ============================================================================
// Core Operations
// ============================================================================

/// Split a string by separator
///
/// Returns an array of string parts. If separator is empty, returns array of individual characters.
pub fn split(s: &str, separator: &str, _span: Span) -> Result<Value, RuntimeError> {
    if separator.is_empty() {
        // Split into individual characters
        let chars: Vec<Value> = s.chars().map(|c| Value::string(c.to_string())).collect();
        Ok(Value::array(chars))
    } else {
        let parts: Vec<Value> = s
            .split(separator)
            .map(|part| Value::string(part.to_string()))
            .collect();
        Ok(Value::array(parts))
    }
}

/// Join an array of strings with separator
pub fn join(parts: &[Value], separator: &str, span: Span) -> Result<String, RuntimeError> {
    let mut strings = Vec::with_capacity(parts.len());

    for part in parts {
        match part {
            Value::String(s) => strings.push(s.as_ref().clone()),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "join() requires array of strings".to_string(),
                    span,
                })
            }
        }
    }

    Ok(strings.join(separator))
}

/// Trim leading and trailing whitespace (Unicode-aware)
pub fn trim(s: &str) -> String {
    s.trim().to_string()
}

/// Trim leading whitespace (Unicode-aware)
pub fn trim_start(s: &str) -> String {
    s.trim_start().to_string()
}

/// Trim trailing whitespace (Unicode-aware)
pub fn trim_end(s: &str) -> String {
    s.trim_end().to_string()
}

// ============================================================================
// Search Operations
// ============================================================================

/// Find first occurrence index
///
/// Returns Option: Some(index) if found, None if not found.
pub fn index_of(haystack: &str, needle: &str) -> Option<f64> {
    if needle.is_empty() {
        return Some(0.0); // Empty string is at index 0
    }

    haystack
        .find(needle)
        .map(|idx| haystack[..idx].chars().count() as f64)
}

/// Find last occurrence index
///
/// Returns Option: Some(index) if found, None if not found.
pub fn last_index_of(haystack: &str, needle: &str) -> Option<f64> {
    if needle.is_empty() {
        return Some(haystack.chars().count() as f64); // Empty string is at the end
    }

    haystack
        .rfind(needle)
        .map(|idx| haystack[..idx].chars().count() as f64)
}

/// Check if string contains substring
pub fn includes(haystack: &str, needle: &str) -> bool {
    haystack.contains(needle)
}

// ============================================================================
// Transformation
// ============================================================================

/// Convert to uppercase (Unicode-aware)
pub fn to_upper_case(s: &str) -> String {
    s.to_uppercase()
}

/// Convert to lowercase (Unicode-aware)
pub fn to_lower_case(s: &str) -> String {
    s.to_lowercase()
}

/// Extract substring from start to end (UTF-8 boundary safe)
///
/// Returns substring from start (inclusive) to end (exclusive).
/// Validates UTF-8 boundaries and checks bounds.
pub fn substring(s: &str, start: f64, end: f64, span: Span) -> Result<String, RuntimeError> {
    // Validate indices are integers
    if start.fract() != 0.0 || end.fract() != 0.0 {
        return Err(RuntimeError::TypeError {
            msg: "substring() indices must be integers".to_string(),
            span,
        });
    }

    let start_char = start as usize;
    let end_char = end as usize;
    let char_len = s.chars().count();

    // Validate bounds (char-based, consistent with indexOf/length)
    if start_char > end_char {
        return Err(RuntimeError::OutOfBounds { span });
    }

    if start_char > char_len || end_char > char_len {
        return Err(RuntimeError::OutOfBounds { span });
    }

    // Convert char indices to byte indices
    let mut char_indices = s.char_indices();
    let start_byte = if start_char == 0 {
        0
    } else {
        char_indices
            .nth(start_char - 1)
            .map(|(byte_pos, ch)| byte_pos + ch.len_utf8())
            .unwrap_or(s.len())
    };
    // Re-advance from start_char to end_char
    let remaining = end_char - start_char;
    let end_byte = if remaining == 0 {
        start_byte
    } else {
        // Restart iteration from start_byte
        s[start_byte..]
            .char_indices()
            .nth(remaining - 1)
            .map(|(off, ch)| start_byte + off + ch.len_utf8())
            .unwrap_or(s.len())
    };

    Ok(s[start_byte..end_byte].to_string())
}

/// Get character at index (returns grapheme cluster, not byte)
///
/// Returns Option: Some(char) if index is valid, None if out of bounds.
/// Returns error only for non-integer index.
pub fn char_at(s: &str, index: f64, span: Span) -> Result<Option<String>, RuntimeError> {
    // Validate index is integer
    if index.fract() != 0.0 {
        return Err(RuntimeError::TypeError {
            msg: "charAt() index must be an integer".to_string(),
            span,
        });
    }

    let idx = index as usize;

    // Get character at index
    Ok(s.chars().nth(idx).map(|c| c.to_string()))
}

/// Repeat string count times
///
/// Limits count to prevent memory abuse.
pub fn repeat(s: &str, count: f64, span: Span) -> Result<String, RuntimeError> {
    // Validate count is integer
    if count.fract() != 0.0 {
        return Err(RuntimeError::TypeError {
            msg: "repeat() count must be an integer".to_string(),
            span,
        });
    }

    let count_i64 = count as i64;

    // Negative count is error
    if count_i64 < 0 {
        return Err(RuntimeError::TypeError {
            msg: "repeat() count cannot be negative".to_string(),
            span,
        });
    }

    // Limit count to prevent memory abuse
    if count_i64 > MAX_REPEAT_COUNT {
        return Err(RuntimeError::InvalidNumericResult { span });
    }

    Ok(s.repeat(count_i64 as usize))
}

/// Replace first occurrence
///
/// Replaces only the first occurrence of search with replacement.
pub fn replace(s: &str, search: &str, replacement: &str) -> String {
    if search.is_empty() {
        // Empty search returns original string
        return s.to_string();
    }

    s.replacen(search, replacement, 1)
}

/// Replace all occurrences
///
/// Replaces every occurrence of search with replacement.
pub fn replace_all(s: &str, search: &str, replacement: &str) -> String {
    if search.is_empty() {
        return s.to_string();
    }

    s.replace(search, replacement)
}

// ============================================================================
// Formatting
// ============================================================================

/// Pad start to reach target length
///
/// If string is already >= length, returns original string.
/// Fill string is repeated as needed.
pub fn pad_start(s: &str, length: f64, fill: &str, span: Span) -> Result<String, RuntimeError> {
    // Validate length is integer
    if length.fract() != 0.0 {
        return Err(RuntimeError::TypeError {
            msg: "padStart() length must be an integer".to_string(),
            span,
        });
    }

    let target_len = length as usize;
    let current_len = s.chars().count();

    if current_len >= target_len {
        return Ok(s.to_string());
    }

    if fill.is_empty() {
        return Ok(s.to_string());
    }

    let padding_needed = target_len - current_len;
    let fill_chars: Vec<char> = fill.chars().collect();
    let fill_len = fill_chars.len();

    let mut result = String::with_capacity(target_len);

    // Add padding
    for i in 0..padding_needed {
        result.push(fill_chars[i % fill_len]);
    }

    // Add original string
    result.push_str(s);

    Ok(result)
}

/// Pad end to reach target length
///
/// If string is already >= length, returns original string.
/// Fill string is repeated as needed.
pub fn pad_end(s: &str, length: f64, fill: &str, span: Span) -> Result<String, RuntimeError> {
    // Validate length is integer
    if length.fract() != 0.0 {
        return Err(RuntimeError::TypeError {
            msg: "padEnd() length must be an integer".to_string(),
            span,
        });
    }

    let target_len = length as usize;
    let current_len = s.chars().count();

    if current_len >= target_len {
        return Ok(s.to_string());
    }

    if fill.is_empty() {
        return Ok(s.to_string());
    }

    let padding_needed = target_len - current_len;
    let fill_chars: Vec<char> = fill.chars().collect();
    let fill_len = fill_chars.len();

    let mut result = String::with_capacity(target_len);

    // Add original string
    result.push_str(s);

    // Add padding
    for i in 0..padding_needed {
        result.push(fill_chars[i % fill_len]);
    }

    Ok(result)
}

/// Check if string starts with prefix
pub fn starts_with(s: &str, prefix: &str) -> bool {
    s.starts_with(prefix)
}

/// Check if string ends with suffix
pub fn ends_with(s: &str, suffix: &str) -> bool {
    s.ends_with(suffix)
}
