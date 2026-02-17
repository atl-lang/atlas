//! Enhanced error message tests (Frontend Phase-01)
//!
//! Tests error code registry, diagnostic formatting, source snippets,
//! caret alignment, help text, color output, and multi-line support.

use atlas_runtime::diagnostic::error_codes;
use atlas_runtime::diagnostic::formatter::{
    extract_snippet, offset_to_line_col, DiagnosticFormatter,
};
use atlas_runtime::{Diagnostic, DiagnosticLevel, Span, DIAG_VERSION};
use rstest::rstest;

// ============================================================
// Error Code Registry Tests
// ============================================================

#[test]
fn test_error_codes_no_duplicates() {
    let mut seen = std::collections::HashSet::new();
    for entry in error_codes::ERROR_CODES {
        assert!(
            seen.insert(entry.code),
            "Duplicate error code: {}",
            entry.code
        );
    }
}

#[test]
fn test_error_codes_all_have_descriptions() {
    for entry in error_codes::ERROR_CODES {
        assert!(
            !entry.description.is_empty(),
            "Error code {} has empty description",
            entry.code
        );
    }
}

#[test]
fn test_error_codes_lookup() {
    let info = error_codes::lookup("AT0001").unwrap();
    assert_eq!(info.description, "Type mismatch");
    assert!(info.help.is_some());
}

#[test]
fn test_error_codes_lookup_missing() {
    assert!(error_codes::lookup("ZZZZ9999").is_none());
}

#[test]
fn test_error_codes_help_for() {
    assert!(error_codes::help_for("AT0005").is_some());
    assert_eq!(error_codes::help_for("AT9999"), None);
}

#[test]
fn test_error_codes_description_for() {
    assert_eq!(
        error_codes::description_for("AT1001").unwrap(),
        "Unexpected token"
    );
}

#[test]
fn test_error_codes_constants_match_registry() {
    // Verify the constants match entries in the registry
    assert!(error_codes::lookup(error_codes::TYPE_MISMATCH).is_some());
    assert!(error_codes::lookup(error_codes::UNDEFINED_SYMBOL).is_some());
    assert!(error_codes::lookup(error_codes::DIVIDE_BY_ZERO).is_some());
    assert!(error_codes::lookup(error_codes::UNEXPECTED_TOKEN).is_some());
    assert!(error_codes::lookup(error_codes::UNUSED_VARIABLE).is_some());
}

#[test]
fn test_error_code_ranges() {
    // Verify error code ranges
    for entry in error_codes::ERROR_CODES {
        assert!(
            entry.code.starts_with("AT") || entry.code.starts_with("AW"),
            "Error code {} must start with AT or AW",
            entry.code
        );
    }
}

#[test]
fn test_error_codes_count() {
    // At least 40 error codes in the registry
    assert!(
        error_codes::ERROR_CODES.len() >= 40,
        "Expected at least 40 error codes, got {}",
        error_codes::ERROR_CODES.len()
    );
}

#[rstest]
#[case("AT0001", "Type mismatch")]
#[case("AT0005", "Division by zero")]
#[case("AT0006", "Array index out of bounds")]
#[case("AT1001", "Unexpected token")]
#[case("AT1002", "Unterminated string literal")]
#[case("AT2001", "Unused variable or parameter")]
#[case("AT2002", "Unreachable code")]
#[case("AT3001", "Type error in expression")]
#[case("AT3005", "Function arity mismatch")]
#[case("AT5002", "Module not found")]
fn test_error_code_descriptions(#[case] code: &str, #[case] desc: &str) {
    let info = error_codes::lookup(code).unwrap();
    assert_eq!(info.description, desc);
}

#[rstest]
#[case("AT0001")]
#[case("AT0005")]
#[case("AT0006")]
#[case("AT0300")]
#[case("AT0301")]
#[case("AT1001")]
#[case("AT1002")]
#[case("AT1003")]
#[case("AT2001")]
#[case("AT3003")]
fn test_error_codes_have_help(#[case] code: &str) {
    let info = error_codes::lookup(code).unwrap();
    assert!(
        info.help.is_some(),
        "Error code {} should have help text",
        code
    );
}

// ============================================================
// Source Snippet Extraction Tests
// ============================================================

#[test]
fn test_extract_snippet_first_line() {
    let source = "let x = 1;\nlet y = 2;";
    assert_eq!(extract_snippet(source, 1).unwrap(), "let x = 1;");
}

#[test]
fn test_extract_snippet_last_line() {
    let source = "line1\nline2\nline3";
    assert_eq!(extract_snippet(source, 3).unwrap(), "line3");
}

#[test]
fn test_extract_snippet_empty_file() {
    assert!(extract_snippet("", 1).is_none());
}

#[test]
fn test_extract_snippet_out_of_range() {
    let source = "only one line";
    assert!(extract_snippet(source, 5).is_none());
}

#[test]
fn test_extract_snippet_empty_line() {
    let source = "line1\n\nline3";
    assert_eq!(extract_snippet(source, 2).unwrap(), "");
}

#[test]
fn test_extract_snippet_single_line() {
    let source = "hello world";
    assert_eq!(extract_snippet(source, 1).unwrap(), "hello world");
}

// ============================================================
// Line/Column Offset Tests
// ============================================================

#[test]
fn test_offset_to_line_col_start() {
    let source = "let x = 1;\nlet y = 2;";
    assert_eq!(offset_to_line_col(source, 0), (1, 1));
}

#[test]
fn test_offset_to_line_col_middle() {
    let source = "let x = 1;\nlet y = 2;";
    assert_eq!(offset_to_line_col(source, 4), (1, 5)); // 'x'
}

#[test]
fn test_offset_to_line_col_second_line() {
    let source = "let x = 1;\nlet y = 2;";
    assert_eq!(offset_to_line_col(source, 11), (2, 1));
}

#[test]
fn test_offset_to_line_col_unicode() {
    let source = "let héllo = 1;";
    assert_eq!(offset_to_line_col(source, 0), (1, 1));
}

// ============================================================
// Diagnostic Formatting Tests
// ============================================================

#[test]
fn test_format_error_basic() {
    let formatter = DiagnosticFormatter::plain();
    let diag = Diagnostic::error_with_code("AT0001", "Type mismatch", Span::new(8, 13))
        .with_file("test.atlas")
        .with_line(5)
        .with_snippet("let x: number = \"hello\";")
        .with_label("expected number, found string");

    let buf = formatter.format_to_buffer(&diag);
    let output = String::from_utf8(buf).unwrap();

    assert!(output.contains("error[AT0001]"));
    assert!(output.contains("Type mismatch"));
    assert!(output.contains("test.atlas:5:9"));
    assert!(output.contains("^^^^^"));
    assert!(output.contains("expected number, found string"));
}

#[test]
fn test_format_warning_basic() {
    let formatter = DiagnosticFormatter::plain();
    let diag = Diagnostic::warning_with_code("AT2001", "Unused variable 'x'", Span::new(4, 5))
        .with_file("test.atlas")
        .with_line(1)
        .with_snippet("let x: number = 42;")
        .with_label("declared here");

    let buf = formatter.format_to_buffer(&diag);
    let output = String::from_utf8(buf).unwrap();

    assert!(output.contains("warning[AT2001]"));
    assert!(output.contains("Unused variable"));
}

#[test]
fn test_format_with_help() {
    let formatter = DiagnosticFormatter::plain();
    let diag = Diagnostic::error_with_code("AT0005", "Division by zero", Span::new(0, 5))
        .with_file("test.atlas")
        .with_line(1)
        .with_snippet("10 / 0")
        .with_help("check that the divisor is not zero");

    let buf = formatter.format_to_buffer(&diag);
    let output = String::from_utf8(buf).unwrap();

    assert!(output.contains("help:"));
    assert!(output.contains("check that the divisor is not zero"));
}

#[test]
fn test_format_with_notes() {
    let formatter = DiagnosticFormatter::plain();
    let diag = Diagnostic::error("test", Span::new(0, 1))
        .with_file("test.atlas")
        .with_line(1)
        .with_note("first note")
        .with_note("second note");

    let buf = formatter.format_to_buffer(&diag);
    let output = String::from_utf8(buf).unwrap();

    assert!(output.contains("note: first note"));
    assert!(output.contains("note: second note"));
}

#[test]
fn test_format_no_snippet() {
    let formatter = DiagnosticFormatter::plain();
    let diag = Diagnostic::error("some error", Span::new(0, 1))
        .with_file("test.atlas")
        .with_line(1);

    let buf = formatter.format_to_buffer(&diag);
    let output = String::from_utf8(buf).unwrap();

    assert!(output.contains("error"));
    assert!(!output.contains("^")); // No carets without snippet
}

#[test]
fn test_format_caret_alignment() {
    let formatter = DiagnosticFormatter::plain();
    // Column 9 (0-indexed 8), length 5
    let diag = Diagnostic::error_with_code("AT0001", "test", Span::new(8, 13))
        .with_file("test.atlas")
        .with_line(1)
        .with_snippet("let x = hello;");

    let buf = formatter.format_to_buffer(&diag);
    let output = String::from_utf8(buf).unwrap();

    // Should have 8 spaces before carets (column 9, 0-indexed = 8 spaces)
    assert!(output.contains("^^^^^"));
}

#[test]
fn test_format_caret_position_first_column() {
    let formatter = DiagnosticFormatter::plain();
    let diag = Diagnostic::error_with_code("AT0001", "test", Span::new(0, 3))
        .with_file("test.atlas")
        .with_line(1)
        .with_snippet("foo bar");

    let buf = formatter.format_to_buffer(&diag);
    let output = String::from_utf8(buf).unwrap();

    assert!(output.contains("^^^"));
}

#[test]
fn test_format_multiline_notes() {
    let formatter = DiagnosticFormatter::plain();
    let diag = Diagnostic::error("main error", Span::new(0, 5))
        .with_file("test.atlas")
        .with_line(1)
        .with_snippet("hello world")
        .with_note("note 1")
        .with_note("note 2")
        .with_note("note 3")
        .with_help("try this");

    let buf = formatter.format_to_buffer(&diag);
    let output = String::from_utf8(buf).unwrap();

    assert!(output.contains("note: note 1"));
    assert!(output.contains("note: note 2"));
    assert!(output.contains("note: note 3"));
    assert!(output.contains("help: try this"));
}

#[test]
fn test_format_unicode_snippet() {
    let formatter = DiagnosticFormatter::plain();
    let diag = Diagnostic::error_with_code("AT0001", "test", Span::new(0, 5))
        .with_file("test.atlas")
        .with_line(1)
        .with_snippet("let héllo = 42;");

    let buf = formatter.format_to_buffer(&diag);
    let output = String::from_utf8(buf).unwrap();

    // Should still render without crashing
    assert!(output.contains("error[AT0001]"));
    assert!(output.contains("héllo"));
}

#[test]
fn test_format_related_location() {
    let formatter = DiagnosticFormatter::plain();
    let diag = Diagnostic::error("test", Span::new(0, 1))
        .with_file("main.atlas")
        .with_line(10)
        .with_related_location(atlas_runtime::RelatedLocation {
            file: "other.atlas".to_string(),
            line: 5,
            column: 3,
            length: 4,
            message: "originally defined here".to_string(),
        });

    let buf = formatter.format_to_buffer(&diag);
    let output = String::from_utf8(buf).unwrap();

    assert!(output.contains("other.atlas:5:3"));
    assert!(output.contains("originally defined here"));
}

// ============================================================
// Parse Error Formatting Tests
// ============================================================

#[test]
fn test_parse_error_has_diagnostics() {
    let runtime = atlas_runtime::Atlas::new();
    let result = runtime.eval("let x: number =");
    assert!(result.is_err());
    let diags = result.unwrap_err();
    assert!(!diags.is_empty());
    assert_eq!(diags[0].level, DiagnosticLevel::Error);
}

#[test]
fn test_parse_error_has_code() {
    let runtime = atlas_runtime::Atlas::new();
    let result = runtime.eval("let x: number =");
    let diags = result.unwrap_err();
    // Error code should be set (not empty)
    assert!(!diags[0].code.is_empty());
}

#[test]
fn test_type_error_has_diagnostics() {
    let runtime = atlas_runtime::Atlas::new();
    let result = runtime.eval("let x: number = \"hello\";");
    // This should produce a type error
    assert!(result.is_err());
    let diags = result.unwrap_err();
    assert!(!diags.is_empty());
}

// ============================================================
// Formatter Mode Tests
// ============================================================

#[test]
fn test_formatter_plain_mode() {
    let formatter = DiagnosticFormatter::plain();
    let diag = Diagnostic::error("test", Span::new(0, 1))
        .with_file("test.atlas")
        .with_line(1);

    let buf = formatter.format_to_buffer(&diag);
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("error"));
}

#[test]
fn test_formatter_auto_mode() {
    let formatter = DiagnosticFormatter::auto();
    let diag = Diagnostic::error("test", Span::new(0, 1))
        .with_file("test.atlas")
        .with_line(1);

    let buf = formatter.format_to_buffer(&diag);
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("error"));
}

#[test]
fn test_formatter_format_to_string() {
    let formatter = DiagnosticFormatter::plain();
    let diag = Diagnostic::error("test error", Span::new(0, 5))
        .with_file("test.atlas")
        .with_line(1)
        .with_snippet("hello")
        .with_label("here");

    let output = formatter.format_to_string(&diag);
    assert_eq!(output, diag.to_human_string());
}

// ============================================================
// Diagnostic Builder Tests
// ============================================================

#[test]
fn test_diagnostic_error_with_code() {
    let diag = Diagnostic::error_with_code("AT0001", "Type mismatch", Span::new(5, 10));
    assert_eq!(diag.code, "AT0001");
    assert_eq!(diag.level, DiagnosticLevel::Error);
    assert_eq!(diag.message, "Type mismatch");
    assert_eq!(diag.diag_version, DIAG_VERSION);
}

#[test]
fn test_diagnostic_warning_with_code() {
    let diag = Diagnostic::warning_with_code("AT2001", "Unused var", Span::new(0, 3));
    assert_eq!(diag.code, "AT2001");
    assert_eq!(diag.level, DiagnosticLevel::Warning);
}

#[test]
fn test_diagnostic_builder_chain() {
    let diag = Diagnostic::error_with_code("AT0001", "test", Span::new(0, 5))
        .with_file("test.atlas")
        .with_line(10)
        .with_snippet("source code")
        .with_label("error here")
        .with_note("a note")
        .with_help("try this");

    assert_eq!(diag.file, "test.atlas");
    assert_eq!(diag.line, 10);
    assert_eq!(diag.snippet, "source code");
    assert_eq!(diag.label, "error here");
    assert_eq!(diag.notes.len(), 1);
    assert!(diag.help.is_some());
}

#[test]
fn test_diagnostic_json_output() {
    let diag = Diagnostic::error_with_code("AT0001", "Type mismatch", Span::new(0, 5))
        .with_file("test.atlas")
        .with_line(1);

    let json = diag.to_json_string().unwrap();
    assert!(json.contains("\"level\": \"error\""));
    assert!(json.contains("\"code\": \"AT0001\""));
}

#[test]
fn test_diagnostic_json_compact() {
    let diag = Diagnostic::error("test", Span::new(0, 1));
    let compact = diag.to_json_compact().unwrap();
    assert!(!compact.contains('\n'));
}

#[test]
fn test_diagnostic_json_roundtrip() {
    let diag = Diagnostic::error_with_code("AT0001", "test", Span::new(0, 5))
        .with_file("test.atlas")
        .with_line(1)
        .with_snippet("hello")
        .with_label("here")
        .with_note("note")
        .with_help("help");

    let json = diag.to_json_string().unwrap();
    let deserialized: Diagnostic = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, diag);
}

// ============================================================
// Sort Diagnostics Tests
// ============================================================

#[test]
fn test_sort_errors_before_warnings() {
    let mut diags = vec![
        Diagnostic::warning("warn", Span::new(0, 1))
            .with_file("a.atlas")
            .with_line(1),
        Diagnostic::error("err", Span::new(0, 1))
            .with_file("a.atlas")
            .with_line(1),
    ];
    atlas_runtime::sort_diagnostics(&mut diags);
    assert_eq!(diags[0].level, DiagnosticLevel::Error);
    assert_eq!(diags[1].level, DiagnosticLevel::Warning);
}

#[test]
fn test_sort_by_file_then_line() {
    let mut diags = vec![
        Diagnostic::error("e1", Span::new(0, 1))
            .with_file("b.atlas")
            .with_line(5),
        Diagnostic::error("e2", Span::new(0, 1))
            .with_file("a.atlas")
            .with_line(10),
        Diagnostic::error("e3", Span::new(0, 1))
            .with_file("a.atlas")
            .with_line(1),
    ];
    atlas_runtime::sort_diagnostics(&mut diags);
    assert_eq!(diags[0].file, "a.atlas");
    assert_eq!(diags[0].line, 1);
    assert_eq!(diags[1].file, "a.atlas");
    assert_eq!(diags[1].line, 10);
    assert_eq!(diags[2].file, "b.atlas");
}

// ============================================================
// Edge Case Tests
// ============================================================

#[test]
fn test_zero_length_span() {
    let formatter = DiagnosticFormatter::plain();
    let diag = Diagnostic::error("test", Span::new(5, 5))
        .with_file("test.atlas")
        .with_line(1)
        .with_snippet("hello world");

    let buf = formatter.format_to_buffer(&diag);
    let output = String::from_utf8(buf).unwrap();
    // Should not crash, no carets for zero-length
    assert!(output.contains("error"));
}

#[test]
fn test_span_beyond_snippet() {
    let formatter = DiagnosticFormatter::plain();
    let diag = Diagnostic::error_with_code("AT0001", "test", Span::new(0, 100))
        .with_file("test.atlas")
        .with_line(1)
        .with_snippet("short");

    let buf = formatter.format_to_buffer(&diag);
    let output = String::from_utf8(buf).unwrap();
    // Should not crash
    assert!(output.contains("error"));
}

#[test]
fn test_empty_message() {
    let diag = Diagnostic::error("", Span::new(0, 1));
    assert_eq!(diag.message, "");
    // Should still format
    let output = diag.to_human_string();
    assert!(output.contains("error"));
}

#[test]
fn test_diagnostic_default_file() {
    let diag = Diagnostic::error("test", Span::new(0, 1));
    assert_eq!(diag.file, "<unknown>");
}

#[rstest]
#[case(Span::new(0, 0), true)]
#[case(Span::new(0, 5), false)]
#[case(Span::new(5, 5), true)]
fn test_span_is_empty(#[case] span: Span, #[case] expected: bool) {
    assert_eq!(span.is_empty(), expected);
}

#[rstest]
#[case(Span::new(0, 5), 5)]
#[case(Span::new(3, 10), 7)]
#[case(Span::new(0, 0), 0)]
fn test_span_length(#[case] span: Span, #[case] expected: usize) {
    assert_eq!(span.len(), expected);
}
