use super::common::*;

// ============================================================================

#[test]
fn regression_fibonacci() {
    let code = r#"
        fn fib(borrow n: number): number {
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
        let arr: []number = [1, 2, 3, 4, 5];
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
        fn double(borrow x: number): number {
            return x * 2;
        }
        fn triple(borrow x: number): number {
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
        fn calculate(borrow a: number, borrow b: number): number {
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
// H-304: Stack corruption when if-block with function call is skipped
// ============================================================================

#[test]
fn regression_h304_if_block_skipped_stack_corruption() {
    // When an if-block containing a function call is NOT executed (condition false),
    // subsequent function calls should still return correct values.
    let code = r#"
        fn get_val(): string {
            return "correct";
        }

        fn test(): string {
            let cond: bool = false;
            if cond {
                let x: string = get_val();
                console.log(x);
            }
            let y: string = get_val();
            return y;
        }

        test();
    "#;
    assert_eval_string(code, "correct");
}

#[test]
fn regression_h304_if_block_taken_then_skipped() {
    // Test that both paths work correctly in sequence
    let code = r#"
        fn get_num(): number {
            return 42;
        }

        fn test(): number {
            let mut result: number = 0;

            // First: condition true
            if true {
                let x: number = get_num();
                result = result + x;
            }

            // Second: condition false (this is where H-304 bug manifests)
            if false {
                let y: number = get_num();
                result = result + y;
            }

            // This call should return 42, not corrupted data
            let z: number = get_num();
            return result + z;
        }

        test();
    "#;
    assert_eval_number(code, 84.0); // 42 + 42
}

#[test]
fn regression_h304_nested_if_with_function_calls() {
    // Nested if blocks with function calls
    let code = r#"
        fn double(borrow n: number): number {
            return n * 2;
        }

        fn test(): number {
            let a: number = 5;

            if a > 0 {
                if false {
                    let inner: number = double(100);
                    console.log(inner.toString());
                }
                // Call after skipped inner if
                let b: number = double(a);
                return b;
            }
            return 0;
        }

        test();
    "#;
    assert_eval_number(code, 10.0);
}

#[test]
fn regression_h304_for_in_body_locals_cleanup() {
    // For-in loops must pop body-declared locals before looping back
    let code = r#"
        fn get_val(): string {
            return "ok";
        }

        fn test(): string {
            let arr: string[] = ["a", "b"];
            for item in arr {
                let x: string = get_val();
                console.log(x);
            }
            let y: string = get_val();
            return y;
        }

        test();
    "#;
    assert_eval_string(code, "ok");
}

#[test]
fn regression_h305_log_variable_name_allowed() {
    // User-defined variables should shadow deprecated bare globals like `log`
    let code = r#"
        fn get_log(): string {
            return "logdata";
        }

        fn test(): string {
            let log: string = get_log();
            return log;
        }

        test();
    "#;
    assert_eval_string(code, "logdata");
}

// ============================================================================
// STABILITY VERIFICATION TESTS (Phase 04)
