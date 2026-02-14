//! Comprehensive tests for nullability rules
//!
//! Tests cover:
//! - null is only assignable to null type (no implicit nullable)
//! - null cannot be assigned to number, string, bool, void, or arrays
//! - Explicit null type variables
//! - null in expressions and operations
//! - null in function parameters and returns
//! - null comparisons

use atlas_runtime::binder::Binder;
use atlas_runtime::diagnostic::{Diagnostic, DiagnosticLevel};
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::typechecker::TypeChecker;
use rstest::rstest;

fn typecheck_source(source: &str) -> Vec<Diagnostic> {
    let mut lexer = Lexer::new(source.to_string());
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

fn assert_no_errors(diagnostics: &[Diagnostic]) {
    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "Expected no errors, got: {:?}",
        errors.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

fn assert_has_error(diagnostics: &[Diagnostic], code: &str) {
    assert!(
        !diagnostics.is_empty(),
        "Expected at least one diagnostic with code {}",
        code
    );
    let found = diagnostics.iter().any(|d| d.code == code);
    assert!(
        found,
        "Expected diagnostic with code {}, got: {:?}",
        code,
        diagnostics.iter().map(|d| &d.code).collect::<Vec<_>>()
    );
}

// ========== Valid Null Usage ==========

#[rstest]
#[case::literal_inference("let x = null;")]
#[case::variable_inference("let x = null;\nlet y = x;")]
#[case::equality_with_null("let x = null == null;")]
#[case::inequality_with_null("let x = null != null;")]
#[case::null_array_literal("let x = [null, null, null];")]
#[case::single_null_array("let x = [null];")]
#[case::nested_null_expression("let x = (null == null) && true;")]
#[case::null_variable_comparison("let x = null;\nlet y = null;\nlet z = x == y;")]
#[case::null_value_usage("let x = null;\nlet y = x == null;")]
#[case::null_comparison_chain(
    "let a = null;\nlet b = null;\nlet result = (a == b) && (b == null);"
)]
fn test_valid_null_usage(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_no_errors(&diagnostics);
}

// ========== Null Assignment Errors ==========

#[rstest]
#[case::to_number("let x: number = null;")]
#[case::to_string(r#"let x: string = null;"#)]
#[case::to_bool("let x: bool = null;")]
#[case::in_number_array("let x = [1, 2, null];")]
#[case::in_string_array(r#"let x = ["a", "b", null];"#)]
fn test_null_assignment_errors(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3001");
}

// ========== Null Function Parameter Errors ==========

#[rstest]
#[case::number_param(
    "fn acceptsNumber(x: number) -> number { return x; }\nlet result = acceptsNumber(null);"
)]
#[case::string_param(
    "fn acceptsString(x: string) -> string { return x; }\nlet result = acceptsString(null);"
)]
#[case::bool_param(
    "fn acceptsBool(x: bool) -> bool { return x; }\nlet result = acceptsBool(null);"
)]
fn test_null_function_parameter_errors(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3001");
}

// ========== Null Function Return Errors ==========

#[rstest]
#[case::number_return("fn returnsNumber() -> number { return null; }")]
#[case::string_return("fn returnsString() -> string { return null; }")]
#[case::bool_return("fn returnsBool() -> bool { return null; }")]
fn test_null_function_return_errors(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3001");
}

// ========== Null Comparison Errors ==========

#[rstest]
#[case::with_number("let x = null == 42;")]
#[case::with_string(r#"let x = null == "hello";"#)]
#[case::with_bool("let x = null == true;")]
#[case::number_with_null("let x = 42 == null;")]
fn test_null_comparison_errors(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3002");
}

// ========== Null Arithmetic Errors ==========

#[rstest]
#[case::addition("let x = null + null;")]
#[case::null_plus_number("let x = null + 42;")]
#[case::number_plus_null("let x = 42 + null;")]
#[case::subtraction("let x = null - null;")]
#[case::multiplication("let x = null * null;")]
#[case::division("let x = null / null;")]
fn test_null_arithmetic_errors(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3002");
}

// ========== Null Logical Operation Errors ==========

#[rstest]
#[case::and_operator("let x = null && null;")]
#[case::or_operator("let x = null || null;")]
#[case::null_and_bool("let x = null && true;")]
#[case::bool_and_null("let x = true && null;")]
fn test_null_logical_errors(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3002");
}

// ========== Null in Conditionals ==========

#[rstest]
#[case::if_condition("if (null) { let x: number = 1; }")]
#[case::while_condition("while (null) { break; }")]
#[case::for_condition("for (let i: number = 0; null; i = i + 1) { break; }")]
fn test_null_in_conditionals(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3001");
}

// ========== Null with Unary Operators ==========

#[rstest]
#[case::negate("let x = -null;")]
#[case::not("let x = !null;")]
fn test_null_unary_errors(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3002");
}

// ========== Null in Arrays ==========

#[rstest]
#[case::null_then_number("let x = [null, 42];")]
#[case::number_then_null("let x = [42, null];")]
fn test_mixed_null_array_errors(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3001");
}

// ========== Edge Cases ==========

#[test]
fn test_null_in_array_indexing_error() {
    let diagnostics = typecheck_source("let arr = [1, 2, 3];\nlet x = arr[null];");
    assert_has_error(&diagnostics, "AT3001");
}
