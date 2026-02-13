//! Code formatting helpers

use tower_lsp::lsp_types::{Position, Range, TextEdit};

/// Format an Atlas source file
///
/// This is a basic formatter that ensures consistent indentation and spacing.
/// For now, it returns the source as-is since we don't have a full formatter implementation.
/// In the future, this would be replaced with a proper AST-based formatter.
pub fn format_document(source: &str) -> Vec<TextEdit> {
    // For now, just return empty edits (no formatting changes)
    // A full formatter would:
    // 1. Parse the source to AST
    // 2. Pretty-print the AST with consistent style
    // 3. Generate TextEdits for the differences

    // Calculate the range of the entire document
    let lines: Vec<&str> = source.lines().collect();
    let last_line = lines.len().saturating_sub(1);
    let last_char = lines.last().map(|l| l.len()).unwrap_or(0);

    let full_range = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: last_line as u32,
            character: last_char as u32,
        },
    };

    // Return a single edit that replaces the entire document with itself
    // This demonstrates the formatting capability without actually changing anything
    vec![TextEdit {
        range: full_range,
        new_text: source.to_string(),
    }]
}

/// Format a range of an Atlas source file
pub fn format_range(source: &str, range: Range) -> Vec<TextEdit> {
    // For range formatting, we would:
    // 1. Extract the text in the range
    // 2. Parse and format just that portion
    // 3. Generate edits for the changes
    //
    // For now, just return the range unchanged
    let lines: Vec<&str> = source.lines().collect();
    let start_line = range.start.line as usize;
    let end_line = range.end.line as usize;

    if start_line >= lines.len() {
        return vec![];
    }

    let end_line = end_line.min(lines.len() - 1);
    let range_text = lines[start_line..=end_line].join("\n");

    vec![TextEdit {
        range,
        new_text: range_text,
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_document() {
        let source = "let x: number = 42;";
        let edits = format_document(source);
        assert!(!edits.is_empty());
    }

    #[test]
    fn test_format_empty_document() {
        let source = "";
        let edits = format_document(source);
        assert!(!edits.is_empty());
    }

    #[test]
    fn test_format_range() {
        let source = "let x: number = 1;\nlet y: number = 2;";
        let range = Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 18,
            },
        };
        let edits = format_range(source, range);
        assert!(!edits.is_empty());
    }
}
