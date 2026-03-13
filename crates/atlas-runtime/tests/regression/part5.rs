use super::common::*;
use atlas_runtime::Atlas;

// ─── Stress Tests ────────────────────────────────────────────────────────────

#[test]
fn stability_stress_recursion_depth_50() {
    // Moderate recursion (50 levels) must complete without stack overflow.
    let code = r#"
        fn countdown(borrow n: number): number {
            if (n <= 0) { return 0; }
            return countdown(n - 1);
        }
        countdown(50);
    "#;
    assert_eval_number(code, 0.0);
}

#[test]
fn stability_stress_recursion_depth_100() {
    // Deeper recursion (100 levels). Run on a thread with an explicit 8MB stack
    // to avoid overflow in debug builds where Rust frames are larger than release.
    let code = r#"
        fn sum_down(borrow n: number): number {
            if (n <= 0) { return 0; }
            return n + sum_down(n - 1);
        }
        sum_down(100);
    "#;
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| assert_eval_number(code, 5050.0))
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn stability_stress_large_array_100_elements() {
    // 100 element array should be allocated and accessed without issues.
    let elements: Vec<String> = (0..100).map(|i| i.to_string()).collect();
    let code = format!("let arr: number[] = [{}]; arr[99];", elements.join(", "));
    assert_eval_number(&code, 99.0);
}

#[test]
fn stability_stress_large_array_500_elements() {
    // 500 element array stress test.
    let elements: Vec<String> = (0..500).map(|i| i.to_string()).collect();
    let code = format!("let arr: number[] = [{}]; arr[499];", elements.join(", "));
    assert_eval_number(&code, 499.0);
}

#[test]
fn stability_stress_many_variables() {
    // Programs with many variables should not exhaust resources.
    let mut code = String::new();
    for i in 0..50 {
        code.push_str(&format!("let v{}: number = {};\n", i, i));
    }
    code.push_str("v49;");
    assert_eval_number(&code, 49.0);
}

#[test]
fn stability_stress_long_string() {
    // A string of 1000 characters should be handled without issues.
    let long = "a".repeat(1000);
    let code = format!(r#""{}";"#, long);
    let runtime = Atlas::new();
    let result = runtime.eval(&code);
    assert!(
        result.is_ok(),
        "Long string evaluation failed: {:?}",
        result
    );
}

#[test]
fn stability_stress_many_function_calls() {
    // Many sequential function calls should not exhaust resources.
    let code = r#"
        fn test(): number {
            fn inc(borrow x: number): number { return x + 1; }
            let mut n: number = 0;
            let mut i: number = 0;
            while (i < 200) {
                n = inc(n);
                i = i + 1;
            }
            n
        }
        test()
    "#;
    assert_eval_number(code, 200.0);
}

#[test]
fn stability_stress_deep_if_else_nesting() {
    // Deeply nested conditionals (10 levels).
    let code = r#"
        fn test(): number {
            let x: number = 5;
            if (x > 0) {
                if (x > 1) {
                    if (x > 2) {
                        if (x > 3) {
                            if (x > 4) {
                                return 42;
                            } else { return 0; }
                        } else { return 0; }
                    } else { return 0; }
                } else { return 0; }
            } else { return 0; }
        }
        test()
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn stability_stress_while_1000_iterations() {
    // 1000 loop iterations should complete successfully.
    let code = r#"
        let mut sum: number = 0;
        let mut i: number = 0;
        while (i < 1000) {
            sum = sum + 1;
            i = i + 1;
        }
        sum;
    "#;
    assert_eval_number(code, 1000.0);
}

#[test]
fn stability_stress_fibonacci_15() {
    // Fibonacci(15) = 610 — exercises recursive call depth.
    let code = r#"
        fn fib(borrow n: number): number {
            if (n <= 1) { return n; }
            return fib(n - 1) + fib(n - 2);
        }
        fib(15);
    "#;
    assert_eval_number(code, 610.0);
}

// ─── Error Recovery Tests ─────────────────────────────────────────────────────

#[test]
fn stability_error_recovery_undefined_variable() {
    // Accessing an undefined variable must return an error, not panic.
    assert_has_error("undefined_var;");
}

#[test]
fn stability_error_recovery_type_mismatch() {
    // Type mismatch must be caught at compile time, not crash at runtime.
    assert_has_error("let x: number = true;");
}

#[test]
fn stability_error_recovery_divide_by_zero() {
    // Divide by zero must produce a runtime error, not a panic.
    assert_has_error("1 / 0;");
}

#[test]
fn stability_error_recovery_array_out_of_bounds() {
    // Out-of-bounds access must produce a runtime error, not a panic.
    assert_has_error("let arr: number[] = [1, 2]; arr[10];");
}

#[test]
fn stability_error_recovery_wrong_argument_count() {
    // Calling a function with wrong arity must produce an error.
    assert_has_error("fn f(borrow x: number): number { return x; } f(1, 2);");
}

#[test]
fn stability_error_recovery_wrong_return_type() {
    // Returning wrong type must produce a type error.
    assert_has_error("fn f(): number { return true; }");
}

#[test]
fn stability_error_recovery_multiple_errors() {
    // Programs with multiple errors must not crash even on first error.
    let code = "let a: number = true; let b: string = 42;";
    let runtime = Atlas::new();
    let result = runtime.eval(code);
    assert!(
        result.is_err(),
        "Expected errors for type-mismatched declarations"
    );
}

#[test]
fn stability_error_recovery_unclosed_string() {
    // Unclosed string literal must produce a lex/parse error, not panic.
    assert_has_error(r#""unclosed string"#);
}

#[test]
fn stability_error_recovery_invalid_operator_usage() {
    // Applying operators to wrong types must produce an error.
    assert_has_error(r#"true + 1;"#);
}

#[test]
fn stability_error_recovery_call_non_function() {
    // Calling a non-function value must produce a runtime error.
    assert_has_error("let x: number = 42; x();");
}

// ─── Release Mode Verification Tests ─────────────────────────────────────────
// These tests verify behaviors that must hold in release mode builds.
// They are designed to catch issues that only manifest when optimizations are on.

#[test]
fn stability_release_arithmetic_precision() {
    // Arithmetic should be precise in both debug and release mode.
    assert_eval_number("100.0 * 100.0;", 10_000.0);
}

#[test]
fn stability_release_large_number_arithmetic() {
    // Large float arithmetic should not lose precision unexpectedly.
    assert_eval_number("1000000.0 + 1.0;", 1_000_001.0);
}

#[test]
fn stability_release_boolean_short_circuit() {
    // Short-circuit evaluation must work correctly in release mode.
    assert_eval_bool("false && true;", false);
    assert_eval_bool("true || false;", true);
}

#[test]
fn stability_release_recursive_correctness() {
    // Recursive functions must produce correct results (not optimized away).
    let code = r#"
        fn factorial(borrow n: number): number {
            if (n <= 1) { return 1; }
            return n * factorial(n - 1);
        }
        factorial(10);
    "#;
    assert_eval_number(code, 3_628_800.0);
}

#[test]
fn stability_release_string_operations() {
    // String concatenation must work correctly in release mode.
    assert_eval_string(r#""foo" + "bar";"#, "foobar");
}

#[test]
fn stability_release_comparison_operators() {
    assert_eval_bool("1 < 2;", true);
    assert_eval_bool("2 < 1;", false);
    assert_eval_bool("1 == 1;", true);
    assert_eval_bool("1 != 2;", true);
    assert_eval_bool("2 >= 2;", true);
    assert_eval_bool("1 <= 2;", true);
}

#[test]
fn stability_release_loop_termination() {
    // Loops must terminate correctly in release mode (no optimizer infinite loop).
    let code = r#"
        let mut x: number = 10;
        while (x > 0) {
            x = x - 1;
        }
        x;
    "#;
    assert_eval_number(code, 0.0);
}

#[test]
fn stability_release_variable_mutation() {
    // Variable mutation must work correctly (not cached/inlined incorrectly).
    let code = r#"
        let mut x: number = 1;
        x = x + 1;
        x = x * 2;
        x;
    "#;
    assert_eval_number(code, 4.0);
}

#[test]
fn stability_release_nested_scope() {
    // Nested scopes must be correctly maintained in release mode.
    let code = r#"
        let x: number = 10;
        fn f(): number {
            let y: number = 20;
            return x + y;
        }
        f();
    "#;
    assert_eval_number(code, 30.0);
}

#[test]
fn stability_release_error_codes_preserved() {
    // Error codes must be the same in debug and release builds.
    assert_error_code("1 / 0;", "AT0005");
    assert_error_code("let arr: number[] = [1]; arr[5];", "AT0006");
}
