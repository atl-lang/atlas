use super::common::*;
use rstest::rstest;

// ============================================================================
// Literals
// ============================================================================

#[rstest]
#[case("42;", 42.0)]
#[case("2.5;", 2.5)]
#[case("0;", 0.0)]
#[case("-42;", -42.0)]
fn regression_number_literals(#[case] code: &str, #[case] expected: f64) {
    assert_eval_number(code, expected);
}

#[rstest]
#[case(r#""hello";"#, "hello")]
#[case(r#""world";"#, "world")]
#[case(r#""""#, "")]
fn regression_string_literals(#[case] code: &str, #[case] expected: &str) {
    assert_eval_string(code, expected);
}

#[rstest]
#[case("true;", true)]
#[case("false;", false)]
fn regression_bool_literals(#[case] code: &str, #[case] expected: bool) {
    assert_eval_bool(code, expected);
}

#[test]
fn regression_null_literal() {
    assert_eval_null("null");
}

// ============================================================================
// Arithmetic Operators
// ============================================================================

#[rstest]
#[case("1 + 2;", 3.0)]
#[case("5 - 3;", 2.0)]
#[case("4 * 3;", 12.0)]
#[case("10 / 2;", 5.0)]
#[case("10 % 3;", 1.0)]
#[case("2 + 3 * 4;", 14.0)] // Precedence
#[case("(2 + 3) * 4;", 20.0)] // Grouping
fn regression_arithmetic(#[case] code: &str, #[case] expected: f64) {
    assert_eval_number(code, expected);
}

// ============================================================================
// Comparison Operators
// ============================================================================

#[rstest]
#[case("1 < 2;", true)]
#[case("2 > 1;", true)]
#[case("1 <= 1;", true)]
#[case("2 >= 2;", true)]
#[case("1 == 1;", true)]
#[case("1 != 2;", true)]
fn regression_comparison(#[case] code: &str, #[case] expected: bool) {
    assert_eval_bool(code, expected);
}

// ============================================================================
// Logical Operators
// ============================================================================

#[rstest]
#[case("true && true;", true)]
#[case("true && false;", false)]
#[case("false || true;", true)]
#[case("false || false;", false)]
#[case("!true;", false)]
#[case("!false;", true)]
fn regression_logical(#[case] code: &str, #[case] expected: bool) {
    assert_eval_bool(code, expected);
}

// ============================================================================
// Variables - Let (Immutable)
// ============================================================================

#[rstest]
#[case("let x: number = 42; x;", 42.0)]
#[case("let x: number = 10; let y: number = 20; x + y;", 30.0)]
fn regression_let_variables(#[case] code: &str, #[case] expected: f64) {
    assert_eval_number(code, expected);
}

// ============================================================================
// Variables - Var (Mutable)
// ============================================================================

#[rstest]
#[case("let mut x: number = 10; x = 20; x;", 20.0)]
#[case("let mut x: number = 1; x = x + 1; x;", 2.0)]
fn regression_var_variables(#[case] code: &str, #[case] expected: f64) {
    assert_eval_number(code, expected);
}

// ============================================================================
// Functions
// ============================================================================

#[test]
fn regression_function_declaration_and_call() {
    let code = r#"
        fn add(borrow a: number, borrow b: number): number {
            return a + b;
        }
        add(2, 3);
    "#;
    assert_eval_number(code, 5.0);
}

#[test]
fn regression_function_recursion() {
    let code = r#"
        fn factorial(borrow n: number): number {
            if (n <= 1) {
                return 1;
            }
            return n * factorial(n - 1);
        }
        factorial(5);
    "#;
    assert_eval_number(code, 120.0);
}

#[test]
fn regression_function_local_variables() {
    let code = r#"
        fn compute(borrow x: number): number {
            let temp: number = x * 2;
            return temp + 1;
        }
        compute(5);
    "#;
    assert_eval_number(code, 11.0);
}

// ============================================================================
// Control Flow - If/Else
// ============================================================================

#[test]
fn regression_if_then() {
    let code = r#"
        fn test(): number {
            let x: number = 10;
            if (x > 5) {
                return x + 10;
            }
            0
        }
        test()
    "#;
    assert_eval_number(code, 20.0);
}

#[test]
fn regression_if_else() {
    let code = r#"
        fn test(): number {
            let x: number = 3;
            if (x > 5) {
                return 10;
            } else {
                return 20;
            }
        }
        test()
    "#;
    assert_eval_number(code, 20.0);
}

// ============================================================================
// Control Flow - While
// ============================================================================

#[test]
fn regression_while_loop() {
    let code = r#"
        let mut i: number = 0;
        let mut sum: number = 0;
        while (i < 5) {
            sum = sum + i;
            i = i + 1;
        }
        sum;
    "#;
    assert_eval_number(code, 10.0); // 0+1+2+3+4 = 10
}

#[test]
fn regression_while_with_break() {
    let code = r#"
        let mut i: number = 0;
        while (i < 10) {
            if (i == 5) {
                break;
            }
            i = i + 1;
        }
        i;
    "#;
    assert_eval_number(code, 5.0);
}

#[test]
fn regression_while_with_continue() {
    let code = r#"
        let mut i: number = 0;
        let mut sum: number = 0;
        while (i < 5) {
            i = i + 1;
            if (i == 3) {
                continue;
            }
            sum = sum + i;
        }
        sum;
    "#;
    assert_eval_number(code, 12.0); // 1+2+4+5 = 12 (skips 3)
}

// ============================================================================
// Arrays
// ============================================================================

#[test]
fn regression_array_literal() {
    let code = r#"
        let arr: []number = [1, 2, 3];
        len(arr);
    "#;
    assert_eval_number(code, 3.0);
}

#[test]
fn regression_array_indexing() {
    let code = r#"
        let arr: []number = [10, 20, 30];
        arr[1];
    "#;
    assert_eval_number(code, 20.0);
}

#[test]
fn regression_array_mutation() {
    let code = r#"
        let mut arr: []number = [1, 2, 3];
        arr[0] = 99;
        arr[0];
    "#;
    assert_eval_number(code, 99.0);
}

#[test]
fn regression_nested_arrays() {
    let code = r#"
        let matrix: [][]number = [[1, 2], [3, 4]];
        matrix[1][0];
    "#;
    assert_eval_number(code, 3.0);
}

// ============================================================================
// String Operations
// ============================================================================

#[test]
fn regression_string_concatenation() {
    let code = r#"
        let s1: string = "hello";
        let s2: string = "world";
        s1 + s2;
    "#;
    assert_eval_string(code, "helloworld");
}

// Note: String indexing is not yet supported in Atlas
// TODO: Enable when typechecker supports string indexing

// ============================================================================
// Standard Library Functions
// ============================================================================

#[test]
fn regression_stdlib_len() {
    let code = r#"len("hello");"#;
    assert_eval_number(code, 5.0);
}

#[test]
fn regression_stdlib_print() {
    // print() returns null
    let code = r#"print("test");"#;
    assert_eval_null(code);
}

#[test]
fn regression_stdlib_str() {
    let code = r#"str(42);"#;
    assert_eval_string(code, "42");
}

// ============================================================================
// Type Errors
// ============================================================================

#[rstest]
#[case(r#"let x: number = "hello";"#, "AT3001")] // Type mismatch
#[case(r#"unknown_var;"#, "AT2002")] // Unknown symbol
#[case(r#"let x: number = 1; x = 2;"#, "AT3003")] // Invalid assignment (let is immutable)
fn regression_type_errors(#[case] code: &str, #[case] expected_code: &str) {
    assert_error_code(code, expected_code);
}

// ============================================================================
// Runtime Errors
// ============================================================================

#[rstest]
#[case("1 / 0;", "AT0005")] // Divide by zero
#[case("let arr: []number = [1, 2]; arr[10];", "AT0006")] // Out of bounds
fn regression_runtime_errors(#[case] code: &str, #[case] expected_code: &str) {
    assert_error_code(code, expected_code);
}

// ============================================================================
// Complex Integration Tests
