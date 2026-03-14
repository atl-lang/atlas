use super::*;

// ============================================================================
// Expression Errors
// ============================================================================

#[rstest]
#[case("1 +", "expression")]
#[case("1 + + 2", "expression")]
#[case("let x = (1 + 2;", "closing")]
#[case("let x = [1, 2, 3;", "closing")]
#[case("[]arr;", "statement")]
#[case("arr[0;", "closing")]
#[case("foo(1, 2, 3;", "closing")]
fn test_expression_errors(#[case] source: &str, #[case] expected: &str) {
    let diagnostics = parse_errors(source);
    assert_has_parser_error(&diagnostics, expected);
}

// ============================================================================
// Block Errors
// ============================================================================

#[rstest]
#[case("{ let x = 1", "}")]
#[case("fn foo(): number { return 1", "}")]
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
    let diagnostics = parse_errors("fn foo(): number { return 1;");
    assert_eq!(diagnostics.len(), 1, "Expected exactly one diagnostic");
    let diag = &diagnostics[0];
    let help = diag.help.first().map(|s| s.as_str()).unwrap_or("");
    let msg = diag.message.as_str();
    // Either the help text or the message should reference `}` or closing
    assert!(
        help.contains("close")
            || help.contains("`}`")
            || msg.contains("`}`")
            || msg.contains("'}'"),
        "Expected context-specific brace-closing message or help, got message: {:?}, help: {:?}",
        msg,
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
    let help = diagnostics[0]
        .help
        .first()
        .map(|s| s.as_str())
        .unwrap_or("");
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
// is_secondary Field Tests (B14-P04 — D-043)
// ============================================================================

/// Diagnostics created via the normal error path are NOT secondary by default.
#[test]
fn test_diagnostic_is_not_secondary_by_default() {
    use atlas_runtime::{Lexer, Parser};
    let mut lexer = Lexer::new("let x = ;");
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (_, diags) = parser.parse();
    assert!(!diags.is_empty());
    assert!(
        !diags[0].is_secondary,
        "Primary parser error should have is_secondary = false"
    );
}

/// Diagnostics marked via `.as_secondary()` have is_secondary = true
/// and display with `note:` prefix (not `error:`).
#[test]
fn test_as_secondary_builder_sets_flag() {
    use atlas_runtime::{Diagnostic, Span};
    let span = Span::new(0, 1);
    let diag = Diagnostic::error_with_code("AT1000", "test error", span).as_secondary();
    assert!(
        diag.is_secondary,
        "as_secondary() should set is_secondary = true"
    );
    let rendered = diag.to_human_string();
    assert!(
        rendered.starts_with("note["),
        "Secondary diagnostic should render with note: prefix, got: {:?}",
        rendered
    );
    assert!(
        rendered.contains("secondary"),
        "Secondary diagnostic header should mention 'secondary', got: {:?}",
        rendered
    );
}

/// Primary diagnostics still render with the level prefix (error/warning).
#[test]
fn test_primary_diagnostic_renders_with_level_prefix() {
    use atlas_runtime::{Diagnostic, Span};
    let span = Span::new(0, 1);
    let diag = Diagnostic::error_with_code("AT1000", "test error", span);
    let rendered = diag.to_human_string();
    assert!(
        rendered.starts_with("error["),
        "Primary error should render with 'error[' prefix, got: {:?}",
        rendered
    );
}

// ============================================================================
// H-224: Type[] postfix array syntax is canonical
// ============================================================================

/// `Type[]` in type position is valid Atlas syntax (H-224 revert from []Type back to Type[]).
#[test]
fn test_h224_postfix_array_syntax_parses_successfully() {
    let source = "fn foo(x: Person[]): number { return 0; }";
    let diagnostics = parse_errors(source);
    assert!(
        diagnostics.is_empty(),
        "Person[] should parse as a valid array type annotation, got errors: {:?}",
        diagnostics
    );
}

/// Writing old prefix `[]Type` emits a clear error pointing to the correct postfix form.
#[test]
fn test_h224_prefix_array_syntax_emits_helpful_error() {
    let source = "fn foo(x: []Person): number { return 0; }";
    let diagnostics = parse_errors(source);
    assert!(
        !diagnostics.is_empty(),
        "[]Person in type position should error"
    );
    let diag = &diagnostics[0];
    // Error message names both old and new form
    assert!(
        diag.message.contains("Person[]") && diag.message.contains("[]Person"),
        "error should show both forms, got: {:?}",
        diag.message
    );
    // Help text shows the fix
    let help = diag.help.join(" ");
    assert!(
        help.contains("Person[]"),
        "help should suggest Person[], got: {:?}",
        help
    );
    // Note explains the rule
    assert!(
        !diag.notes.is_empty() && diag.notes[0].contains("TypeScript"),
        "note should explain TypeScript-style postfix rule, got: {:?}",
        diag.notes
    );
}

// ============================================================================
// Operator Precedence Tests (from operator_precedence_tests.rs)
// ============================================================================
