use super::common::*;

// ─── Stdlib Verification ─────────────────────────────────────────────────────

#[test]
fn milestone_stdlib_len_function() {
    assert_eval_number("let a: number[] = [1, 2, 3]; len(a);", 3.0);
}

#[test]
fn milestone_stdlib_console_log_function() {
    // console.log should not crash; return value is null.
    assert_no_error(r#"console.log("hello milestone");"#);
}

#[test]
fn milestone_stdlib_math_abs() {
    assert_eval_number("Math.abs(-5.0);", 5.0);
}

#[test]
fn milestone_stdlib_math_max() {
    assert_eval_number("Math.max(3.0, 7.0);", 7.0);
}

#[test]
fn milestone_stdlib_math_min() {
    assert_eval_number("Math.min(3.0, 7.0);", 3.0);
}

#[test]
fn milestone_stdlib_string_to_upper() {
    assert_eval_string(r#"toUpperCase("hello");"#, "HELLO");
}

#[test]
fn milestone_stdlib_string_to_lower() {
    assert_eval_string(r#"toLowerCase("HELLO");"#, "hello");
}

#[test]
fn milestone_stdlib_string_contains() {
    assert_eval_bool(r#"is_some(indexOf("hello world", "world"));"#, true);
    assert_eval_bool(r#"is_some(indexOf("hello world", "xyz"));"#, false);
}

#[test]
fn milestone_stdlib_string_length() {
    assert_eval_number(r#"len("hello");"#, 5.0);
}

#[test]
fn milestone_stdlib_type_conversion_to_string() {
    assert_eval_string("toString(42.0);", "42");
}

// ─── Error Code Verification ──────────────────────────────────────────────────

#[test]
fn milestone_error_codes_stable() {
    // These error codes must remain stable across versions.
    assert_error_code("1 / 0;", "AT0005"); // DivisionByZero
    assert_error_code("let a: number[] = [1]; a[5];", "AT0006"); // IndexOutOfBounds
}

#[test]
fn milestone_lex_error_unterminated_string() {
    assert_has_error(r#""unterminated"#);
}
