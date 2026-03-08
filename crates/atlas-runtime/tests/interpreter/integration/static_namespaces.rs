use super::*;

// ============================================================================
// B10-P07: Static namespaces Json, Math, Env — PascalCase.method() syntax
// Both interpreter and VM parity tested throughout.
// ============================================================================

// --- Json.parse() ---
// parseJSON returns Result<json>. Use unwrap() to extract the value.
// On invalid input it returns Result(Err(...)) — use isErr() to check.

#[test]
fn test_json_parse_object() {
    // parseJSON returns Ok(json) on valid input — unwrap should succeed
    let src = r#"let v = Json.parse("{\"x\": 1}"); unwrap(v); true;"#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

#[test]
fn test_json_parse_invalid_returns_err() {
    // parseJSON returns Result(Err(...)) on invalid input
    let src = r#"let v = Json.parse("not json"); isErr(v);"#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

// --- Json.stringify() ---

#[test]
fn test_json_stringify_number() {
    let src = r#"Json.stringify(42);"#;
    assert_eval_string(src, "42");
    assert_parity(src);
}

#[test]
fn test_json_stringify_string() {
    let src = r#"Json.stringify("hello");"#;
    assert_eval_string(src, "\"hello\"");
    assert_parity(src);
}

// --- Json.isValid() ---

#[test]
fn test_json_is_valid_true() {
    let src = r#"Json.isValid("{\"a\": 1}");"#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

#[test]
fn test_json_is_valid_false() {
    let src = r#"Json.isValid("not json");"#;
    assert_eval_bool(src, false);
    assert_parity(src);
}

// --- Math.sqrt() ---
// sqrt() returns Result<number>. Use unwrap() to extract.

#[test]
fn test_math_sqrt() {
    let src = r#"unwrap(Math.sqrt(9));"#;
    assert_eval_number(src, 3.0);
    assert_parity(src);
}

// --- Math.abs() ---

#[test]
fn test_math_abs_negative() {
    let src = r#"Math.abs(-5);"#;
    assert_eval_number(src, 5.0);
    assert_parity(src);
}

#[test]
fn test_math_abs_positive() {
    let src = r#"Math.abs(3);"#;
    assert_eval_number(src, 3.0);
    assert_parity(src);
}

// --- Math.floor() / Math.ceil() / Math.round() ---

#[test]
fn test_math_floor() {
    let src = r#"Math.floor(3.7);"#;
    assert_eval_number(src, 3.0);
    assert_parity(src);
}

#[test]
fn test_math_ceil() {
    let src = r#"Math.ceil(3.2);"#;
    assert_eval_number(src, 4.0);
    assert_parity(src);
}

#[test]
fn test_math_round_up() {
    let src = r#"Math.round(3.6);"#;
    assert_eval_number(src, 4.0);
    assert_parity(src);
}

#[test]
fn test_math_round_down() {
    let src = r#"Math.round(3.4);"#;
    assert_eval_number(src, 3.0);
    assert_parity(src);
}

// --- Math.min() / Math.max() ---

#[test]
fn test_math_min() {
    let src = r#"Math.min(3, 7);"#;
    assert_eval_number(src, 3.0);
    assert_parity(src);
}

#[test]
fn test_math_max() {
    let src = r#"Math.max(3, 7);"#;
    assert_eval_number(src, 7.0);
    assert_parity(src);
}

// --- Math.pow() ---

#[test]
fn test_math_pow() {
    let src = r#"Math.pow(2, 10);"#;
    assert_eval_number(src, 1024.0);
    assert_parity(src);
}

// --- Math.sign() ---

#[test]
fn test_math_sign_negative() {
    let src = r#"Math.sign(-42);"#;
    assert_eval_number(src, -1.0);
    assert_parity(src);
}

#[test]
fn test_math_sign_positive() {
    let src = r#"Math.sign(5);"#;
    assert_eval_number(src, 1.0);
    assert_parity(src);
}

#[test]
fn test_math_sign_zero() {
    let src = r#"Math.sign(0);"#;
    assert_eval_number(src, 0.0);
    assert_parity(src);
}

// --- Math.clamp() ---
// clamp() returns Result<number>. Use unwrap() to extract.

#[test]
fn test_math_clamp_within() {
    let src = r#"unwrap(Math.clamp(5, 0, 10));"#;
    assert_eval_number(src, 5.0);
    assert_parity(src);
}

#[test]
fn test_math_clamp_below() {
    let src = r#"unwrap(Math.clamp(-5, 0, 10));"#;
    assert_eval_number(src, 0.0);
    assert_parity(src);
}

#[test]
fn test_math_clamp_above() {
    let src = r#"unwrap(Math.clamp(15, 0, 10));"#;
    assert_eval_number(src, 10.0);
    assert_parity(src);
}

// --- Math.random() ---

#[test]
fn test_math_random_range() {
    // random() returns [0, 1) — verify it's a number in range (untyped binding avoids AT3001)
    let src = r#"
        let r = Math.random();
        r >= 0;
    "#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

// --- Env.get() ---
// getEnv requires env permissions. Use allow_all security context via run_interpreter/run_vm.

#[test]
fn test_env_get_known_var() {
    // PATH is always set — verify get() returns a non-null value
    // Use run_interpreter/run_vm (allow_all security context) instead of assert_eval_bool
    let src = r#"let v = Env.get("PATH"); v != null;"#;
    let result = run_interpreter(src);
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    assert!(
        result.unwrap().contains("Bool(true)"),
        "Expected Bool(true)"
    );
    assert_parity(src);
}

// ============================================================================
// End B10-P07 tests
// ============================================================================
