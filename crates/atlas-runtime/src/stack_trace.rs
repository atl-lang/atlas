//! Stack trace helpers for runtime error reporting.

use crate::diagnostic::formatter::offset_to_line_col;
use crate::diagnostic::StackTraceFrame;
use crate::span::Span;
use std::fs;

/// Convert a span into a stack trace frame.
///
/// If `source_override` is provided and the span's file matches, line/column
/// are computed from the provided source text. Otherwise, the helper attempts
/// to read the file from disk. When neither is available, it falls back to
/// line 1 with a column based on the byte offset.
pub fn stack_frame_from_span(
    function: impl Into<String>,
    span: Span,
    source_override: Option<(&str, &str)>,
) -> StackTraceFrame {
    let file = span.file().to_string();
    let (line, column) = line_column_for_span(span, source_override);
    StackTraceFrame {
        function: function.into(),
        file,
        line,
        column,
    }
}

fn line_column_for_span(span: Span, source_override: Option<(&str, &str)>) -> (usize, usize) {
    let file = span.file();
    if let Some((source_file, source_text)) = source_override {
        if file.as_ref() == source_file {
            return offset_to_line_col(source_text, span.start);
        }
    }

    if file.starts_with('<') {
        return (1, span.start + 1);
    }

    if let Ok(source) = fs::read_to_string(file.as_ref()) {
        return offset_to_line_col(&source, span.start);
    }

    (1, span.start + 1)
}
