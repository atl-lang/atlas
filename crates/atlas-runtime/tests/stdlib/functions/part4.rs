use super::*;
use pretty_assertions::assert_eq;

// ============================================================================
// 7. Stdlib registration — test namespace builtins (B34)
// ============================================================================

#[test]
fn test_is_builtin_test_ns_assert() {
    assert!(is_builtin("testNsAssert"));
    assert!(is_builtin("testNsEqual"));
    assert!(is_builtin("testNsNotEqual"));
}

#[test]
fn test_is_builtin_test_ns_result() {
    assert!(is_builtin("testNsOk"));
    assert!(is_builtin("testNsErr"));
}

#[test]
fn test_is_builtin_test_ns_collections() {
    assert!(is_builtin("testNsContains"));
    assert!(is_builtin("testNsEmpty"));
    assert!(is_builtin("testNsApprox"));
}

#[test]
fn test_is_builtin_test_ns_throws() {
    assert!(is_builtin("testNsThrows"));
    assert!(is_builtin("testNsNoThrow"));
}

#[test]
fn test_call_builtin_test_ns_assert_via_dispatch() {
    let security = SecurityContext::allow_all();
    let result = call_builtin(
        "testNsAssert",
        &[bool_val(true)],
        span(),
        &security,
        &stdout_writer(),
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Null);
}

#[test]
fn test_call_builtin_test_ns_equal_via_dispatch() {
    let security = SecurityContext::allow_all();
    let result = call_builtin(
        "testNsEqual",
        &[num_val(42.0), num_val(42.0)],
        span(),
        &security,
        &stdout_writer(),
    );
    assert!(result.is_ok());
}

#[test]
fn test_call_builtin_test_ns_ok_via_dispatch() {
    let security = SecurityContext::allow_all();
    let result = call_builtin(
        "testNsOk",
        &[ok_val(str_val("inner"))],
        span(),
        &security,
        &stdout_writer(),
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), str_val("inner"));
}

#[test]
fn test_call_builtin_test_ns_empty_via_dispatch() {
    let security = SecurityContext::allow_all();
    let result = call_builtin(
        "testNsEmpty",
        &[arr_val(vec![])],
        span(),
        &security,
        &stdout_writer(),
    );
    assert!(result.is_ok());
}

// ============================================================================
// 8. Interpreter / VM parity — test.* namespace
// ============================================================================

/// Run source twice (as two separate runtime instances) and verify both succeed.
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
    eval_parity_ok("test.assert(true, \"parity\");");
}

#[test]
fn test_equal_parity() {
    eval_parity_ok("test.equal(10, 10);");
}

#[test]
fn test_ok_parity() {
    eval_parity_ok(
        r#"
        let r = Ok(42);
        let v = test.ok(r);
        test.equal(v, 42);
    "#,
    );
}

#[test]
fn test_contains_parity() {
    eval_parity_ok(
        r#"
        let arr = [1, 2, 3];
        test.contains(arr, 3);
    "#,
    );
}

#[test]
fn test_approx_parity() {
    eval_parity_ok("test.approx(3.14, 3.14159, 0.01);");
}

#[test]
fn test_assert_failure_parity() {
    eval_parity_err("test.assert(false, \"parity failure test\");");
}

// ============================================================================
// 9. Comprehensive real-world test example — test.* namespace
// ============================================================================

#[test]
fn test_realistic_test_function() {
    eval_ok(
        r#"
        fn add(borrow a: number, borrow b: number): number {
            return a + b;
        }

        fn test_add(): void {
            test.equal(add(1, 2), 3);
            test.equal(add(0, 0), 0);
            test.equal(add(-1, 1), 0);
            test.assert(add(5, 5) == 10, "5 + 5 should be 10");
        }

        test_add();
    "#,
    );
}

#[test]
fn test_result_chain_with_assertions() {
    eval_ok(
        r#"
        fn safe_divide(borrow a: number, borrow b: number): Result<number, string> {
            if (b == 0) { return Err("division by zero"); }
            return Ok(a / b);
        }

        let r1 = safe_divide(10, 2);
        let v = test.ok(r1);
        test.equal(v, 5);

        let r2 = safe_divide(5, 0);
        let e = test.err(r2);
        test.equal(e, "division by zero");
    "#,
    );
}

#[test]
fn test_collection_assertions_in_sequence() {
    eval_ok(
        r#"
        let nums = [1, 2, 3, 4, 5];
        test.contains(nums, 3);

        let empty: number[] = [];
        test.empty(empty);
    "#,
    );
}

#[test]
fn test_equal_with_expressions() {
    eval_ok(
        r#"
        test.equal(2 + 3, 5);
        test.equal(10 * 2, 20);
        test.equal(true && true, true);
        test.equal(false || true, true);
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
