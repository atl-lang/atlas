//! Function declarations, calls, and recursion tests

use rstest::rstest;

use crate::common::*;

#[test]
fn test_function_definition_and_call() {
    let code = r#"
        fn add(a: number, b: number) -> number {
            return a + b;
        }
        add(3, 4)
    "#;
    assert_eval_number(code, 7.0);
}

#[test]
fn test_function_with_local_return() {
    let code = r#"
        fn foo(x: number) -> number {
            let y: number = x + 1;
            return y;
        }
        foo(5)
    "#;
    assert_eval_number(code, 6.0);
}

#[test]
fn test_function_with_early_return() {
    let code = r#"
        fn abs(x: number) -> number {
            if (x < 0) {
                return -x;
            }
            return x;
        }
        abs(-5)
    "#;
    assert_eval_number(code, 5.0);
}

#[test]
fn test_function_recursion() {
    let code = r#"
        fn factorial(n: number) -> number {
            if (n <= 1) {
                return 1;
            }
            return n * factorial(n - 1);
        }
        factorial(5)
    "#;
    assert_eval_number(code, 120.0);
}

#[test]
fn test_function_with_local_variables() {
    let code = r#"
        fn compute(x: number) -> number {
            let a: number = x + 1;
            let b: number = a * 2;
            return b - 1;
        }
        compute(5)
    "#;
    assert_eval_number(code, 11.0);
}

#[test]
fn test_function_nested_calls() {
    let code = r#"
        fn add(a: number, b: number) -> number {
            return a + b;
        }
        fn multiply(x: number, y: number) -> number {
            return x * y;
        }
        fn compute(n: number) -> number {
            return add(multiply(n, 2), 5);
        }
        compute(3)
    "#;
    assert_eval_number(code, 11.0);
}

#[rstest]
#[case(
    "fn add(a: number, b: number) -> number { return a + b; } add(5)",
    "AT3005"
)]
#[case(
    "fn add(a: number, b: number) -> number { return a + b; } add(1, 2, 3)",
    "AT3005"
)]
fn test_function_wrong_arity(#[case] code: &str, #[case] error_code: &str) {
    assert_error_code(code, error_code);
}

#[test]
fn test_function_void_return() {
    let code = r#"
        var result: number = 0;
        fn set_result(x: number) -> void {
            result = x;
        }
        set_result(42);
        result
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_function_no_parameters() {
    let code = r#"
        fn get_answer() -> number {
            return 42;
        }
        get_answer()
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_function_multiple_parameters() {
    let code = r#"
        fn sum_four(a: number, b: number, c: number, d: number) -> number {
            return a + b + c + d;
        }
        sum_four(1, 2, 3, 4)
    "#;
    assert_eval_number(code, 10.0);
}

#[test]
fn test_function_call_stack_depth() {
    let code = r#"
        fn count_down(n: number) -> number {
            if (n <= 0) {
                return 0;
            }
            return n + count_down(n - 1);
        }
        count_down(5)
    "#;
    assert_eval_number(code, 15.0);
}

#[test]
fn test_function_local_variable_isolation() {
    let code = r#"
        var global: number = 100;
        fn modify_local() -> number {
            let global: number = 50;
            return global;
        }
        let result: number = modify_local();
        result + global
    "#;
    assert_eval_number(code, 150.0);
}

#[test]
fn test_function_mutually_recursive() {
    let code = r#"
        fn is_even(n: number) -> bool {
            if (n == 0) {
                return true;
            }
            return is_odd(n - 1);
        }
        fn is_odd(n: number) -> bool {
            if (n == 0) {
                return false;
            }
            return is_even(n - 1);
        }
        is_even(4)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_fibonacci() {
    let code = r#"
        fn fib(n: number) -> number {
            if (n <= 1) {
                return n;
            }
            return fib(n - 1) + fib(n - 2);
        }
        fib(10)
    "#;
    assert_eval_number(code, 55.0);
}

#[test]
fn test_runtime_error_in_function_call() {
    let code = r#"
        fn divide(a: number, b: number) -> number {
            return a / b;
        }
        divide(10, 0)
    "#;
    assert_error_code(code, "AT0005");
}
