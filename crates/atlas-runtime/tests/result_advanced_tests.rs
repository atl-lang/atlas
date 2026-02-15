//! Advanced Result<T,E> method tests (interpreter)
//!
//! Tests for expect, result_map, result_map_err, result_and_then, result_or_else, result_ok, result_err, ? operator

mod common;
use common::*;

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
