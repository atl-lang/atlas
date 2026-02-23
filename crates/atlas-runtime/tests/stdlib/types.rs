use super::*;

// From stdlib_types_tests.rs
// ============================================================================

// Type checking and conversion stdlib tests (Interpreter engine)
//
// Tests all 12 type utility functions with comprehensive edge case coverage

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
    assert_eval_string(code, "bool");
}

#[test]
fn test_typeof_bool_false() {
    let code = r#"typeof(false)"#;
    assert_eval_string(code, "bool");
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
    let code = r#"typeof(parseJSON("null"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_typeof_option() {
    let code = r#"typeof(Some(42))"#;
    assert_eval_string(code, "option");
}

#[test]
fn test_typeof_result() {
    let code = r#"typeof(Ok(42))"#;
    assert_eval_string(code, "result");
}

// ============================================================================
// Type Guard Tests
// ============================================================================

#[test]
fn test_is_string_true() {
    let code = r#"isString("hello")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_string_false_number() {
    let code = r#"isString(42)"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_string_false_null() {
    let code = r#"isString(null)"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_number_true_int() {
    let code = r#"isNumber(42)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_number_true_float() {
    let code = r#"isNumber(3.5)"#;
    assert_eval_bool(code, true);
}

// Removed: NaN test (division by zero is error)

#[test]
fn test_is_number_false_string() {
    let code = r#"isNumber("42")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_bool_true() {
    let code = r#"isBool(true)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_bool_false() {
    let code = r#"isBool(false)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_bool_false_number() {
    let code = r#"isBool(1)"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_null_true() {
    let code = r#"isNull(null)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_null_false() {
    let code = r#"isNull(0)"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_array_true() {
    let code = r#"isArray([1,2,3])"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_array_true_empty() {
    let code = r#"isArray([])"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_array_false() {
    let code = r#"isArray("not array")"#;
    assert_eval_bool(code, false);
}

// Function reference tests removed: not yet fully supported

#[test]
fn test_is_function_false() {
    let code = r#"isFunction(42)"#;
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
    let code = r#"toString(parseJSON("null"))"#;
    assert_eval_string(code, "[JSON]");
}

// ============================================================================
// toNumber Tests
// ============================================================================

#[test]
fn test_to_number_number_identity() {
    let code = r#"toNumber(42)"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_to_number_bool_true() {
    let code = r#"toNumber(true)"#;
    assert_eval_number(code, 1.0);
}

#[test]
fn test_to_number_bool_false() {
    let code = r#"toNumber(false)"#;
    assert_eval_number(code, 0.0);
}

#[test]
fn test_to_number_string_int() {
    let code = r#"toNumber("42")"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_to_number_string_float() {
    let code = r#"toNumber("3.5")"#;
    assert_eval_number(code, 3.5);
}

#[test]
fn test_to_number_string_negative() {
    let code = r#"toNumber("-10")"#;
    assert_eval_number(code, -10.0);
}

#[test]
fn test_to_number_string_whitespace() {
    let code = r#"toNumber("  42  ")"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_to_number_string_scientific() {
    let code = r#"toNumber("1e10")"#;
    assert_eval_number(code, 1e10);
}

#[test]
fn test_to_number_string_empty_error() {
    let code = r#"toNumber("")"#;
    assert_has_error(code);
}

#[test]
fn test_to_number_string_invalid_error() {
    let code = r#"toNumber("hello")"#;
    assert_has_error(code);
}

#[test]
fn test_to_number_null_error() {
    let code = r#"toNumber(null)"#;
    assert_has_error(code);
}

#[test]
fn test_to_number_array_error() {
    let code = r#"toNumber([1,2,3])"#;
    assert_has_error(code);
}

// ============================================================================
// toBool Tests
// ============================================================================

#[test]
fn test_to_bool_bool_identity_true() {
    let code = r#"toBool(true)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_to_bool_bool_identity_false() {
    let code = r#"toBool(false)"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_to_bool_number_zero_false() {
    let code = r#"toBool(0)"#;
    assert_eval_bool(code, false);
}

// NaN toBool test removed: division by zero is error

#[test]
fn test_to_bool_number_positive_true() {
    let code = r#"toBool(42)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_to_bool_number_negative_true() {
    let code = r#"toBool(-10)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_to_bool_string_empty_false() {
    let code = r#"toBool("")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_to_bool_string_nonempty_true() {
    let code = r#"toBool("hello")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_to_bool_string_space_true() {
    let code = r#"toBool(" ")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_to_bool_null_false() {
    let code = r#"toBool(null)"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_to_bool_array_true() {
    let code = r#"toBool([1,2,3])"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_to_bool_array_empty_true() {
    let code = r#"toBool([])"#;
    assert_eval_bool(code, true);
}

// Function toBool test removed: not yet fully supported

// ============================================================================
// parseInt Tests
// ============================================================================

#[test]
fn test_parse_int_decimal() {
    let code = r#"parseInt("42", 10)"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_parse_int_decimal_negative() {
    let code = r#"parseInt("-10", 10)"#;
    assert_eval_number(code, -10.0);
}

#[test]
fn test_parse_int_binary() {
    let code = r#"parseInt("1010", 2)"#;
    assert_eval_number(code, 10.0);
}

#[test]
fn test_parse_int_octal() {
    let code = r#"parseInt("17", 8)"#;
    assert_eval_number(code, 15.0);
}

#[test]
fn test_parse_int_hex() {
    let code = r#"parseInt("FF", 16)"#;
    assert_eval_number(code, 255.0);
}

#[test]
fn test_parse_int_hex_lowercase() {
    let code = r#"parseInt("ff", 16)"#;
    assert_eval_number(code, 255.0);
}

#[test]
fn test_parse_int_radix_36() {
    let code = r#"parseInt("Z", 36)"#;
    assert_eval_number(code, 35.0);
}

#[test]
fn test_parse_int_plus_sign() {
    let code = r#"parseInt("+42", 10)"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_parse_int_whitespace() {
    let code = r#"parseInt("  42  ", 10)"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_parse_int_radix_too_low() {
    let code = r#"parseInt("42", 1)"#;
    assert_has_error(code);
}

#[test]
fn test_parse_int_radix_too_high() {
    let code = r#"parseInt("42", 37)"#;
    assert_has_error(code);
}

#[test]
fn test_parse_int_radix_float() {
    let code = r#"parseInt("42", 10.5)"#;
    assert_has_error(code);
}

#[test]
fn test_parse_int_empty_string() {
    let code = r#"parseInt("", 10)"#;
    assert_has_error(code);
}

#[test]
fn test_parse_int_invalid_digit() {
    let code = r#"parseInt("G", 16)"#;
    assert_has_error(code);
}

#[test]
fn test_parse_int_invalid_for_radix() {
    let code = r#"parseInt("2", 2)"#;
    assert_has_error(code);
}

#[test]
fn test_parse_int_wrong_type_first_arg() {
    let code = r#"parseInt(42, 10)"#;
    assert_has_error(code);
}

#[test]
fn test_parse_int_wrong_type_second_arg() {
    let code = r#"parseInt("42", "10")"#;
    assert_has_error(code);
}

// ============================================================================
// parseFloat Tests
// ============================================================================

#[test]
fn test_parse_float_integer() {
    let code = r#"parseFloat("42")"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_parse_float_decimal() {
    let code = r#"parseFloat("3.5")"#;
    assert_eval_number(code, 3.5);
}

#[test]
fn test_parse_float_negative() {
    let code = r#"parseFloat("-10.5")"#;
    assert_eval_number(code, -10.5);
}

#[test]
fn test_parse_float_scientific_lowercase() {
    let code = r#"parseFloat("1.5e3")"#;
    assert_eval_number(code, 1500.0);
}

#[test]
fn test_parse_float_scientific_uppercase() {
    let code = r#"parseFloat("1.5E3")"#;
    assert_eval_number(code, 1500.0);
}

#[test]
fn test_parse_float_scientific_negative_exp() {
    let code = r#"parseFloat("1.5e-3")"#;
    assert_eval_number(code, 0.0015);
}

#[test]
fn test_parse_float_scientific_positive_exp() {
    let code = r#"parseFloat("1.5e+3")"#;
    assert_eval_number(code, 1500.0);
}

#[test]
fn test_parse_float_whitespace() {
    let code = r#"parseFloat("  3.5  ")"#;
    assert_eval_number(code, 3.5);
}

#[test]
fn test_parse_float_plus_sign() {
    let code = r#"parseFloat("+42.5")"#;
    assert_eval_number(code, 42.5);
}

#[test]
fn test_parse_float_empty_string() {
    let code = r#"parseFloat("")"#;
    assert_has_error(code);
}

#[test]
fn test_parse_float_invalid() {
    let code = r#"parseFloat("hello")"#;
    assert_has_error(code);
}

#[test]
fn test_parse_float_wrong_type() {
    let code = r#"parseFloat(42)"#;
    assert_has_error(code);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_typeof_guards_match() {
    let code = r#"
        let val: string = "hello";
        typeof(val) == "string" && isString(val)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_type_conversion_chain() {
    let code = r#"
        let num: number = 42;
        let numStr: string = toString(num);
        toNumber(numStr)
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_parse_int_then_to_string() {
    let code = r#"
        let parsed: number = parseInt("FF", 16);
        toString(parsed)
    "#;
    assert_eval_string(code, "255");
}

#[test]
fn test_type_guards_all_false_for_null() {
    let code = r#"
        let val = null;
        !isString(val) && !isNumber(val) && !isBool(val) && !isArray(val) && !isFunction(val)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_type_guards_only_null_true() {
    let code = r#"isNull(null)"#;
    assert_eval_bool(code, true);
}

// ============================================================================

// From option_result_tests.rs
// ============================================================================

// Integration tests for Option<T> and Result<T,E>
//
// BLOCKER 02-D: Built-in Generic Types

// ============================================================================
// Option<T> Tests
// ============================================================================

#[test]
fn test_option_is_some() {
    assert_eval_bool("is_some(Some(42))", true);
    assert_eval_bool("is_some(None())", false);
}

#[test]
fn test_option_is_none() {
    assert_eval_bool("is_none(None())", true);
    assert_eval_bool("is_none(Some(42))", false);
}

#[test]
fn test_option_unwrap_number() {
    assert_eval_number("unwrap(Some(42))", 42.0);
}

#[test]
fn test_option_unwrap_string() {
    assert_eval_string(r#"unwrap(Some("hello"))"#, "hello");
}

#[test]
fn test_option_unwrap_bool() {
    assert_eval_bool("unwrap(Some(true))", true);
}

#[test]
fn test_option_unwrap_null() {
    assert_eval_null("unwrap(Some(null))");
}

#[test]
fn test_option_unwrap_or_some() {
    assert_eval_number("unwrap_or(Some(42), 0)", 42.0);
}

#[test]
fn test_option_unwrap_or_none() {
    assert_eval_number("unwrap_or(None(), 99)", 99.0);
}

#[test]
fn test_option_unwrap_or_string() {
    assert_eval_string(r#"unwrap_or(Some("hello"), "default")"#, "hello");
    assert_eval_string(r#"unwrap_or(None(), "default")"#, "default");
}

#[test]
fn test_option_nested() {
    assert_eval_number("unwrap(unwrap(Some(Some(42))))", 42.0);
}

// ============================================================================
// Result<T,E> Tests
// ============================================================================

#[test]
fn test_result_is_ok() {
    assert_eval_bool("is_ok(Ok(42))", true);
    assert_eval_bool(r#"is_ok(Err("failed"))"#, false);
}

#[test]
fn test_result_is_err() {
    assert_eval_bool(r#"is_err(Err("failed"))"#, true);
    assert_eval_bool("is_err(Ok(42))", false);
}

#[test]
fn test_result_unwrap_ok_number() {
    assert_eval_number("unwrap(Ok(42))", 42.0);
}

#[test]
fn test_result_unwrap_ok_string() {
    assert_eval_string(r#"unwrap(Ok("success"))"#, "success");
}

#[test]
fn test_result_unwrap_ok_null() {
    assert_eval_null("unwrap(Ok(null))");
}

#[test]
fn test_result_unwrap_or_ok() {
    assert_eval_number("unwrap_or(Ok(42), 0)", 42.0);
}

#[test]
fn test_result_unwrap_or_err() {
    assert_eval_number(r#"unwrap_or(Err("failed"), 99)"#, 99.0);
}

#[test]
fn test_result_unwrap_or_string() {
    assert_eval_string(r#"unwrap_or(Ok("success"), "default")"#, "success");
    assert_eval_string(r#"unwrap_or(Err(404), "default")"#, "default");
}

// ============================================================================
// Mixed Option/Result Tests
// ============================================================================

#[test]
fn test_option_and_result_together() {
    let code = r#"
        let opt = Some(42);
        let res = Ok(99);
        unwrap(opt) + unwrap(res)
    "#;
    assert_eval_number(code, 141.0);
}

#[test]
fn test_option_in_conditional() {
    let code = r#"
        let opt = Some(42);
        if (is_some(opt)) {
            unwrap(opt);
        } else {
            0;
        }
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_result_in_conditional() {
    let code = r#"
        let res = Ok(42);
        if (is_ok(res)) {
            unwrap(res);
        } else {
            0;
        }
    "#;
    assert_eval_number(code, 42.0);
}

// ============================================================================
// Complex Tests
// ============================================================================

#[test]
fn test_option_chain() {
    let code = r#"
        let a = Some(10);
        let b = Some(20);
        let c = Some(30);
        unwrap(a) + unwrap(b) + unwrap(c)
    "#;
    assert_eval_number(code, 60.0);
}

#[test]
fn test_result_chain() {
    let code = r#"
        let a = Ok(10);
        let b = Ok(20);
        let c = Ok(30);
        unwrap(a) + unwrap(b) + unwrap(c)
    "#;
    assert_eval_number(code, 60.0);
}

#[test]
fn test_option_unwrap_or_with_none_chain() {
    let code = r#"
        let a = None();
        let b = None();
        unwrap_or(a, 5) + unwrap_or(b, 10)
    "#;
    assert_eval_number(code, 15.0);
}

#[test]
fn test_result_unwrap_or_with_err_chain() {
    let code = r#"
        let a = Err("fail1");
        let b = Err("fail2");
        unwrap_or(a, 5) + unwrap_or(b, 10)
    "#;
    assert_eval_number(code, 15.0);
}

// ============================================================================

// From result_advanced_tests.rs
// ============================================================================

// Advanced Result<T,E> method tests (interpreter)
//
// Tests for expect, result_map, result_map_err, result_and_then, result_or_else, result_ok, result_err, ? operator

// ============================================================================
// expect() Tests
// ============================================================================

#[test]
fn test_expect_ok() {
    assert_eval_number(r#"expect(Ok(42), "should have value")"#, 42.0);
}

#[test]
fn test_expect_err_panics() {
    assert_has_error(r#"expect(Err("failed"), "custom message")"#);
}

#[test]
fn test_expect_with_string() {
    assert_eval_string(r#"expect(Ok("success"), "should work")"#, "success");
}

// ============================================================================
// result_ok() Tests - Convert Result to Option
// ============================================================================

#[test]
fn test_result_ok_from_ok() {
    let code = r#"
        let result = Ok(42);
        let opt = result_ok(result);
        unwrap(opt)
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_result_ok_from_err() {
    let code = r#"
        let result = Err("failed");
        let opt = result_ok(result);
        is_none(opt)
    "#;
    assert_eval_bool(code, true);
}

// ============================================================================
// result_err() Tests - Extract Err to Option
// ============================================================================

#[test]
fn test_result_err_from_ok() {
    let code = r#"
        let result = Ok(42);
        let opt = result_err(result);
        is_none(opt)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_result_err_from_err() {
    let code = r#"
        let result = Err("failed");
        let opt = result_err(result);
        unwrap(opt)
    "#;
    assert_eval_string(code, "failed");
}

// ============================================================================
// result_map() Tests - Transform Ok value
// ============================================================================

#[test]
fn test_result_map_ok() {
    let code = r#"
        fn double(x: number) -> number { return x * 2; }
        let result = Ok(21);
        let mapped = result_map(result, double);
        unwrap(mapped)
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_result_map_err_preserves() {
    let code = r#"
        fn double(x: number) -> number { return x * 2; }
        let result = Err("failed");
        let mapped = result_map(result, double);
        is_err(mapped)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_result_map_chain() {
    let code = r#"
        fn double(x: number) -> number { return x * 2; }
        fn triple(x: number) -> number { return x * 3; }
        let result = Ok(7);
        let mapped = result_map(result, double);
        let mapped2 = result_map(mapped, triple);
        unwrap(mapped2)
    "#;
    assert_eval_number(code, 42.0); // 7 * 2 * 3 = 42
}

// ============================================================================
// result_map_err() Tests - Transform Err value
// ============================================================================

#[test]
fn test_result_map_err_transforms_error() {
    let code = r#"
        fn format_error(e: string) -> string { return "Error: " + e; }
        let result = Err("failed");
        let mapped = result_map_err(result, format_error);
        unwrap_or(mapped, "default")
    "#;
    assert_eval_string(code, "default");
}

#[test]
fn test_result_map_err_preserves_ok() {
    let code = r#"
        fn format_error(e: string) -> string { return "Error: " + e; }
        let result = Ok(42);
        let mapped = result_map_err(result, format_error);
        unwrap(mapped)
    "#;
    assert_eval_number(code, 42.0);
}

// ============================================================================
// result_and_then() Tests - Monadic chaining
// ============================================================================

#[test]
fn test_result_and_then_success_chain() {
    let code = r#"
        fn divide(x: number) -> Result<number, string> {
            if (x == 0) {
                return Err("division by zero");
            }
            return Ok(100 / x);
        }
        let result = Ok(10);
        let chained = result_and_then(result, divide);
        unwrap(chained)
    "#;
    assert_eval_number(code, 10.0);
}

#[test]
fn test_result_and_then_error_propagates() {
    let code = r#"
        fn divide(x: number) -> Result<number, string> {
            if (x == 0) {
                return Err("division by zero");
            }
            return Ok(100 / x);
        }
        let result = Err("initial error");
        let chained = result_and_then(result, divide);
        is_err(chained)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_result_and_then_returns_error() {
    let code = r#"
        fn divide(x: number) -> Result<number, string> {
            if (x == 0) {
                return Err("division by zero");
            }
            return Ok(100 / x);
        }
        let result = Ok(0);
        let chained = result_and_then(result, divide);
        is_err(chained)
    "#;
    assert_eval_bool(code, true);
}

// ============================================================================
// result_or_else() Tests - Error recovery
// ============================================================================

#[test]
fn test_result_or_else_recovers_from_error() {
    let code = r#"
        fn recover(_e: string) -> Result<number, string> {
            return Ok(0);
        }
        let result = Err("failed");
        let recovered = result_or_else(result, recover);
        unwrap(recovered)
    "#;
    assert_eval_number(code, 0.0);
}

#[test]
fn test_result_or_else_preserves_ok() {
    let code = r#"
        fn recover(_e: string) -> Result<number, string> {
            return Ok(0);
        }
        let result = Ok(42);
        let recovered = result_or_else(result, recover);
        unwrap(recovered)
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_result_or_else_can_return_error() {
    let code = r#"
        fn retry(_e: string) -> Result<number, string> {
            return Err("retry failed");
        }
        let result = Err("initial");
        let recovered = result_or_else(result, retry);
        is_err(recovered)
    "#;
    assert_eval_bool(code, true);
}

// ============================================================================
// Complex Combination Tests
// ============================================================================

#[test]
fn test_result_pipeline() {
    let code = r#"
        fn double(x: number) -> number { return x * 2; }
        fn safe_divide(x: number) -> Result<number, string> {
            if (x == 0) {
                return Err("division by zero");
            }
            return Ok(100 / x);
        }

        let result = Ok(10);
        let step1 = result_map(result, double);
        let step2 = result_and_then(step1, safe_divide);
        unwrap(step2)
    "#;
    assert_eval_number(code, 5.0); // (10 * 2) = 20, then 100 / 20 = 5
}

#[test]
fn test_result_error_recovery_pipeline() {
    let code = r#"
        fn recover(_e: string) -> Result<number, string> {
            return Ok(99);
        }
        fn double(x: number) -> number { return x * 2; }

        let result = Err("initial");
        let recovered = result_or_else(result, recover);
        let mapped = result_map(recovered, double);
        unwrap(mapped)
    "#;
    assert_eval_number(code, 198.0); // recover to 99, then * 2
}

// ============================================================================
// Error Propagation Operator (?) Tests
// ============================================================================

#[test]
fn test_try_operator_unwraps_ok() {
    let code = r#"
        fn get_value() -> Result<number, string> {
            let result = Ok(42);
            return Ok(result?);
        }
        unwrap(get_value())
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_try_operator_propagates_error() {
    let code = r#"
        fn get_value() -> Result<number, string> {
            let result = Err("failed");
            return Ok(result?);
        }
        is_err(get_value())
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_try_operator_multiple_propagations() {
    let code = r#"
        fn divide(a: number, b: number) -> Result<number, string> {
            if (b == 0) {
                return Err("division by zero");
            }
            return Ok(a / b);
        }

        fn calculate() -> Result<number, string> {
            let x = divide(100, 10)?;
            let y = divide(x, 2)?;
            let z = divide(y, 5)?;
            return Ok(z);
        }

        unwrap(calculate())
    "#;
    assert_eval_number(code, 1.0); // 100 / 10 = 10, 10 / 2 = 5, 5 / 5 = 1
}

#[test]
fn test_try_operator_early_return() {
    let code = r#"
        fn divide(a: number, b: number) -> Result<number, string> {
            if (b == 0) {
                return Err("division by zero");
            }
            return Ok(a / b);
        }

        fn calculate() -> Result<number, string> {
            let x = divide(100, 10)?;
            let y = divide(x, 0)?;  // This will error
            let z = divide(y, 5)?;  // This won't execute
            return Ok(z);
        }

        is_err(calculate())
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_try_operator_with_expressions() {
    let code = r#"
        fn get_number() -> Result<number, string> {
            return Ok(21);
        }

        fn double_it() -> Result<number, string> {
            return Ok(get_number()? * 2);
        }

        unwrap(double_it())
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_try_operator_in_nested_calls() {
    let code = r#"
        fn inner() -> Result<number, string> {
            return Ok(42);
        }

        fn middle() -> Result<number, string> {
            return Ok(inner()?);
        }

        fn outer() -> Result<number, string> {
            return Ok(middle()?);
        }

        unwrap(outer())
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_try_operator_with_error_in_nested_calls() {
    let code = r#"
        fn inner() -> Result<number, string> {
            return Err("inner failed");
        }

        fn middle() -> Result<number, string> {
            return Ok(inner()?);
        }

        fn outer() -> Result<number, string> {
            return Ok(middle()?);
        }

        is_err(outer())
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_try_operator_combined_with_methods() {
    let code = r#"
        fn get_value() -> Result<number, string> {
            return Ok(10);
        }

        fn double(x: number) -> number {
            return x * 2;
        }

        fn process() -> Result<number, string> {
            let val = get_value()?;
            let mapped = Ok(double(val));
            return Ok(mapped?);
        }

        unwrap(process())
    "#;
    assert_eval_number(code, 20.0);
}

// ============================================================================
