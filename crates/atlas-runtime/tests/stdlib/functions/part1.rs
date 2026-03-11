use super::*;

// ============================================================================

// First-class functions tests for interpreter
//
// Tests that functions can be:
// - Stored in variables
// - Passed as arguments
// - Returned from functions
// - Called through variables
//
// Note: Some tests currently trigger false-positive "unused parameter" warnings.
// This is a pre-existing bug in the warning system (AT2001) - it doesn't recognize
// parameters passed to function-valued variables as "used". The actual first-class
// function functionality works correctly. This will be fixed in a separate phase.

// ============================================================================
// Category 1: Variable Storage (20 tests)
// ============================================================================

#[test]
fn test_store_function_in_let() {
    let source = r#"
        fn double(borrow x: number): number { return x * 2; }
        let f = double;
        f(5);
    "#;
    assert_eval_number(source, 10.0);
}

#[test]
fn test_store_function_in_var() {
    let source = r#"
        fn triple(borrow x: number): number { return x * 3; }
        let mut f = triple;
        f(4);
    "#;
    assert_eval_number(source, 12.0);
}

#[test]
fn test_reassign_function_variable() {
    let source = r#"
        fn add(borrow a: number, borrow b: number): number { return a + b; }
        fn mul(borrow a: number, borrow b: number): number { return a * b; }
        let mut f = add;
        let x = f(2, 3);
        f = mul;
        let y = f(2, 3);
        y;
    "#;
    assert_eval_number(source, 6.0);
}

#[test]
fn test_console_log_returns_null() {
    let source = r#"
        console.log("test");
    "#;
    // console.log returns null
    assert_eval_null(source);
}

#[test]
fn test_store_builtin_len() {
    let source = r#"
        let l = len;
        l("hello");
    "#;
    assert_eval_number(source, 5.0);
}

#[test]
fn test_store_builtin_str() {
    let source = r#"
        let s = str;
        s(42);
    "#;
    assert_eval_string(source, "42");
}

#[test]
fn test_multiple_function_variables() {
    let source = r#"
        fn add(borrow a: number, borrow b: number): number { return a + b; }
        fn sub(borrow a: number, borrow b: number): number { return a - b; }
        let f1 = add;
        let f2 = sub;
        f1(10, 3) + f2(10, 3);
    "#;
    assert_eval_number(source, 20.0);
}

#[test]
fn test_function_variable_with_same_name() {
    let source = r#"
        fn double(borrow x: number): number { return x * 2; }
        let double = double;
        double(5);
    "#;
    assert_eval_number(source, 10.0);
}

#[test]
fn test_function_variable_in_block() {
    let source = r#"
        fn square(borrow x: number): number { return x * x; }
        {
            let f = square;
            f(3);
        }
    "#;
    assert_eval_number(source, 9.0);
}

#[test]
fn test_function_variable_shadowing() {
    let source = r#"
        fn add(borrow a: number, borrow b: number): number { return a + b; }
        fn mul(borrow a: number, borrow b: number): number { return a * b; }
        let f = add;
        {
            let f = mul;
            f(2, 3);
        }
    "#;
    assert_eval_number(source, 6.0);
}

// ============================================================================
// Category 2: Function Parameters (25 tests)
// ============================================================================

#[test]
fn test_pass_function_as_argument() {
    let source = r#"
        fn apply(borrow f: (number): number, x: number): number {
            return f(x);
        }
        fn double(borrow n: number): number { return n * 2; }
        apply(double, 5);
    "#;
    assert_eval_number(source, 10.0);
}

#[test]
fn test_pass_builtin_as_argument() {
    let source = r#"
        fn applyStr(borrow f: (number): string, x: number): string {
            return f(x);
        }
        applyStr(str, 42);
    "#;
    assert_eval_string(source, "42");
}

#[test]
fn test_pass_function_through_variable() {
    let source = r#"
        fn apply(borrow f: (number): number, x: number): number {
            return f(x);
        }
        fn triple(borrow n: number): number { return n * 3; }
        let myFunc = triple;
        apply(myFunc, 4);
    "#;
    assert_eval_number(source, 12.0);
}

#[test]
fn test_multiple_function_parameters() {
    let source = r#"
        fn compose(
            borrow f: (number): number,
            g: (number): number,
            x: number
        ): number {
            return f(g(x));
        }
        fn double(borrow n: number): number { return n * 2; }
        fn inc(borrow n: number): number { return n + 1; }
        compose(double, inc, 5);
    "#;
    assert_eval_number(source, 12.0);
}

#[test]
fn test_function_parameter_called_multiple_times() {
    let source = r#"
        fn applyTwice(borrow f: (number): number, x: number): number {
            return f(f(x));
        }
        fn double(borrow n: number): number { return n * 2; }
        applyTwice(double, 3);
    "#;
    assert_eval_number(source, 12.0);
}

#[test]
fn test_function_parameter_with_string() {
    let source = r#"
        fn apply(borrow f: (string): number, s: string): number {
            return f(s);
        }
        apply(len, "hello");
    "#;
    assert_eval_number(source, 5.0);
}

#[test]
fn test_function_parameter_two_args() {
    let source = r#"
        fn applyBinary(
            borrow f: (number, number): number,
            a: number,
            b: number
        ): number {
            return f(a, b);
        }
        fn add(borrow x: number, borrow y: number): number { return x + y; }
        applyBinary(add, 10, 20);
    "#;
    assert_eval_number(source, 30.0);
}

#[test]
fn test_conditional_function_call() {
    let source = r#"
        fn apply(borrow f: (number): number, x: number, flag: bool): number {
            if (flag) {
                return f(x);
            }
            return x;
        }
        fn double(borrow n: number): number { return n * 2; }
        apply(double, 5, true);
    "#;
    assert_eval_number(source, 10.0);
}

// ============================================================================
// Category 3: Function Returns (15 tests)
