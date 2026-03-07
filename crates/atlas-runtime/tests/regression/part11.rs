use super::common::*;

// ============================================================================
// B9: Template string interpolation — ${ } syntax
// ============================================================================

#[test]
fn test_b9_template_basic_interpolation() {
    let code = r#"
        let name = "world";
        `hello ${name}!`
    "#;
    assert_eval_string(code, "hello world!");
}

#[test]
fn test_b9_template_dollar_not_leaked() {
    // Previously: `hello ${name}` produced "hello $world" — $ leaked into output
    let code = r#"
        let x = 42;
        `value is ${x} end`
    "#;
    assert_eval_string(code, "value is 42 end");
}

#[test]
fn test_b9_template_at_start() {
    let code = r#"
        let x = 7;
        `${x} is the answer`
    "#;
    assert_eval_string(code, "7 is the answer");
}

#[test]
fn test_b9_template_multiple_interpolations() {
    let code = r#"
        let a = "foo";
        let b = "bar";
        `${a} and ${b}`
    "#;
    assert_eval_string(code, "foo and bar");
}

#[test]
fn test_b9_template_expression_interpolation() {
    let code = r#"
        let x = 3;
        `result: ${x * 2 + 1}`
    "#;
    assert_eval_string(code, "result: 7");
}

#[test]
fn test_b9_template_no_interpolation() {
    let code = r#"
        `plain text`
    "#;
    assert_eval_string(code, "plain text");
}

// ============================================================================
// B9: Implicit returns — tail expression as function return value
// ============================================================================

#[test]
fn test_b9_implicit_return_simple() {
    let code = r#"
        fn double(x: number) -> number {
            x * 2
        }
        double(5)
    "#;
    assert_eval_number(code, 10.0);
}

#[test]
fn test_b9_implicit_return_no_false_unused_warning() {
    // Params used only in tail_expr must not emit AT2001 unused warnings
    // This test just verifies it compiles and runs without errors
    let code = r#"
        fn add(a: number, b: number) -> number {
            a + b
        }
        add(3, 4)
    "#;
    assert_eval_number(code, 7.0);
}

#[test]
fn test_b9_implicit_return_string() {
    let code = r#"
        fn greet(name: string) -> string {
            `hello ${name}`
        }
        greet("atlas")
    "#;
    assert_eval_string(code, "hello atlas");
}

#[test]
fn test_b9_implicit_return_explicit_still_works() {
    // explicit return must still work alongside implicit
    let code = r#"
        fn abs(x: number) -> number {
            if x < 0 {
                return x * -1;
            }
            x
        }
        abs(-3) + abs(4)
    "#;
    assert_eval_number(code, 7.0);
}
