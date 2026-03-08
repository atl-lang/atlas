use super::common::*;

// ─── Edge Case Tests ──────────────────────────────────────────────────────────

#[test]
fn stability_edge_empty_string_literal() {
    assert_eval_string("\"\";", "");
}

#[test]
fn stability_edge_empty_array() {
    // Empty array literals are allowed with explicit type context.
    assert_eval_number("let arr: []number = []; len(arr);", 0.0);
}

#[test]
fn stability_edge_zero_value() {
    assert_eval_number("0;", 0.0);
}

#[test]
fn stability_edge_negative_zero() {
    // -0.0 is a valid float; should evaluate without error.
    assert_no_error("-0.0;");
}

#[test]
fn stability_edge_large_integer() {
    // Large numbers within float64 range should work fine.
    assert_eval_number("1000000.0;", 1_000_000.0);
}

#[test]
fn stability_edge_float_precision() {
    // Basic float precision: 0.1 + 0.2 should produce a number (not crash).
    assert_no_error("0.1 + 0.2;");
}

#[test]
fn stability_edge_negative_number() {
    assert_eval_number("-42;", -42.0);
}

#[test]
fn stability_edge_null_literal() {
    assert_eval_null("null;");
}

#[test]
fn stability_edge_boolean_true() {
    assert_eval_bool("true;", true);
}

#[test]
fn stability_edge_boolean_false() {
    assert_eval_bool("false;", false);
}

#[test]
fn stability_edge_single_char_string() {
    assert_eval_string(r#""a";"#, "a");
}

#[test]
fn stability_edge_deeply_nested_arithmetic() {
    // 10 levels of nesting — should not overflow the stack.
    assert_no_error("((((((((((1 + 2) + 3) + 4) + 5) + 6) + 7) + 8) + 9) + 10) + 0);");
}

#[test]
fn stability_edge_chained_comparisons() {
    assert_no_error("1 < 2;");
    assert_no_error("2 > 1;");
    assert_no_error("1 == 1;");
    assert_no_error("1 != 2;");
}

#[test]
fn stability_edge_not_operator() {
    assert_eval_bool("!true;", false);
    assert_eval_bool("!false;", true);
}

#[test]
fn stability_edge_string_with_spaces() {
    assert_eval_string(r#""hello world";"#, "hello world");
}

#[test]
fn stability_edge_multiple_statements() {
    // Programs with many statements should not crash.
    assert_no_error(
        r#"
        let a: number = 1;
        let b: number = 2;
        let c: number = 3;
        let d: number = 4;
        let e: number = 5;
        a + b + c + d + e;
    "#,
    );
}

#[test]
fn stability_edge_string_escape_sequences() {
    // Strings with standard escape-adjacent characters.
    assert_no_error(r#""tab\there";"#);
}

#[test]
fn stability_edge_nested_array_access() {
    assert_eval_number("let arr: []number = [10, 20, 30]; arr[0];", 10.0);
    assert_eval_number("let arr: []number = [10, 20, 30]; arr[2];", 30.0);
}

#[test]
fn stability_edge_function_with_no_return_value() {
    // Void functions must not crash on call.
    assert_no_error("fn greet() -> null { } greet();");
}
