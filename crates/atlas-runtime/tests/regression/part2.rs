use super::common::*;

// ============================================================================

#[test]
fn regression_fibonacci() {
    let code = r#"
        fn fib(n: number) -> number {
            if (n <= 1) {
                return n;
            }
            return fib(n - 1) + fib(n - 2);
        }
        fib(10);
    "#;
    assert_eval_number(code, 55.0);
}

#[test]
fn regression_array_sum() {
    let code = r#"
        let arr: number[] = [1, 2, 3, 4, 5];
        let mut sum: number = 0;
        let mut i: number = 0;
        while (i < len(arr)) {
            sum = sum + arr[i];
            i = i + 1;
        }
        sum;
    "#;
    assert_eval_number(code, 15.0);
}

#[test]
fn regression_nested_function_calls() {
    let code = r#"
        fn double(x: number) -> number {
            return x * 2;
        }
        fn triple(x: number) -> number {
            return x * 3;
        }
        double(triple(5));
    "#;
    assert_eval_number(code, 30.0);
}

// Note: Scope shadowing is comprehensively tested in scope_shadowing_tests.rs

#[test]
fn regression_mixed_operations() {
    let code = r#"
        fn calculate(a: number, b: number) -> number {
            let sum: number = a + b;
            let product: number = a * b;
            if (sum > product) {
                return sum;
            } else {
                return product;
            }
        }
        calculate(5, 6);
    "#;
    assert_eval_number(code, 30.0); // product (5*6=30) > sum (5+6=11)
}

// ============================================================================
// STABILITY VERIFICATION TESTS (Phase 04)
