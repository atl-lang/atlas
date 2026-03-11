use super::common::*;
use atlas_runtime::Atlas;

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
        fn double(borrow x: number): number {
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
        fn add(borrow a: number, borrow b: number): number {
            a + b
        }
        add(3, 4)
    "#;
    assert_eval_number(code, 7.0);
}

#[test]
fn test_b9_implicit_return_string() {
    let code = r#"
        fn greet(borrow name: string): string {
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
        fn my_abs(borrow x: number): number {
            if x < 0 {
                return x * -1;
            }
            x
        }
        my_abs(-3) + my_abs(4)
    "#;
    assert_eval_number(code, 7.0);
}

// ============================================================================
// H-194: AT3001 type mismatch uses dual-span (declaration + value site)
// ============================================================================

#[test]
fn test_h194_type_mismatch_has_related_location() {
    // H-194: AT3001 should show two spans like Rust E0308:
    //   let x: number = "hello";
    //          ------   ^^^^^^^
    //          |        found string
    //          expected due to this type annotation
    //
    // The diagnostic must have a related location pointing at the type annotation.
    let code = r#"let x: number = "hello";"#;
    let runtime = Atlas::new();
    let diags = runtime.eval(code).expect_err("expected AT3001");
    let at3001 = diags
        .iter()
        .find(|d| d.code == "AT3001")
        .expect("expected AT3001 diagnostic");
    assert!(
        !at3001.related.is_empty(),
        "AT3001 should have a related location pointing at the type annotation. Got: {:?}",
        at3001
    );
    let rel = &at3001.related[0];
    assert!(
        rel.message.contains("expected") || rel.message.contains("annotation"),
        "Related location message should describe expected type, got: {:?}",
        rel.message
    );
}

// ============================================================================
// H-193: Error pipeline collects diagnostics across all phases
// ============================================================================

#[test]
fn test_h193_error_pipeline_collects_all_phase_errors() {
    // Bug H-193: eval_source returned early on binder errors, so typecheck
    // never ran. A file with errors in both phases must report ALL of them.
    //
    // AT2003 = duplicate function (binder)
    // AT3001 = type annotation mismatch (typechecker)
    let code = r#"
        fn greet(): number { 1; }
        fn greet(): number { 2; }
        let x: number = "hello";
    "#;
    let runtime = Atlas::new();
    let diags = runtime
        .eval(code)
        .expect_err("expected errors from both phases");
    let codes: Vec<&str> = diags.iter().map(|d| d.code.as_str()).collect();
    assert!(
        codes.iter().any(|c| *c == "AT2003"),
        "Expected AT2003 (duplicate function) in diagnostics, got: {:?}",
        codes
    );
    assert!(
        codes.iter().any(|c| *c == "AT3001"),
        "Expected AT3001 (type mismatch) in diagnostics, got: {:?}",
        codes
    );
}

// ============================================================================
// H-260: number and bool instance methods — TypeScript parity (D-021)
// ============================================================================

#[test]
fn test_h260_number_tostring() {
    assert_eval_string(
        r#"
fn main(): string {
    let n: number = 42.0;
    return n.toString();
}
"#,
        "42",
    );
}

#[test]
fn test_h260_number_tofixed() {
    assert_eval_string(
        r#"
fn main(): string {
    let n: number = 3.14159;
    return n.toFixed(2);
}
"#,
        "3.14",
    );
}

#[test]
fn test_h260_number_toint() {
    assert_eval_number(
        r#"
fn main(): number {
    let n: number = 7.9;
    return n.toInt();
}
"#,
        7.0,
    );
}

#[test]
fn test_h260_bool_tostring_true() {
    assert_eval_string(
        r#"
fn main(): string {
    let b: bool = true;
    return b.toString();
}
"#,
        "true",
    );
}

#[test]
fn test_h260_bool_tostring_false() {
    assert_eval_string(
        r#"
fn main(): string {
    let b: bool = false;
    return b.toString();
}
"#,
        "false",
    );
}

// ============================================================================
// H-261: \x hex and \uXXXX unicode escape sequences in string literals
// ============================================================================

#[test]
fn test_h261_hex_escape_in_string() {
    assert_eval_string(r#"let s = "\x1b[31m"; s"#, "\x1b[31m");
}

#[test]
fn test_h261_unicode_escape_in_string() {
    assert_eval_string(r#"let s = "\u0041"; s"#, "A");
}

#[test]
fn test_h261_hex_escape_in_template_string() {
    assert_eval_string(r#"let s = `\x1b[31m`; s"#, "\x1b[31m");
}

#[test]
fn test_h261_unicode_escape_four_digits() {
    assert_eval_string(r#"let s = "\u2764"; s"#, "❤");
}
