//! Golden tests for lexer edge cases
//!
//! These tests verify that lexer error diagnostics are stable and precise.

use atlas_runtime::{diagnostic::Diagnostic, lexer::Lexer};
use std::fs;
use std::path::Path;

/// Helper to read test file and lex it
fn lex_file(filename: &str) -> (Vec<Diagnostic>, String) {
    let path = Path::new("tests/errors").join(filename);
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to read test file: {}", path.display()));

    let mut lexer = Lexer::new(&source);
    let (_, diagnostics) = lexer.tokenize();

    (diagnostics, source)
}

/// Helper to verify diagnostic has expected code
fn assert_has_error_code(diagnostics: &[Diagnostic], expected_code: &str) {
    assert!(
        !diagnostics.is_empty(),
        "Expected at least one diagnostic with code {}",
        expected_code
    );

    let has_code = diagnostics.iter().any(|d| d.code == expected_code);
    assert!(
        has_code,
        "Expected diagnostic code {} but got codes: {:?}",
        expected_code,
        diagnostics.iter().map(|d| &d.code).collect::<Vec<_>>()
    );
}

#[test]
fn test_golden_unterminated_string() {
    let (diagnostics, _) = lex_file("unterminated_string.atl");

    // Should report AT1002: Unterminated string
    assert_has_error_code(&diagnostics, "AT1002");

    // Verify diagnostic has proper structure
    let diag = &diagnostics[0];
    assert_eq!(diag.level, atlas_runtime::diagnostic::DiagnosticLevel::Error);
    assert!(diag.message.contains("Unterminated string"));
    assert!(diag.line >= 1);
    assert!(diag.column >= 1);
}

#[test]
fn test_golden_invalid_escape() {
    let (diagnostics, _) = lex_file("invalid_escape.atl");

    // Should report AT1003: Invalid escape sequence
    assert_has_error_code(&diagnostics, "AT1003");

    // Verify the escape character is mentioned
    let has_escape_error = diagnostics
        .iter()
        .any(|d| d.code == "AT1003" && d.message.contains("\\"));
    assert!(has_escape_error, "Expected error message to mention escape sequence");
}

#[test]
fn test_golden_unexpected_char() {
    let (diagnostics, _) = lex_file("unexpected_char.atl");

    // Should report AT1001: Unexpected character (multiple times)
    assert!(diagnostics.len() >= 3, "Expected at least 3 errors for @, $, #");

    // All should be AT1001 errors
    for diag in &diagnostics {
        assert_eq!(diag.code, "AT1001");
    }
}

#[test]
fn test_golden_unterminated_comment() {
    let (diagnostics, _) = lex_file("unterminated_comment.atl");

    // Should report AT1004: Unterminated multi-line comment
    assert_has_error_code(&diagnostics, "AT1004");

    let diag = &diagnostics[0];
    assert!(diag.message.contains("Unterminated multi-line comment"));
}

#[test]
fn test_golden_single_ampersand() {
    let (diagnostics, _) = lex_file("single_ampersand.atl");

    // Should report AT1001: Unexpected character '&'
    assert_has_error_code(&diagnostics, "AT1001");

    let has_ampersand_error = diagnostics
        .iter()
        .any(|d| d.code == "AT1001" && d.message.contains("&"));
    assert!(has_ampersand_error);
}

#[test]
fn test_golden_single_pipe() {
    let (diagnostics, _) = lex_file("single_pipe.atl");

    // Should report AT1001: Unexpected character '|'
    assert_has_error_code(&diagnostics, "AT1001");

    let has_pipe_error = diagnostics
        .iter()
        .any(|d| d.code == "AT1001" && d.message.contains("|"));
    assert!(has_pipe_error);
}

#[test]
fn test_all_golden_files_produce_errors() {
    // Verify that all golden test files produce at least one diagnostic
    let test_files = [
        "unterminated_string.atl",
        "invalid_escape.atl",
        "unexpected_char.atl",
        "unterminated_comment.atl",
        "single_ampersand.atl",
        "single_pipe.atl",
    ];

    for filename in &test_files {
        let (diagnostics, _) = lex_file(filename);
        assert!(
            !diagnostics.is_empty(),
            "Expected errors in {} but got none",
            filename
        );
    }
}

#[test]
fn test_diagnostic_stability() {
    // Verify that running the same file twice produces identical diagnostics
    let (diag1, _) = lex_file("unterminated_string.atl");
    let (diag2, _) = lex_file("unterminated_string.atl");

    assert_eq!(diag1.len(), diag2.len(), "Diagnostic count should be stable");
    for (d1, d2) in diag1.iter().zip(diag2.iter()) {
        assert_eq!(d1.code, d2.code, "Diagnostic codes should be stable");
        assert_eq!(d1.message, d2.message, "Diagnostic messages should be stable");
        assert_eq!(d1.line, d2.line, "Diagnostic lines should be stable");
        assert_eq!(d1.column, d2.column, "Diagnostic columns should be stable");
    }
}
