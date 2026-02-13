//! Modern Parser Error Tests
//!
//! Converted from parser_error_tests.rs (354 lines → ~120 lines = 66% reduction)

mod common;

use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use rstest::rstest;

fn parse_source(source: &str) -> Vec<atlas_runtime::diagnostic::Diagnostic> {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (_program, diagnostics) = parser.parse();
    diagnostics
}

fn assert_has_parser_error(
    diagnostics: &[atlas_runtime::diagnostic::Diagnostic],
    expected_substring: &str,
) {
    assert!(!diagnostics.is_empty(), "Expected at least one diagnostic");
    let expected_lower = expected_substring.to_lowercase();
    let found = diagnostics
        .iter()
        .any(|d| d.message.to_lowercase().contains(&expected_lower) && d.code == "AT1000");
    assert!(
        found,
        "Expected AT1000 error with '{}', got: {:?}",
        expected_substring,
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

// ============================================================================
// Missing Semicolons
// ============================================================================

#[rstest]
#[case("let x = 42", "';'")]
#[case("foo()", "';'")]
#[case("return 42", "';'")]
#[case("break", "';'")]
#[case("continue", "';'")]
fn test_missing_semicolons(#[case] source: &str, #[case] expected: &str) {
    let diagnostics = parse_source(source);
    assert_has_parser_error(&diagnostics, expected);
}

// ============================================================================
// Variable Declaration Errors
// ============================================================================

#[rstest]
#[case("let = 42;", "variable name")]
#[case("let x;", "=")]
#[case("let x = ;", "expression")]
fn test_var_declaration_errors(#[case] source: &str, #[case] expected: &str) {
    let diagnostics = parse_source(source);
    assert_has_parser_error(&diagnostics, expected);
}

// ============================================================================
// Function Declaration Errors
// ============================================================================

#[rstest]
#[case("fn () { }", "function name")]
#[case("fn foo { }", "'('")]
#[case("fn foo()", "'{'")]
#[case("fn foo() { let x = 1;", "'}'")]
#[case("fn foo(x) { }", "':'")]
#[case("fn foo(: number) { }", "parameter name")]
fn test_function_declaration_errors(#[case] source: &str, #[case] expected: &str) {
    let diagnostics = parse_source(source);
    assert_has_parser_error(&diagnostics, expected);
}

// ============================================================================
// Nested Functions (Not Allowed)
// ============================================================================

#[rstest]
#[case("fn outer() { fn inner() { return 1; } }", "function")]
#[case("if (true) { fn foo() { return 1; } }", "function")]
#[case("while (true) { fn foo() { return 1; } }", "function")]
#[case(
    "for (let i = 0; i < 10; i = i + 1) { fn foo() { return 1; } }",
    "function"
)]
fn test_nested_functions_not_allowed(#[case] source: &str, #[case] expected: &str) {
    let diagnostics = parse_source(source);
    assert_has_parser_error(&diagnostics, expected);
}

// ============================================================================
// If Statement Errors
// ============================================================================

#[rstest]
#[case("if { }", "(")]
#[case("if (true { }", ")")]
#[case("if (true) }", "{")]
fn test_if_statement_errors(#[case] source: &str, #[case] expected: &str) {
    let diagnostics = parse_source(source);
    assert_has_parser_error(&diagnostics, expected);
}

// ============================================================================
// While Loop Errors
// ============================================================================

#[rstest]
#[case("while { }", "(")]
#[case("while (true { }", ")")]
#[case("while (true) }", "{")]
fn test_while_loop_errors(#[case] source: &str, #[case] expected: &str) {
    let diagnostics = parse_source(source);
    assert_has_parser_error(&diagnostics, expected);
}

// ============================================================================
// For Loop Errors
// ============================================================================

#[rstest]
#[case("for { }", "(")]
#[case("for (let i = 0 { }", ";")]
#[case("for (let i = 0; i < 10 { }", ";")]
#[case("for (let i = 0; i < 10; i++ { }", ")")]
#[case("for (let i = 0; i < 10; i++) }", "{")]
fn test_for_loop_errors(#[case] source: &str, #[case] expected: &str) {
    let diagnostics = parse_source(source);
    assert_has_parser_error(&diagnostics, expected);
}

// ============================================================================
// Expression Errors
// ============================================================================

#[rstest]
#[case("1 +", "expression")]
#[case("1 + + 2", "expression")]
#[case("let x = (1 + 2;", "')'")]
#[case("let x = [1, 2, 3;", "']'")]
#[case("arr[];", "expression")]
#[case("arr[0;", "']'")]
#[case("foo(1, 2, 3;", "')'")]
fn test_expression_errors(#[case] source: &str, #[case] expected: &str) {
    let diagnostics = parse_source(source);
    assert_has_parser_error(&diagnostics, expected);
}

// ============================================================================
// Block Errors
// ============================================================================

#[rstest]
#[case("{ let x = 1", "}")]
#[case("fn foo() -> number { return 1", "}")]
fn test_block_errors(#[case] source: &str, #[case] expected: &str) {
    let diagnostics = parse_source(source);
    assert_has_parser_error(&diagnostics, expected);
}

// ============================================================================
// Array Literal Errors
// ============================================================================

#[test]
fn test_array_literal_unclosed() {
    // Note: This might get consumed as expression start, so just check for error
    let diagnostics = parse_source("[1, 2");
    assert!(!diagnostics.is_empty(), "Expected error for unclosed array");
}

// ============================================================================
// Unary Operator Errors
// ============================================================================

#[rstest]
#[case("-", "expression")]
#[case("!", "expression")]
fn test_unary_errors(#[case] source: &str, #[case] expected: &str) {
    let diagnostics = parse_source(source);
    assert_has_parser_error(&diagnostics, expected);
}
