use super::*;

// ============================================================================
// Expression Errors
// ============================================================================

#[rstest]
#[case("1 +", "expression")]
#[case("1 + + 2", "expression")]
#[case("let x = (1 + 2;", "')'")]
#[case("let x = [1, 2, 3;", "']'")]
#[case("[]arr;", "expression")]
#[case("arr[0;", "']'")]
#[case("foo(1, 2, 3;", "')'")]
fn test_expression_errors(#[case] source: &str, #[case] expected: &str) {
    let diagnostics = parse_errors(source);
    assert_has_parser_error(&diagnostics, expected);
}

// ============================================================================
// Block Errors
// ============================================================================

#[rstest]
#[case("{ let x = 1", "}")]
#[case("fn foo() -> number { return 1", "}")]
fn test_block_errors(#[case] source: &str, #[case] expected: &str) {
    let diagnostics = parse_errors(source);
    assert_has_parser_error(&diagnostics, expected);
}

// ============================================================================
// Array Literal Errors
// ============================================================================

#[test]
fn test_array_literal_unclosed() {
    // Note: This might get consumed as expression start, so just check for error
    let diagnostics = parse_errors("[1, 2");
    assert!(!diagnostics.is_empty(), "Expected error for unclosed array");
}

// ============================================================================
// Unary Operator Errors
// ============================================================================

#[rstest]
#[case("-", "expression")]
#[case("!", "expression")]
fn test_unary_errors(#[case] source: &str, #[case] expected: &str) {
    let diagnostics = parse_errors(source);
    assert_has_parser_error(&diagnostics, expected);
}

// ============================================================================
// Cascade Suppression Tests (B14-P01 — D-043)
// ============================================================================

/// A single malformed expression should produce exactly 1 error (not a cascade).
#[test]
fn test_cascade_suppression_single_bad_expression() {
    // `let x = ;` — one bad expression yields exactly one diagnostic
    let diagnostics = parse_errors("let x = ;");
    assert_eq!(
        diagnostics.len(),
        1,
        "Expected exactly 1 diagnostic for one malformed expression, got {}: {:?}",
        diagnostics.len(),
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

/// A deliberately broken expression inside a function should not cascade into
/// unrelated diagnostics — 1 bug = 1 primary error within the same recovery region.
#[test]
fn test_cascade_suppression_broken_binary_expr() {
    // `1 +` — incomplete binary expression, should be one diagnostic
    let diagnostics = parse_errors("1 +");
    assert_eq!(
        diagnostics.len(),
        1,
        "Expected exactly 1 diagnostic for incomplete binary op, got {}: {:?}",
        diagnostics.len(),
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

/// Two independent errors on separate statements should each produce 1 diagnostic
/// (parser recovers between them, resetting panic mode).
#[test]
fn test_cascade_suppression_two_independent_errors() {
    // Two separate malformed statements — each should produce exactly 1 diagnostic
    let diagnostics = parse_errors("let x = ; let y = ;");
    assert_eq!(
        diagnostics.len(),
        2,
        "Expected 2 diagnostics (one per broken statement), got {}: {:?}",
        diagnostics.len(),
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

// ============================================================================
// Context-Aware Help Text Tests (B14-P03 — D-043)
// ============================================================================

/// Missing `}` should produce help text about closing a block/struct literal,
/// NOT the old registry string about string escapes (the AT1003 code clash bug).
#[test]
fn test_help_text_missing_brace_is_context_specific() {
    let diagnostics = parse_errors("fn foo() -> number { return 1;");
    assert_eq!(diagnostics.len(), 1, "Expected exactly one diagnostic");
    let help = diagnostics[0].help.as_deref().unwrap_or("");
    assert!(
        help.contains("close") || help.contains("`}`"),
        "Expected context-specific brace-closing help, got: {:?}",
        help
    );
    // Verify the old wrong registry help (string escapes) is NOT attached
    assert!(
        !help.contains("escape") && !help.contains("\\n"),
        "Got incorrect string-escape help on a missing-brace error: {:?}",
        help
    );
}

/// Missing semicolon help should say to add `;`, not a generic message.
#[test]
fn test_help_text_missing_semi_is_context_specific() {
    let diagnostics = parse_errors("let x = 1 let y = 2;");
    assert!(!diagnostics.is_empty(), "Expected at least one diagnostic");
    let help = diagnostics[0].help.as_deref().unwrap_or("");
    assert!(
        help.contains(";") || help.contains("semicolon") || help.contains("statement"),
        "Expected context-specific semicolon help, got: {:?}",
        help
    );
    // Verify old wrong registry help (block comment) is NOT attached
    assert!(
        !help.contains("*/") && !help.contains("comment"),
        "Got incorrect block-comment help on a missing-semicolon error: {:?}",
        help
    );
}

// ============================================================================
// Operator Precedence Tests (from operator_precedence_tests.rs)
// ============================================================================
