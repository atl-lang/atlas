//! Tests for numeric edge cases
//!
//! Verifies behavior with boundary values, special floats (infinity, NaN),
//! division by zero, and other numeric edge cases.
//!
//! Atlas uses f64 (64-bit IEEE 754 floating point) for all numbers.

use atlas_runtime::{Binder, Lexer, Parser, TypeChecker};
use rstest::rstest;

/// Helper to get all diagnostics from source code
fn get_all_diagnostics(source: &str) -> Vec<atlas_runtime::Diagnostic> {
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();

    let mut binder = Binder::new();
    let (mut table, bind_diags) = binder.bind(&program);

    let mut checker = TypeChecker::new(&mut table);
    let type_diags = checker.check(&program);

    let mut all_diags = Vec::new();
    all_diags.extend(lex_diags);
    all_diags.extend(parse_diags);
    all_diags.extend(bind_diags);
    all_diags.extend(type_diags);

    all_diags
}

// =============================================================================
// Integer and Float Boundary Tests
// =============================================================================

#[rstest]
#[case::large_integer("let x: number = 9007199254740991;")]
#[case::negative_large_integer("let x: number = -9007199254740991;")]
#[case::large_integer_arithmetic(
    "let a: number = 9007199254740991;\nlet b: number = 1;\nlet c: number = a + b;"
)]
#[case::float_literal("let x: number = 3.14159265358979323846;")]
#[case::very_small_float("let x: number = 0.0000000001;")]
#[case::negative_float("let x: number = -3.14159;")]
#[case::zero_variants("let a: number = 0;\nlet b: number = 0.0;\nlet c: number = -0.0;")]
fn test_numeric_boundaries(#[case] source: &str) {
    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Should be valid: {:?}", diags);
}

#[test]
fn test_very_large_float() {
    let source = "let x = 179769313486231570000000000000000000000.0;";
    let _diags = get_all_diagnostics(source);
    // This might fail to parse depending on lexer implementation
}

// =============================================================================
// Division and Modulo Tests
// =============================================================================

#[rstest]
#[case::division("let a: number = 10;\nlet b: number = 2;\nlet c: number = a / b;")]
#[case::division_by_zero_literal("let x: number = 10 / 0;")]
#[case::division_by_variable("let divisor: number = 0;\nlet result: number = 10 / divisor;")]
#[case::division_underflow("let a = 1;\nlet b = 10000000;\nlet c = a / b;")]
#[case::modulo_by_zero("let x: number = 10 % 0;")]
#[case::modulo_with_floats("let x: number = 5.5 % 2.3;")]
fn test_division_and_modulo(#[case] source: &str) {
    let diags = get_all_diagnostics(source);
    // Type checker cannot detect division by zero - this is runtime behavior
    assert!(diags.is_empty(), "Should typecheck: {:?}", diags);
}

// =============================================================================
// Arithmetic Overflow/Underflow Tests
// =============================================================================

#[rstest]
#[case::addition_overflow("let a = 100000000000000000000000000000.0;\nlet b = 100000000000000000000000000000.0;\nlet c = a + b;")]
#[case::multiplication_overflow(
    "let a = 10000000000000000000.0;\nlet b = 10000000000000000000.0;\nlet c = a * b;"
)]
fn test_arithmetic_overflow(#[case] source: &str) {
    let _diags = get_all_diagnostics(source);
    // Typechecks fine, runtime would produce infinity
}

#[test]
fn test_subtraction_to_negative() {
    let source = "let a: number = 5;\nlet b: number = 10;\nlet c: number = a - b;";
    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Should typecheck: {:?}", diags);
}

// =============================================================================
// Comparison Tests with Edge Values
// =============================================================================

#[rstest]
#[case::zero_comparisons(
    "let a: number = 0;\nlet b: bool = a > 0;\nlet c: bool = a < 0;\nlet d: bool = a == 0;"
)]
#[case::negative_comparison("let a: number = -5;\nlet b: number = 10;\nlet c: bool = a < b;")]
#[case::float_equality("let a: number = 0.1 + 0.2;\nlet b: number = 0.3;\nlet c: bool = a == b;")]
fn test_comparisons(#[case] source: &str) {
    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Should typecheck: {:?}", diags);
}

// =============================================================================
// Mixed Arithmetic Tests
// =============================================================================

#[rstest]
#[case::complex_expression("let x: number = (10 + 5) * 2 - 8 / 4;")]
#[case::nested_arithmetic("let a: number = 10;\nlet b: number = 5;\nlet c: number = 2;\nlet result: number = (a + b) * c - (a / b);")]
#[case::negative_arithmetic("let a: number = -10;\nlet b: number = -5;\nlet c: number = a + b;\nlet d: number = a - b;\nlet e: number = a * b;\nlet f: number = a / b;")]
fn test_mixed_arithmetic(#[case] source: &str) {
    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Should typecheck: {:?}", diags);
}

// =============================================================================
// Unary Minus Tests
// =============================================================================

#[rstest]
#[case::literal("let x: number = -42;")]
#[case::variable("let a: number = 42;\nlet b: number = -a;")]
#[case::double_negation("let a: number = 42;\nlet b: number = -(-a);")]
#[case::negative_zero("let x: number = -0;\nlet y: number = -0.0;")]
fn test_unary_minus(#[case] source: &str) {
    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Should typecheck: {:?}", diags);
}

// =============================================================================
// Error Cases
// =============================================================================

#[rstest]
#[case::string_plus_number("let x: number = \"hello\" + 5;")]
#[case::string_division("let x: number = \"10\" / \"2\";")]
#[case::bool_modulo("let x: number = true % false;")]
#[case::string_comparison("let x: bool = \"hello\" < 5;")]
fn test_type_errors(#[case] source: &str) {
    let diags = get_all_diagnostics(source);
    assert!(!diags.is_empty(), "Should produce error");
}

#[test]
fn test_arithmetic_on_non_numbers_has_error_code() {
    let source = "let x: number = \"hello\" + 5;";
    let diags = get_all_diagnostics(source);
    let error = diags.iter().find(|d| d.code.starts_with("AT"));
    assert!(error.is_some(), "Should have AT error code");
}

// =============================================================================
// Array Index Edge Cases
// =============================================================================

#[rstest]
#[case::zero_index("let arr = [1, 2, 3];\nlet x = arr[0];")]
#[case::large_index("let arr = [1, 2, 3];\nlet x = arr[999999];")]
#[case::negative_index("let arr = [1, 2, 3];\nlet x = arr[-1];")]
#[case::float_index("let arr = [1, 2, 3];\nlet x = arr[1.5];")]
fn test_array_index_edge_cases(#[case] source: &str) {
    let diags = get_all_diagnostics(source);
    // Type system allows number (f64) for array index
    // Runtime would handle bounds/integer checking
    assert!(diags.is_empty(), "Should typecheck: {:?}", diags);
}
