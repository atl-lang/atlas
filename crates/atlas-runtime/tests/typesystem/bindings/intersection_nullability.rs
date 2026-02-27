use super::super::*;
#[allow(unused_imports)]
use pretty_assertions::assert_eq;

// From intersection_type_tests.rs
// ============================================================================

// Tests for intersection types (Phase typing-04)

// ============================================================================
// Intersection construction tests
// ============================================================================

#[rstest]
#[case("let _x: number & number = 1;")]
#[case("let _x: string & string = \"ok\";")]
#[case("let _x: bool & bool = true;")]
#[case("let _x: number[] & number[] = [1, 2];")]
#[case("type Same = number & number; let _x: Same = 1;")]
#[case("fn f(x: number) -> number { return x; } let _x: ((number) -> number) & ((number) -> number) = f;")]
#[case("let _x: (number | string) & number = 1;")]
#[case("let _x: (number | string) & number = 2;")]
#[case("let _x: (number | string) & string = \"hi\";")]
#[case("let _x: (number | string | bool) & bool = true;")]
#[case("let _x: (number & number)[] = [1];")]
#[case("type Id<T> = T & T; let _x: Id<number> = 1;")]
#[case("let _x: (number | string) & (number | string) = \"ok\";")]
#[case("let _x: (number | string) & (number | string) = 2;")]
fn test_intersection_construction(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// ============================================================================
// Intersection error tests
// ============================================================================

#[rstest]
#[case("let _x: number & string = 1;")]
#[case("let _x: number & string = \"ok\";")]
#[case("let _x: bool & number = true;")]
#[case("let _x: string & null = \"ok\";")]
#[case("let _x: null & string = null;")]
#[case("let _x: (number | string) & number = \"bad\";")]
#[case("let _x: (number | string) & string = 1;")]
#[case("let _x: (bool | string) & number = 1;")]
#[case(
    "fn f(x: number) -> number { return x; } let _x: (number) -> number & (string) -> string = f;"
)]
#[case("let _x: number & string & bool = 1;")]
#[case("type Id<T> = T & string; let _x: Id<number> = 1;")]
#[case("let _x: (number | string) & (bool | string) = 1;")]
#[case("let _x: (number | string) & (bool | string) = true;")]
#[case("let _x: (number | string) & (bool | string) = null;")]
fn test_intersection_errors(#[case] source: &str) {
    let diags = errors(source);
    assert!(!diags.is_empty(), "Expected errors, got none");
}

// ============================================================================
// Union/intersection interaction tests
// ============================================================================

#[rstest]
#[case("let _x: (number | string) & number = 1;")]
#[case("let _x: (number | string | bool) & number = 1;")]
#[case("let _x: (number | string) & string = \"ok\";")]
#[case("let _x: (number | string | bool) & bool = true;")]
#[case("let _x: (number | string | bool) & string = \"ok\";")]
#[case("let _x: (number | string | bool) & number = 2;")]
#[case("let _x: (number | string) & (bool | string) = \"ok\";")]
#[case("let _x: (number | string) & (bool | string) = \"yes\";")]
fn test_intersection_distribution(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

#[rstest]
#[case("let _x: (number | string) & number = \"bad\";")]
#[case("let _x: (number | string | bool) & string = 10;")]
#[case("let _x: (number | string | bool) & bool = \"no\";")]
#[case("let _x: (number | string | bool) & number = false;")]
fn test_intersection_distribution_errors(#[case] source: &str) {
    let diags = errors(source);
    assert!(!diags.is_empty(), "Expected errors, got none");
}

// ============================================================================
// Intersection + method/index operations
// ============================================================================

#[rstest]
#[case("let _x: number[] & number[] = [1, 2]; let _y: number = _x[0];")]
#[case("let _x: number[] & number[] = [1, 2]; let _y: number = _x[1];")]
#[case("let _x: number[] & number[] = [1, 2]; let _y: number = _x[0] + _x[1];")]
fn test_intersection_operations(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// ============================================================================

// From nullability_tests.rs
// ============================================================================

// Comprehensive tests for nullability rules
//
// Tests cover:
// - null is only assignable to null type (no implicit nullable)
// - null cannot be assigned to number, string, bool, void, or arrays
// - Explicit null type variables
// - null in expressions and operations
// - null in function parameters and returns
// - null comparisons

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

// ============================================================================

// From type_alias_tests.rs
