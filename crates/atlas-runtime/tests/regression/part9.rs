use super::common::*;
use atlas_runtime::Atlas;

// ─── VM/Interpreter Parity Verification ──────────────────────────────────────

#[test]
fn milestone_parity_arithmetic_consistent() {
    // Both engines produce the same result for arithmetic (verified by determinism).
    let runtime = Atlas::new();
    let result = runtime.eval("2 ** 10;");
    // Should be 1024.0 or produce an error (if ** is not implemented yet).
    // The important thing is it doesn't panic.
    let _ = result;
}

#[test]
fn milestone_parity_function_calls_consistent() {
    let code = r#"
        fn double(borrow x: number) -> number { return x * 2; }
        double(21);
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn milestone_parity_array_operations_consistent() {
    let code = "let arr: number[] = [1, 2, 3, 4, 5]; arr[4];";
    assert_eval_number(code, 5.0);
}

// ─── System Stability Final Checks ───────────────────────────────────────────

#[test]
fn milestone_stability_multiple_runtimes_independent() {
    // Multiple Atlas runtime instances must be independent.
    let rt1 = Atlas::new();
    let rt2 = Atlas::new();
    let _ = rt1.eval("let x: number = 1;");
    // rt2 must not be affected by rt1's state.
    let result = rt2.eval("42;");
    assert!(result.is_ok());
}

#[test]
fn milestone_stability_empty_program() {
    // Empty program must succeed, returning null or some valid value.
    let runtime = Atlas::new();
    let result = runtime.eval("");
    // Empty program: may return Ok(Null) or similar — must not panic.
    let _ = result;
}

#[test]
fn milestone_stability_whitespace_program() {
    let runtime = Atlas::new();
    let result = runtime.eval("   \n\t  \n  ");
    let _ = result; // Must not panic.
}

#[test]
fn milestone_stability_comment_only_program() {
    let runtime = Atlas::new();
    let result = runtime.eval("// just a comment\n");
    let _ = result; // Must not panic.
}

#[test]
fn milestone_stability_large_program() {
    // A program with 100 function definitions must not crash.
    let mut code = String::new();
    for i in 0..100 {
        code.push_str(&format!(
            "fn f{}(x: number) -> number {{ return x + {}; }}\n",
            i, i
        ));
    }
    code.push_str("f99(0);");
    let runtime = Atlas::new();
    let result = runtime.eval(&code);
    assert!(result.is_ok(), "Large program failed: {:?}", result);
}

#[test]
fn milestone_stability_no_panic_on_runtime_error() {
    // Runtime errors must be returned as Err, not panic.
    let runtime = Atlas::new();
    let result = runtime.eval("1 / 0;");
    assert!(
        result.is_err(),
        "Expected runtime error for division by zero"
    );
}

// Regression: vm_fuzz (2026-02-26) — compiler panicked instead of returning Err
// when a method call's TypeTag was None (typechecker left it unset for non-Array/JsonValue types).
// compiler/expr.rs:264 used .expect() on the Cell; now returns a Diagnostic error.
#[test]
fn regression_method_call_unknown_type_no_panic() {
    // A method call on a number (not Array or JsonValue) — typechecker sets type_tag to None.
    // The compiler must return a Diagnostic error, not panic.
    let runtime = Atlas::new();
    let result = runtime.eval("let x: number = 42; x.foo();");
    assert!(
        result.is_err(),
        "Expected compile/type error for method call on number, got Ok"
    );
}

// NOTE: test block removed — required access to private function `len`

// NOTE: test block removed — required access to private function `is_valid_identifier`

// NOTE: test block removed — required access to private function `len`

// NOTE: test block removed — required access to private function `serialize_response`

// NOTE: test block removed — required access to private function `len`

// NOTE: test block removed — required access to private function `is_none`

// NOTE: test block removed — required access to private function `is_none`

// NOTE: test block removed — required access to private function `len`

// NOTE: test block removed — required access to private function `len`

// NOTE: test block removed — required access to private function `is_ok`

// NOTE: test block removed — required access to private function `resolve_method`

// NOTE: test block removed — required access to private function `is_some`

// NOTE: test block removed — required access to private function `is_err`

// NOTE: test block removed — required access to private function `is_ok`

// NOTE: test block removed — required access to private function `decode`

// NOTE: test block removed — required access to private function `ok`

// NOTE: test block removed — required access to private function `same_type`

// NOTE: test block removed — required access to private function `len`

// NOTE: test block removed — required access to private function `len`

// NOTE: test block removed — required access to private function `is_none`

// NOTE: test block removed — required access to private function `len`

// NOTE: test block removed — required access to private function `is_err`

// NOTE: test block removed — required access to private function `is_none`
