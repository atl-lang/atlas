//! Interpreter tests for basic math functions
//!
//! Tests: abs, floor, ceil, round, min, max, sqrt, pow, log

use crate::stdlib::eval_ok;
use atlas_runtime::value::Value;

// ============================================================================
// abs() tests
// ============================================================================

#[test]
fn test_abs_positive() {
    let result = eval_ok("abs(42);");
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_abs_negative() {
    let result = eval_ok("abs(-42);");
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_abs_zero() {
    let result = eval_ok("abs(0);");
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_abs_infinity() {
    let result = eval_ok("abs(1 / 0);");
    assert!(matches!(result, Value::Number(x) if x.is_infinite() && x.is_sign_positive()));
}

#[test]
fn test_abs_nan() {
    let result = eval_ok("abs(0 / 0);");
    assert!(matches!(result, Value::Number(x) if x.is_nan()));
}

// ============================================================================
// floor() tests
// ============================================================================

#[test]
fn test_floor_positive() {
    let result = eval_ok("floor(4.7);");
    assert_eq!(result, Value::Number(4.0));
}

#[test]
fn test_floor_negative() {
    let result = eval_ok("floor(-4.3);");
    assert_eq!(result, Value::Number(-5.0));
}

#[test]
fn test_floor_integer() {
    let result = eval_ok("floor(5.0);");
    assert_eq!(result, Value::Number(5.0));
}

#[test]
fn test_floor_nan() {
    let result = eval_ok("floor(0 / 0);");
    assert!(matches!(result, Value::Number(x) if x.is_nan()));
}

// ============================================================================
// ceil() tests
// ============================================================================

#[test]
fn test_ceil_positive() {
    let result = eval_ok("ceil(4.3);");
    assert_eq!(result, Value::Number(5.0));
}

#[test]
fn test_ceil_negative() {
    let result = eval_ok("ceil(-4.7);");
    assert_eq!(result, Value::Number(-4.0));
}

#[test]
fn test_ceil_integer() {
    let result = eval_ok("ceil(5.0);");
    assert_eq!(result, Value::Number(5.0));
}

#[test]
fn test_ceil_nan() {
    let result = eval_ok("ceil(0 / 0);");
    assert!(matches!(result, Value::Number(x) if x.is_nan()));
}

// ============================================================================
// round() tests
// ============================================================================

#[test]
fn test_round_up() {
    let result = eval_ok("round(4.6);");
    assert_eq!(result, Value::Number(5.0));
}

#[test]
fn test_round_down() {
    let result = eval_ok("round(4.4);");
    assert_eq!(result, Value::Number(4.0));
}

#[test]
fn test_round_ties_to_even_2_5() {
    let result = eval_ok("round(2.5);");
    assert_eq!(result, Value::Number(2.0)); // Ties to even
}

#[test]
fn test_round_ties_to_even_3_5() {
    let result = eval_ok("round(3.5);");
    assert_eq!(result, Value::Number(4.0)); // Ties to even
}

#[test]
fn test_round_negative() {
    let result = eval_ok("round(-4.6);");
    assert_eq!(result, Value::Number(-5.0));
}

#[test]
fn test_round_nan() {
    let result = eval_ok("round(0 / 0);");
    assert!(matches!(result, Value::Number(x) if x.is_nan()));
}

// ============================================================================
// min() / max() tests
// ============================================================================

#[test]
fn test_min_normal() {
    let result = eval_ok("min(5, 3);");
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn test_min_equal() {
    let result = eval_ok("min(5, 5);");
    assert_eq!(result, Value::Number(5.0));
}

#[test]
fn test_min_negative() {
    let result = eval_ok("min(-5, -3);");
    assert_eq!(result, Value::Number(-5.0));
}

#[test]
fn test_min_nan_propagation() {
    let result = eval_ok("min(5, 0 / 0);");
    assert!(matches!(result, Value::Number(x) if x.is_nan()));
}

#[test]
fn test_max_normal() {
    let result = eval_ok("max(5, 3);");
    assert_eq!(result, Value::Number(5.0));
}

#[test]
fn test_max_equal() {
    let result = eval_ok("max(5, 5);");
    assert_eq!(result, Value::Number(5.0));
}

#[test]
fn test_max_negative() {
    let result = eval_ok("max(-5, -3);");
    assert_eq!(result, Value::Number(-3.0));
}

#[test]
fn test_max_nan_propagation() {
    let result = eval_ok("max(5, 0 / 0);");
    assert!(matches!(result, Value::Number(x) if x.is_nan()));
}

// ============================================================================
// sqrt() tests
// ============================================================================

#[test]
fn test_sqrt_perfect_square() {
    let result = eval_ok("sqrt(16);");
    assert_eq!(result, Value::Number(4.0));
}

#[test]
fn test_sqrt_zero() {
    let result = eval_ok("sqrt(0);");
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_sqrt_negative_returns_nan() {
    let result = eval_ok("sqrt(-4);");
    assert!(matches!(result, Value::Number(x) if x.is_nan()));
}

#[test]
fn test_sqrt_infinity() {
    let result = eval_ok("sqrt(1 / 0);");
    assert!(matches!(result, Value::Number(x) if x.is_infinite()));
}

// ============================================================================
// pow() tests
// ============================================================================

#[test]
fn test_pow_positive_exponent() {
    let result = eval_ok("pow(2, 3);");
    assert_eq!(result, Value::Number(8.0));
}

#[test]
fn test_pow_zero_exponent() {
    let result = eval_ok("pow(5, 0);");
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_pow_negative_exponent() {
    let result = eval_ok("pow(2, -2);");
    assert_eq!(result, Value::Number(0.25));
}

#[test]
fn test_pow_fractional_exponent() {
    let result = eval_ok("pow(4, 0.5);");
    assert_eq!(result, Value::Number(2.0));
}

// ============================================================================
// log() tests
// ============================================================================

#[test]
fn test_log_e() {
    let result = eval_ok("log(E);");
    let expected = 1.0;
    match result {
        Value::Number(n) => assert!((n - expected).abs() < 0.000001),
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_log_one() {
    let result = eval_ok("log(1);");
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_log_negative_returns_nan() {
    let result = eval_ok("log(-1);");
    assert!(matches!(result, Value::Number(x) if x.is_nan()));
}

#[test]
fn test_log_zero_returns_neg_infinity() {
    let result = eval_ok("log(0);");
    assert!(matches!(result, Value::Number(x) if x.is_infinite() && x.is_sign_negative()));
}
