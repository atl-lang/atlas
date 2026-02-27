use super::*;

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
// From vm_option_result_tests.rs
// ============================================================================

// VM tests for Option<T> and Result<T,E>
//
// BLOCKER 02-D: Built-in Generic Types
//
// These tests verify VM parity with interpreter for Option and Result support.
// Tests mirror option_result_tests.rs to ensure identical behavior.

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
// From vm_result_advanced_tests.rs
// ============================================================================

// VM tests for advanced Result<T,E> methods
//
// These tests verify VM parity with interpreter for advanced Result operations.
// Tests mirror result_advanced_tests.rs to ensure identical behavior (including ? operator).

// ============================================================================
// expect() Tests
// ============================================================================

#[test]
fn test_expect_ok() {
    assert_eval_number(r#"expect(Ok(42), "should have value")"#, 42.0);
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
