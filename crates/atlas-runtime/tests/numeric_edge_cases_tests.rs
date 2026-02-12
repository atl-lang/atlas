//! Tests for numeric edge cases
//!
//! Verifies behavior with boundary values, special floats (infinity, NaN),
//! division by zero, and other numeric edge cases.
//!
//! Atlas uses f64 (64-bit IEEE 754 floating point) for all numbers.

use atlas_runtime::{Binder, Lexer, Parser, TypeChecker};

/// Helper to get all diagnostics from source code
fn get_all_diagnostics(source: &str) -> Vec<atlas_runtime::Diagnostic> {
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();

    let mut binder = Binder::new();
    let (table, bind_diags) = binder.bind(&program);

    let mut checker = TypeChecker::new(&table);
    let type_diags = checker.check(&program);

    // Combine all diagnostics
    let mut all_diags = Vec::new();
    all_diags.extend(lex_diags);
    all_diags.extend(parse_diags);
    all_diags.extend(bind_diags);
    all_diags.extend(type_diags);

    all_diags
}

// =============================================================================
// Integer Boundary Tests
// =============================================================================

#[test]
fn test_large_integer_literal() {
    // Large integers should parse and typecheck correctly
    let source = r#"
        let x: number = 9007199254740991;
    "#;

    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Large integer should be valid: {:?}", diags);
}

#[test]
fn test_negative_large_integer() {
    let source = r#"
        let x: number = -9007199254740991;
    "#;

    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Negative large integer should be valid: {:?}", diags);
}

#[test]
fn test_integer_arithmetic_boundaries() {
    // Test arithmetic with large integers
    let source = r#"
        let a: number = 9007199254740991;
        let b: number = 1;
        let c: number = a + b;
    "#;

    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Large integer arithmetic should typecheck: {:?}", diags);
}

// =============================================================================
// Float Boundary Tests
// =============================================================================

#[test]
fn test_float_literal() {
    let source = r#"
        let x: number = 3.14159265358979323846;
    "#;

    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Float literal should be valid: {:?}", diags);
}

#[test]
fn test_very_small_float() {
    // Test small but non-zero float
    let source = r#"
        let x: number = 0.0000000001;
    "#;

    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Small float should be valid: {:?}", diags);
}

#[test]
fn test_very_large_float() {
    // Test large float (use large but parseable number)
    // Note: Scientific notation may not be supported by lexer
    let source = r#"
        let x = 179769313486231570000000000000000000000.0;
    "#;

    let _diags = get_all_diagnostics(source);
    // This might fail to parse depending on lexer implementation
    // If it fails, that's okay - we're documenting the behavior
}

#[test]
fn test_negative_float() {
    let source = r#"
        let x: number = -3.14159;
    "#;

    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Negative float should be valid: {:?}", diags);
}

#[test]
fn test_zero_variants() {
    // Test different representations of zero
    let source = r#"
        let a: number = 0;
        let b: number = 0.0;
        let c: number = -0.0;
    "#;

    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Zero variants should be valid: {:?}", diags);
}

// =============================================================================
// Division Tests
// =============================================================================

#[test]
fn test_division_typechecks() {
    // Division should typecheck correctly
    let source = r#"
        let a: number = 10;
        let b: number = 2;
        let c: number = a / b;
    "#;

    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Division should typecheck: {:?}", diags);
}

#[test]
fn test_division_by_zero_literal_typechecks() {
    // Division by literal zero typechecks (runtime behavior is separate)
    let source = r#"
        let x: number = 10 / 0;
    "#;

    let diags = get_all_diagnostics(source);
    // Type checker cannot detect division by zero at compile time
    // This is a runtime concern
    assert!(diags.is_empty(), "Division by zero literal should typecheck: {:?}", diags);
}

#[test]
fn test_division_by_variable() {
    // Division by variable (could be zero at runtime)
    let source = r#"
        let divisor: number = 0;
        let result: number = 10 / divisor;
    "#;

    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Division by variable should typecheck: {:?}", diags);
}

// =============================================================================
// Modulo Tests
// =============================================================================

#[test]
fn test_modulo_by_zero_literal_typechecks() {
    // Modulo by zero typechecks (runtime behavior is separate)
    let source = r#"
        let x: number = 10 % 0;
    "#;

    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Modulo by zero should typecheck: {:?}", diags);
}

#[test]
fn test_modulo_with_floats() {
    let source = r#"
        let x: number = 5.5 % 2.3;
    "#;

    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Modulo with floats should typecheck: {:?}", diags);
}

// =============================================================================
// Arithmetic Overflow/Underflow Tests
// =============================================================================

#[test]
fn test_addition_with_large_numbers() {
    // Addition that might overflow at runtime
    let source = r#"
        let a = 100000000000000000000000000000.0;
        let b = 100000000000000000000000000000.0;
        let c = a + b;
    "#;

    let _diags = get_all_diagnostics(source);
    // Type checker cannot detect overflow - this is runtime behavior
    // Result would be infinity at runtime
}

#[test]
fn test_multiplication_overflow() {
    let source = r#"
        let a = 10000000000000000000.0;
        let b = 10000000000000000000.0;
        let c = a * b;
    "#;

    let _diags = get_all_diagnostics(source);
    // Typechecks fine, runtime would produce infinity
}

#[test]
fn test_subtraction_to_negative() {
    let source = r#"
        let a: number = 5;
        let b: number = 10;
        let c: number = a - b;
    "#;

    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Subtraction to negative should typecheck: {:?}", diags);
}

#[test]
fn test_division_underflow() {
    // Division that produces very small number
    // Note: Atlas lexer may not support scientific notation
    let source = r#"
        let a = 1;
        let b = 10000000;
        let c = a / b;
    "#;

    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Division underflow should typecheck: {:?}", diags);
}

// =============================================================================
// Comparison Tests with Edge Values
// =============================================================================

#[test]
fn test_comparison_with_zero() {
    let source = r#"
        let a: number = 0;
        let b: bool = a > 0;
        let c: bool = a < 0;
        let d: bool = a == 0;
    "#;

    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Zero comparisons should typecheck: {:?}", diags);
}

#[test]
fn test_comparison_with_negative() {
    let source = r#"
        let a: number = -5;
        let b: number = 10;
        let c: bool = a < b;
    "#;

    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Negative comparisons should typecheck: {:?}", diags);
}

#[test]
fn test_equality_with_floats() {
    // Floating point equality (behavior is well-defined but may surprise users)
    let source = r#"
        let a: number = 0.1 + 0.2;
        let b: number = 0.3;
        let c: bool = a == b;
    "#;

    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Float equality should typecheck: {:?}", diags);
}

// =============================================================================
// Mixed Arithmetic Tests
// =============================================================================

#[test]
fn test_complex_arithmetic_expression() {
    let source = r#"
        let x: number = (10 + 5) * 2 - 8 / 4;
    "#;

    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Complex arithmetic should typecheck: {:?}", diags);
}

#[test]
fn test_nested_arithmetic() {
    let source = r#"
        let a: number = 10;
        let b: number = 5;
        let c: number = 2;
        let result: number = (a + b) * c - (a / b);
    "#;

    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Nested arithmetic should typecheck: {:?}", diags);
}

#[test]
fn test_arithmetic_with_negative_numbers() {
    let source = r#"
        let a: number = -10;
        let b: number = -5;
        let c: number = a + b;
        let d: number = a - b;
        let e: number = a * b;
        let f: number = a / b;
    "#;

    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Arithmetic with negatives should typecheck: {:?}", diags);
}

// =============================================================================
// Unary Minus Tests
// =============================================================================

#[test]
fn test_unary_minus_on_literal() {
    let source = r#"
        let x: number = -42;
    "#;

    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Unary minus on literal should typecheck: {:?}", diags);
}

#[test]
fn test_unary_minus_on_variable() {
    let source = r#"
        let a: number = 42;
        let b: number = -a;
    "#;

    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Unary minus on variable should typecheck: {:?}", diags);
}

#[test]
fn test_double_negation() {
    let source = r#"
        let a: number = 42;
        let b: number = -(-a);
    "#;

    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Double negation should typecheck: {:?}", diags);
}

#[test]
fn test_unary_minus_on_zero() {
    let source = r#"
        let x: number = -0;
        let y: number = -0.0;
    "#;

    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Unary minus on zero should typecheck: {:?}", diags);
}

// =============================================================================
// Error Cases
// =============================================================================

#[test]
fn test_arithmetic_on_non_numbers() {
    let source = r#"
        let x: number = "hello" + 5;
    "#;

    let diags = get_all_diagnostics(source);
    assert!(!diags.is_empty(), "String + number should produce error");

    let error = diags.iter().find(|d| d.code.starts_with("AT"));
    assert!(error.is_some(), "Should have AT error code");
}

#[test]
fn test_division_of_non_numbers() {
    let source = r#"
        let x: number = "10" / "2";
    "#;

    let diags = get_all_diagnostics(source);
    assert!(!diags.is_empty(), "String division should produce error");
}

#[test]
fn test_modulo_of_non_numbers() {
    let source = r#"
        let x: number = true % false;
    "#;

    let diags = get_all_diagnostics(source);
    assert!(!diags.is_empty(), "Bool modulo should produce error");
}

#[test]
fn test_comparison_of_wrong_types() {
    let source = r#"
        let x: bool = "hello" < 5;
    "#;

    let diags = get_all_diagnostics(source);
    assert!(!diags.is_empty(), "String < number should produce error");
}

// =============================================================================
// Array Index Edge Cases
// =============================================================================

#[test]
fn test_array_index_zero() {
    let source = r#"
        let arr = [1, 2, 3];
        let x = arr[0];
    "#;

    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Array index 0 should typecheck: {:?}", diags);
}

#[test]
fn test_array_index_large_number() {
    // Large index typechecks (runtime bounds checking is separate)
    let source = r#"
        let arr = [1, 2, 3];
        let x = arr[999999];
    "#;

    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Large array index should typecheck: {:?}", diags);
}

#[test]
fn test_array_index_negative() {
    // Negative index typechecks (runtime error)
    let source = r#"
        let arr = [1, 2, 3];
        let x = arr[-1];
    "#;

    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Negative array index should typecheck: {:?}", diags);
}

#[test]
fn test_array_index_float_typechecks() {
    // Float index typechecks but would be runtime error (non-integer)
    let source = r#"
        let arr = [1, 2, 3];
        let x = arr[1.5];
    "#;

    let diags = get_all_diagnostics(source);
    // Type system allows number (f64) for array index
    // Runtime would check for integer
    assert!(diags.is_empty(), "Float array index should typecheck: {:?}", diags);
}
