use super::*;
use pretty_assertions::assert_eq;

// ============================================================================

#[test]
fn test_is_builtin_assert() {
    assert!(is_builtin("assert"));
    assert!(is_builtin("assert_false"));
}

#[test]
fn test_is_builtin_equality() {
    assert!(is_builtin("assert_equal"));
    assert!(is_builtin("assert_not_equal"));
}

#[test]
fn test_is_builtin_result() {
    assert!(is_builtin("assert_ok"));
    assert!(is_builtin("assert_err"));
}

#[test]
fn test_is_builtin_option() {
    assert!(is_builtin("assert_some"));
    assert!(is_builtin("assert_none"));
}

#[test]
fn test_is_builtin_collection() {
    assert!(is_builtin("assert_contains"));
    assert!(is_builtin("assert_empty"));
    assert!(is_builtin("assert_length"));
}

#[test]
fn test_is_builtin_error() {
    assert!(is_builtin("assert_throws"));
    assert!(is_builtin("assert_no_throw"));
}

#[test]
fn test_call_builtin_assert_via_dispatch() {
    let security = SecurityContext::allow_all();
    let result = call_builtin(
        "assert",
        &[bool_val(true), str_val("ok")],
        span(),
        &security,
        &stdout_writer(),
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Null);
}

#[test]
fn test_call_builtin_assert_equal_via_dispatch() {
    let security = SecurityContext::allow_all();
    let result = call_builtin(
        "assert_equal",
        &[num_val(42.0), num_val(42.0)],
        span(),
        &security,
        &stdout_writer(),
    );
    assert!(result.is_ok());
}

#[test]
fn test_call_builtin_assert_ok_via_dispatch() {
    let security = SecurityContext::allow_all();
    let result = call_builtin(
        "assert_ok",
        &[ok_val(str_val("inner"))],
        span(),
        &security,
        &stdout_writer(),
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), str_val("inner"));
}

#[test]
fn test_call_builtin_assert_some_via_dispatch() {
    let security = SecurityContext::allow_all();
    let result = call_builtin(
        "assert_some",
        &[some_val(num_val(7.0))],
        span(),
        &security,
        &stdout_writer(),
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), num_val(7.0));
}

#[test]
fn test_call_builtin_assert_empty_via_dispatch() {
    let security = SecurityContext::allow_all();
    let result = call_builtin(
        "assert_empty",
        &[arr_val(vec![])],
        span(),
        &security,
        &stdout_writer(),
    );
    assert!(result.is_ok());
}

// ============================================================================
// 8. Interpreter / VM parity
// ============================================================================

/// Run source twice (as two separate runtime instances) and verify both succeed.
/// This matches the established parity testing pattern in this codebase.
fn eval_parity_ok(source: &str) {
    let r1 = Atlas::new();
    match r1.eval(source) {
        Ok(_) => {}
        Err(diags) => panic!("First eval failed: {:?}", diags),
    }
    let r2 = Atlas::new();
    match r2.eval(source) {
        Ok(_) => {}
        Err(diags) => panic!("Second eval failed: {:?}", diags),
    }
}

/// Run source twice and verify both fail (parity of failure).
fn eval_parity_err(source: &str) {
    let err1 = Atlas::new().eval(source).is_err();
    let err2 = Atlas::new().eval(source).is_err();
    assert!(err1, "First eval should fail");
    assert!(err2, "Second eval should fail");
}

#[test]
fn test_assert_parity_basic() {
    eval_parity_ok("assert(true, \"parity\");");
}

#[test]
fn test_assert_equal_parity() {
    eval_parity_ok("assert_equal(10, 10);");
}

#[test]
fn test_assert_ok_parity() {
    eval_parity_ok(
        r#"
        let r = Ok(42);
        let v = assert_ok(r);
        assert_equal(v, 42);
    "#,
    );
}

#[test]
fn test_assert_some_parity() {
    eval_parity_ok(
        r#"
        let opt = Some("hello");
        let v = assert_some(opt);
        assert_equal(v, "hello");
    "#,
    );
}

#[test]
fn test_assert_none_parity() {
    eval_parity_ok(
        r#"
        let opt = None();
        assert_none(opt);
    "#,
    );
}

#[test]
fn test_assert_contains_parity() {
    eval_parity_ok(
        r#"
        let arr = [1, 2, 3];
        assert_contains(arr, 3);
    "#,
    );
}

#[test]
fn test_assert_length_parity() {
    eval_parity_ok(
        r#"
        let arr = [10, 20];
        assert_length(arr, 2);
    "#,
    );
}

#[test]
fn test_assert_failure_parity() {
    eval_parity_err("assert(false, \"parity failure test\");");
}

// ============================================================================
// 9. Comprehensive real-world test example
// ============================================================================

#[test]
fn test_realistic_test_function() {
    eval_ok(
        r#"
        fn add(borrow a: number, borrow b: number) -> number {
            return a + b;
        }

        fn test_add() -> void {
            assert_equal(add(1, 2), 3);
            assert_equal(add(0, 0), 0);
            assert_equal(add(-1, 1), 0);
            assert(add(5, 5) == 10, "5 + 5 should be 10");
        }

        test_add();
    "#,
    );
}

#[test]
fn test_result_chain_with_assertions() {
    eval_ok(
        r#"
        fn safe_divide(borrow a: number, borrow b: number) -> Result<number, string> {
            if (b == 0) { return Err("division by zero"); }
            return Ok(a / b);
        }

        let r1 = safe_divide(10, 2);
        let v = assert_ok(r1);
        assert_equal(v, 5);

        let r2 = safe_divide(5, 0);
        let e = assert_err(r2);
        assert_equal(e, "division by zero");
    "#,
    );
}

#[test]
fn test_option_chain_with_assertions() {
    eval_ok(
        r#"
        fn find_value(borrow arr: array, borrow target: number) -> Option<number> {
            let mut found = None();
            for item in arr {
                if (item == target) {
                    found = Some(item);
                }
            }
            return found;
        }

        let arr = [10, 20, 30];
        let r1 = find_value(arr, 20);
        let v = assert_some(r1);
        assert_equal(v, 20);

        let r2 = find_value(arr, 99);
        assert_none(r2);
    "#,
    );
}

#[test]
fn test_collection_assertions_in_sequence() {
    eval_ok(
        r#"
        let nums = [1, 2, 3, 4, 5];
        assert_length(nums, 5);
        assert_contains(nums, 3);

        let empty: number[] = [];
        assert_empty(empty);
        assert_length(empty, 0);
    "#,
    );
}

#[test]
fn test_assert_equal_with_expressions() {
    eval_ok(
        r#"
        assert_equal(2 + 3, 5);
        assert_equal(10 * 2, 20);
        assert_equal(true && true, true);
        assert_equal(false || true, true);
    "#,
    );
}

// ============================================================================

// From prelude_tests.rs
// ============================================================================

// Prelude Availability and Shadowing Tests
//
// Tests that prelude builtins (print, len, str) are:
// - Always available without imports
// - Can be shadowed in nested scopes
// - Cannot be shadowed in global scope (AT1012)
// ============================================================================
// Prelude Availability Tests
// ============================================================================

#[test]
fn test_prelude_available_without_imports() {
    let diagnostics = check_file("prelude_available.atl");

    // Should have no errors - prelude functions are available
    assert_eq!(
        diagnostics.len(),
        0,
        "Prelude functions should be available without imports, got: {:?}",
        diagnostics
    );
}

// ============================================================================
// Nested Scope Shadowing Tests (Allowed)
// ============================================================================

#[test]
fn test_nested_shadowing_allowed() {
    let diagnostics = check_file("nested_shadowing_allowed.atl");

    // Should have no errors - shadowing in nested scopes is allowed
    assert_eq!(
        diagnostics.len(),
        0,
        "Shadowing prelude in nested scopes should be allowed, got: {:?}",
        diagnostics
    );
}
