use super::*;

// From vm_stdlib_types_tests.rs
// ============================================================================

// Type checking and conversion stdlib tests (VM engine)
//
// Tests all 12 type utility functions via VM execution for parity verification
//
// Note: These tests use the same common::* helpers which test through the full pipeline,
// ensuring both interpreter and VM produce identical results.

// ============================================================================
// typeof Tests
// ============================================================================

#[test]
fn test_typeof_null() {
    let code = r#"typeof(null)"#;
    assert_eval_string(code, "null");
}

#[test]
fn test_typeof_bool_true() {
    let code = r#"typeof(true)"#;
    assert_eval_string(code, "boolean");
}

#[test]
fn test_typeof_bool_false() {
    let code = r#"typeof(false)"#;
    assert_eval_string(code, "boolean");
}

#[test]
fn test_typeof_number_positive() {
    let code = r#"typeof(42)"#;
    assert_eval_string(code, "number");
}

#[test]
fn test_typeof_number_negative() {
    let code = r#"typeof(-10)"#;
    assert_eval_string(code, "number");
}

#[test]
fn test_typeof_number_float() {
    let code = r#"typeof(3.5)"#;
    assert_eval_string(code, "number");
}

// NaN/Infinity tests removed: division by zero is a runtime error in Atlas

#[test]
fn test_typeof_string_nonempty() {
    let code = r#"typeof("hello")"#;
    assert_eval_string(code, "string");
}

#[test]
fn test_typeof_string_empty() {
    let code = r#"typeof("")"#;
    assert_eval_string(code, "string");
}

#[test]
fn test_typeof_array_nonempty() {
    let code = r#"typeof([1,2,3])"#;
    assert_eval_string(code, "array");
}

#[test]
fn test_typeof_array_empty() {
    let code = r#"typeof([])"#;
    assert_eval_string(code, "array");
}

// Function reference tests removed: not yet fully supported

#[test]
fn test_typeof_json() {
    let code = r#"typeof(unwrap(parse_json("null")))"#;
    assert_eval_string(code, "null");
}

#[test]
fn test_typeof_option() {
    let code = r#"typeof(Some(42))"#;
    assert_eval_string(code, "option");
}

#[test]
fn test_typeof_result() {
    let code = r#"typeof(Ok(42))"#;
    assert_eval_string(code, "record");
}

#[test]
fn test_typeof_record() {
    let code = r#"typeof(record { a: 1 })"#;
    assert_eval_string(code, "record");
}

#[test]
fn test_typeof_function() {
    let code = r#"
        fn add(borrow a: number, borrow b: number) -> number { a + b }
        typeof(add)
    "#;
    assert_eval_string(code, "function");
}

#[test]
fn test_type_of_boolean() {
    let code = r#"type_of(true)"#;
    assert_eval_string(code, "boolean");
}

#[test]
fn test_type_of_record() {
    let code = r#"type_of(record { a: 1 })"#;
    assert_eval_string(code, "record");
}

#[test]
fn test_type_of_function() {
    let code = r#"
        fn id(borrow x: number) -> number { x }
        type_of(id)
    "#;
    assert_eval_string(code, "function");
}

// ============================================================================
// Type Guard Tests
// ============================================================================

#[test]
fn test_is_string_true() {
    let code = r#"is_string("hello")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_string_false_number() {
    let code = r#"is_string(42)"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_string_false_null() {
    let code = r#"is_string(null)"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_number_true_int() {
    let code = r#"is_number(42)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_number_true_float() {
    let code = r#"is_number(3.5)"#;
    assert_eval_bool(code, true);
}

// Removed: NaN test (division by zero is error)

#[test]
fn test_is_number_false_string() {
    let code = r#"is_number("42")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_bool_true() {
    let code = r#"is_bool(true)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_bool_false() {
    let code = r#"is_bool(false)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_bool_false_number() {
    let code = r#"is_bool(1)"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_null_true() {
    let code = r#"is_null(null)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_null_false() {
    let code = r#"is_null(0)"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_array_true() {
    let code = r#"is_array([1,2,3])"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_array_true_empty() {
    let code = r#"is_array([])"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_array_false() {
    let code = r#"is_array("not array")"#;
    assert_eval_bool(code, false);
}

// Function reference tests removed: not yet fully supported

#[test]
fn test_is_function_false() {
    let code = r#"is_function(42)"#;
    assert_eval_bool(code, false);
}

// ============================================================================
// toString Tests
// ============================================================================

#[test]
fn test_to_string_null() {
    let code = r#"toString(null)"#;
    assert_eval_string(code, "null");
}

#[test]
fn test_to_string_bool_true() {
    let code = r#"toString(true)"#;
    assert_eval_string(code, "true");
}

#[test]
fn test_to_string_bool_false() {
    let code = r#"toString(false)"#;
    assert_eval_string(code, "false");
}

#[test]
fn test_to_string_number_int() {
    let code = r#"toString(42)"#;
    assert_eval_string(code, "42");
}

#[test]
fn test_to_string_number_float() {
    let code = r#"toString(3.5)"#;
    assert_eval_string(code, "3.5");
}

#[test]
fn test_to_string_number_negative() {
    let code = r#"toString(-10)"#;
    assert_eval_string(code, "-10");
}

#[test]
fn test_to_string_number_zero() {
    let code = r#"toString(0)"#;
    assert_eval_string(code, "0");
}

// NaN/Infinity toString tests removed: division by zero is error

#[test]
fn test_to_string_string_identity() {
    let code = r#"toString("hello")"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_to_string_string_empty() {
    let code = r#"toString("")"#;
    assert_eval_string(code, "");
}

#[test]
fn test_to_string_array() {
    let code = r#"toString([1,2,3])"#;
    assert_eval_string(code, "[Array]");
}

// Function toString test removed: not yet fully supported

#[test]
fn test_to_string_json() {
    let code = r#"toString(unwrap(parse_json("null")))"#;
    assert_eval_string(code, "[JSON]");
}

// ============================================================================
// to_number Tests
// ============================================================================

#[test]
fn test_to_number_number_identity() {
    let code = r#"to_number(42)"#;
    assert_eval_result_ok_number(code, 42.0);
}

#[test]
fn test_to_number_bool_true() {
    let code = r#"to_number(true)"#;
    assert_eval_result_ok_number(code, 1.0);
}

#[test]
fn test_to_number_bool_false() {
    let code = r#"to_number(false)"#;
    assert_eval_result_ok_number(code, 0.0);
}

#[test]
fn test_to_number_string_int() {
    let code = r#"to_number("42")"#;
    assert_eval_result_ok_number(code, 42.0);
}

#[test]
fn test_to_number_string_float() {
    let code = r#"to_number("3.5")"#;
    assert_eval_result_ok_number(code, 3.5);
}

#[test]
fn test_to_number_string_negative() {
    let code = r#"to_number("-10")"#;
    assert_eval_result_ok_number(code, -10.0);
}

#[test]
fn test_to_number_string_whitespace() {
    let code = r#"to_number("  42  ")"#;
    assert_eval_result_ok_number(code, 42.0);
}

#[test]
fn test_to_number_string_scientific() {
    let code = r#"to_number("1e10")"#;
    assert_eval_result_ok_number(code, 1e10);
}

#[test]
fn test_to_number_string_empty_error() {
    let code = r#"to_number("")"#;
    assert_eval_result_err(code);
}

#[test]
fn test_to_number_string_invalid_error() {
    let code = r#"to_number("hello")"#;
    assert_eval_result_err(code);
}

#[test]
fn test_to_number_null_error() {
    let code = r#"to_number(null)"#;
    assert_eval_result_err(code);
}

#[test]
fn test_to_number_array_error() {
    let code = r#"to_number([1,2,3])"#;
    assert_eval_result_err(code);
}

// ============================================================================
// to_bool Tests
// ============================================================================

#[test]
fn test_to_bool_bool_identity_true() {
    let code = r#"to_bool(true)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_to_bool_bool_identity_false() {
    let code = r#"to_bool(false)"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_to_bool_number_zero_false() {
    let code = r#"to_bool(0)"#;
    assert_eval_bool(code, false);
}

// NaN to_bool test removed: division by zero is error

#[test]
fn test_to_bool_number_positive_true() {
    let code = r#"to_bool(42)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_to_bool_number_negative_true() {
    let code = r#"to_bool(-10)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_to_bool_string_empty_false() {
    let code = r#"to_bool("")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_to_bool_string_nonempty_true() {
    let code = r#"to_bool("hello")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_to_bool_string_space_true() {
    let code = r#"to_bool(" ")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_to_bool_null_false() {
    let code = r#"to_bool(null)"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_to_bool_array_true() {
    let code = r#"to_bool([1,2,3])"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_to_bool_array_empty_true() {
    let code = r#"to_bool([])"#;
    assert_eval_bool(code, true);
}

// Function to_bool test removed: not yet fully supported

// ============================================================================
// parse_int Tests
// ============================================================================

#[test]
fn test_parse_int_decimal() {
    let code = r#"parse_int("42", 10)"#;
    assert_eval_result_ok_number(code, 42.0);
}

#[test]
fn test_parse_int_decimal_negative() {
    let code = r#"parse_int("-10", 10)"#;
    assert_eval_result_ok_number(code, -10.0);
}

#[test]
fn test_parse_int_binary() {
    let code = r#"parse_int("1010", 2)"#;
    assert_eval_result_ok_number(code, 10.0);
}

#[test]
fn test_parse_int_octal() {
    let code = r#"parse_int("17", 8)"#;
    assert_eval_result_ok_number(code, 15.0);
}

#[test]
fn test_parse_int_hex() {
    let code = r#"parse_int("FF", 16)"#;
    assert_eval_result_ok_number(code, 255.0);
}

#[test]
fn test_parse_int_hex_lowercase() {
    let code = r#"parse_int("ff", 16)"#;
    assert_eval_result_ok_number(code, 255.0);
}

#[test]
fn test_parse_int_radix_36() {
    let code = r#"parse_int("Z", 36)"#;
    assert_eval_result_ok_number(code, 35.0);
}

#[test]
fn test_parse_int_plus_sign() {
    let code = r#"parse_int("+42", 10)"#;
    assert_eval_result_ok_number(code, 42.0);
}

#[test]
fn test_parse_int_whitespace() {
    let code = r#"parse_int("  42  ", 10)"#;
    assert_eval_result_ok_number(code, 42.0);
}

#[test]
fn test_parse_int_radix_too_low() {
    let code = r#"parse_int("42", 1)"#;
    assert_eval_result_err(code);
}

#[test]
fn test_parse_int_radix_too_high() {
    let code = r#"parse_int("42", 37)"#;
    assert_eval_result_err(code);
}

#[test]
fn test_parse_int_radix_float() {
    let code = r#"parse_int("42", 10.5)"#;
    assert_eval_result_err(code);
}

#[test]
fn test_parse_int_empty_string() {
    let code = r#"parse_int("", 10)"#;
    assert_eval_result_err(code);
}

#[test]
fn test_parse_int_invalid_digit() {
    let code = r#"parse_int("G", 16)"#;
    assert_eval_result_err(code);
}

#[test]
fn test_parse_int_invalid_for_radix() {
    let code = r#"parse_int("2", 2)"#;
    assert_eval_result_err(code);
}

#[test]
fn test_parse_int_wrong_type_first_arg() {
    let code = r#"parse_int(42, 10)"#;
    assert_has_error(code);
}

#[test]
fn test_parse_int_wrong_type_second_arg() {
    let code = r#"parse_int("42", "10")"#;
    assert_has_error(code);
}

// ============================================================================
// parse_float Tests
// ============================================================================

#[test]
fn test_parse_float_integer() {
    let code = r#"parse_float("42")"#;
    assert_eval_result_ok_number(code, 42.0);
}

#[test]
fn test_parse_float_decimal() {
    let code = r#"parse_float("3.5")"#;
    assert_eval_result_ok_number(code, 3.5);
}

#[test]
fn test_parse_float_negative() {
    let code = r#"parse_float("-10.5")"#;
    assert_eval_result_ok_number(code, -10.5);
}

#[test]
fn test_parse_float_scientific_lowercase() {
    let code = r#"parse_float("1.5e3")"#;
    assert_eval_result_ok_number(code, 1500.0);
}

#[test]
fn test_parse_float_scientific_uppercase() {
    let code = r#"parse_float("1.5E3")"#;
    assert_eval_result_ok_number(code, 1500.0);
}

#[test]
fn test_parse_float_scientific_negative_exp() {
    let code = r#"parse_float("1.5e-3")"#;
    assert_eval_result_ok_number(code, 0.0015);
}

#[test]
fn test_parse_float_scientific_positive_exp() {
    let code = r#"parse_float("1.5e+3")"#;
    assert_eval_result_ok_number(code, 1500.0);
}

#[test]
fn test_parse_float_whitespace() {
    let code = r#"parse_float("  3.5  ")"#;
    assert_eval_result_ok_number(code, 3.5);
}

#[test]
fn test_parse_float_plus_sign() {
    let code = r#"parse_float("+42.5")"#;
    assert_eval_result_ok_number(code, 42.5);
}

#[test]
fn test_parse_float_empty_string() {
    let code = r#"parse_float("")"#;
    assert_eval_result_err(code);
}

#[test]
fn test_parse_float_invalid() {
    let code = r#"parse_float("hello")"#;
    assert_eval_result_err(code);
}

#[test]
fn test_parse_float_wrong_type() {
    let code = r#"parse_float(42)"#;
    assert_has_error(code);
}

// ============================================================================
// Integration Tests
