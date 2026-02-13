//! Boolean operations and comparison tests

use rstest::rstest;

use crate::common::*;

#[rstest]
#[case("5 == 5", true)]
#[case("5 != 3", true)]
#[case("3 < 5", true)]
#[case("5 > 3", true)]
#[case("true && true", true)]
#[case("false || true", true)]
#[case("!false", true)]
#[case("true && false", false)]
#[case("false || false", false)]
#[case("!true", false)]
fn test_comparison_and_boolean_ops(#[case] code: &str, #[case] expected: bool) {
    assert_eval_bool(code, expected);
}

#[test]
fn test_variable_declaration_and_use() {
    let code = r#"
        let x: number = 42;
        x
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_variable_assignment() {
    let code = r#"
        var x: number = 10;
        x = 20;
        x
    "#;
    assert_eval_number(code, 20.0);
}

#[test]
fn test_variable_arithmetic() {
    let code = r#"
        let a: number = 5;
        let b: number = 3;
        a + b
    "#;
    assert_eval_number(code, 8.0);
}

#[test]
fn test_block_scope() {
    let code = r#"
        let x: number = 1;
        if (true) {
            let x: number = 2;
            x;
        }
    "#;
    assert_eval_number(code, 2.0);
}

#[test]
fn test_function_scope() {
    let code = r#"
        var x: number = 10;
        fn foo(x: number) -> number {
            return x + 1;
        }
        foo(5)
    "#;
    assert_eval_number(code, 6.0);
}
