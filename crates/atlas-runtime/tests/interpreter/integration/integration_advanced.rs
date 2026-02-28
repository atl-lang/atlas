use super::*;
use pretty_assertions::assert_eq;

fn test_integration_nested_function_with_params() {
    // Test nested function that takes parameters (avoids closure capture warnings)
    let code = r#"
        fn outer(x: number) -> number {
            fn inner(y: number) -> number {
                return y * 2;
            }
            return inner(x);
        }
        outer(10);
    "#;
    assert_eval_number(code, 20.0);
}

#[test]
fn test_integration_nested_function_calls() {
    let code = r#"
        fn a(x: number) -> number { return x + 1; }
        fn b(x: number) -> number { return a(x) + 1; }
        fn c(x: number) -> number { return b(x) + 1; }
        c(0);
    "#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_integration_scope_shadowing() {
    let code = r#"
        let x = 1;
        fn test() -> number {
            let x = 2;
            return x;
        }
        test() + x;
    "#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_integration_multiple_function_levels() {
    // Test function calls across multiple levels
    let code = r#"
        fn level1(x: number) -> number {
            fn level2(y: number) -> number {
                fn level3(z: number) -> number {
                    return z + 1;
                }
                return level3(y) + 1;
            }
            return level2(x) + 1;
        }
        level1(0);
    "#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_integration_function_as_parameter() {
    // Test higher-order function pattern
    let code = r#"
        fn apply(f: (number) -> number, x: number) -> number {
            return f(x);
        }
        fn double(n: number) -> number {
            return n * 2;
        }
        apply(double, 5);
    "#;
    assert_eval_number(code, 10.0);
}

// ============================================================================
// Phase interpreter-02: Integration Tests - Error Recovery
// ============================================================================

#[test]
fn test_integration_undefined_variable_error() {
    let result = Atlas::new().eval("undefined_var;");
    assert!(result.is_err(), "Expected error for undefined variable");
}

#[test]
fn test_integration_type_mismatch_error() {
    let result = Atlas::new().eval(r#"let x: number = "hello";"#);
    assert!(result.is_err(), "Expected type mismatch error");
}

#[test]
fn test_integration_divide_by_zero_error() {
    assert_error_code("10 / 0;", "AT0005");
}

#[test]
fn test_integration_array_index_out_of_bounds() {
    let result = Atlas::new().eval("let arr = [1, 2, 3]; arr[10];");
    assert!(result.is_err(), "Expected array index out of bounds error");
}

#[test]
fn test_integration_function_wrong_arity() {
    let code = r#"
        fn add(a: number, b: number) -> number { return a + b; }
        add(1);
    "#;
    let result = Atlas::new().eval(code);
    assert!(result.is_err(), "Expected function arity error");
}

// ============================================================================
// Phase interpreter-02: Integration Tests - Complex Programs
// ============================================================================

#[test]
fn test_integration_fibonacci_recursive() {
    let code = r#"
        fn fib(n: number) -> number {
            if (n <= 1) { return n; }
            return fib(n - 1) + fib(n - 2);
        }
        fib(10);
    "#;
    assert_eval_number(code, 55.0);
}

#[test]
fn test_integration_factorial() {
    let code = r#"
        fn factorial(n: number) -> number {
            if (n <= 1) { return 1; }
            return n * factorial(n - 1);
        }
        factorial(5);
    "#;
    assert_eval_number(code, 120.0);
}

#[test]
fn test_integration_sum_to_n() {
    let code = r#"
        fn sum_to(n: number) -> number {
            var sum = 0;
            var i = 1;
            while (i <= n) {
                sum = sum + i;
                i = i + 1;
            }
            return sum;
        }
        sum_to(100);
    "#;
    assert_eval_number(code, 5050.0);
}

#[test]
fn test_integration_is_prime() {
    let code = r#"
        fn is_prime(n: number) -> bool {
            if (n < 2) { return false; }
            var i = 2;
            while (i * i <= n) {
                if (n % i == 0) { return false; }
                i = i + 1;
            }
            return true;
        }
        is_prime(17);
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_integration_is_not_prime() {
    let code = r#"
        fn is_prime(n: number) -> bool {
            if (n < 2) { return false; }
            var i = 2;
            while (i * i <= n) {
                if (n % i == 0) { return false; }
                i = i + 1;
            }
            return true;
        }
        is_prime(15);
    "#;
    assert_eval_bool(code, false);
}

// ============================================================================
// Phase interpreter-02: Integration Tests - Stdlib Functions
// ============================================================================

#[test]
fn test_integration_stdlib_len_string() {
    assert_eval_number(r#"len("hello");"#, 5.0);
}

#[test]
fn test_integration_stdlib_len_array() {
    assert_eval_number("len([1, 2, 3, 4, 5]);", 5.0);
}

#[test]
fn test_integration_stdlib_str() {
    assert_eval_string("str(42);", "42");
}

#[test]
fn test_integration_stdlib_trim() {
    assert_eval_string(r#"trim("  hello  ");"#, "hello");
}

#[test]
fn test_integration_stdlib_split_join() {
    let code = r#"
        let parts = split("a,b,c", ",");
        join(parts, "-");
    "#;
    assert_eval_string(code, "a-b-c");
}

#[test]
fn test_integration_stdlib_substring() {
    assert_eval_string(r#"substring("hello world", 0, 5);"#, "hello");
}

#[test]
fn test_integration_stdlib_includes() {
    assert_eval_bool(r#"includes("hello world", "world");"#, true);
}

#[test]
fn test_integration_stdlib_starts_with() {
    assert_eval_bool(r#"startsWith("hello world", "hello");"#, true);
}

#[test]
fn test_integration_stdlib_ends_with() {
    assert_eval_bool(r#"endsWith("hello world", "world");"#, true);
}

#[test]
fn test_integration_stdlib_replace() {
    assert_eval_string(
        r#"replace("hello world", "world", "atlas");"#,
        "hello atlas",
    );
}

// ============================================================================
// Phase interpreter-02: Performance Correctness Tests
// ============================================================================

#[test]
fn test_perf_loop_1000_iterations() {
    let code = "var i = 0; while (i < 1000) { i = i + 1; } i;";
    assert_eval_number(code, 1000.0);
}

#[test]
fn test_perf_nested_loop_correctness() {
    let code = r#"
        var count = 0;
        var i = 0;
        while (i < 10) {
            var j = 0;
            while (j < 10) {
                count = count + 1;
                j = j + 1;
            }
            i = i + 1;
        }
        count;
    "#;
    assert_eval_number(code, 100.0);
}

#[test]
fn test_perf_string_accumulation() {
    let code = r#"
        var s = "";
        var i = 0;
        while (i < 50) {
            s = s + "x";
            i = i + 1;
        }
        len(s);
    "#;
    assert_eval_number(code, 50.0);
}

#[test]
fn test_perf_function_calls_correctness() {
    let code = r#"
        fn inc(x: number) -> number { return x + 1; }
        var r = 0;
        var i = 0;
        while (i < 100) {
            r = inc(r);
            i = i + 1;
        }
        r;
    "#;
    assert_eval_number(code, 100.0);
}

#[test]
fn test_perf_array_operations() {
    // Test array indexing performance
    let code = r#"
        let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        var sum = 0;
        var i = 0;
        while (i < 100) {
            sum = sum + arr[i % 10];
            i = i + 1;
        }
        sum;
    "#;
    assert_eval_number(code, 550.0); // sum of 1-10 is 55, times 10 = 550
}
