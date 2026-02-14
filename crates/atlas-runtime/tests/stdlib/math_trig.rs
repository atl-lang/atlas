//! Interpreter tests for trigonometric functions
//!
//! Tests: sin, cos, tan, asin, acos, atan

use crate::stdlib::eval_ok;
use atlas_runtime::value::Value;

// ============================================================================
// sin() tests
// ============================================================================

#[test]
fn test_sin_zero() {
    let result = eval_ok("sin(0);");
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_sin_pi_over_2() {
    let result = eval_ok("sin(PI / 2);");
    match result {
        Value::Number(n) => assert!((n - 1.0).abs() < 0.000001),
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_sin_pi() {
    let result = eval_ok("sin(PI);");
    match result {
        Value::Number(n) => assert!(n.abs() < 0.000001),
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_sin_nan() {
    let result = eval_ok("sin(0 / 0);");
    assert!(matches!(result, Value::Number(x) if x.is_nan()));
}

// ============================================================================
// cos() tests
// ============================================================================

#[test]
fn test_cos_zero() {
    let result = eval_ok("cos(0);");
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_cos_pi() {
    let result = eval_ok("cos(PI);");
    match result {
        Value::Number(n) => assert!((n - (-1.0)).abs() < 0.000001),
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_cos_pi_over_2() {
    let result = eval_ok("cos(PI / 2);");
    match result {
        Value::Number(n) => assert!(n.abs() < 0.000001),
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_cos_nan() {
    let result = eval_ok("cos(0 / 0);");
    assert!(matches!(result, Value::Number(x) if x.is_nan()));
}

// ============================================================================
// tan() tests
// ============================================================================

#[test]
fn test_tan_zero() {
    let result = eval_ok("tan(0);");
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_tan_pi_over_4() {
    let result = eval_ok("tan(PI / 4);");
    match result {
        Value::Number(n) => assert!((n - 1.0).abs() < 0.000001),
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_tan_nan() {
    let result = eval_ok("tan(0 / 0);");
    assert!(matches!(result, Value::Number(x) if x.is_nan()));
}

// ============================================================================
// asin() tests
// ============================================================================

#[test]
fn test_asin_zero() {
    let result = eval_ok("asin(0);");
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_asin_one() {
    let result = eval_ok("asin(1);");
    match result {
        Value::Number(n) => {
            let expected = std::f64::consts::PI / 2.0;
            assert!((n - expected).abs() < 0.000001);
        }
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_asin_out_of_range_returns_nan() {
    let result = eval_ok("asin(2);");
    assert!(matches!(result, Value::Number(x) if x.is_nan()));
}

#[test]
fn test_asin_negative_out_of_range_returns_nan() {
    let result = eval_ok("asin(-2);");
    assert!(matches!(result, Value::Number(x) if x.is_nan()));
}

// ============================================================================
// acos() tests
// ============================================================================

#[test]
fn test_acos_one() {
    let result = eval_ok("acos(1);");
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_acos_zero() {
    let result = eval_ok("acos(0);");
    match result {
        Value::Number(n) => {
            let expected = std::f64::consts::PI / 2.0;
            assert!((n - expected).abs() < 0.000001);
        }
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_acos_out_of_range_returns_nan() {
    let result = eval_ok("acos(2);");
    assert!(matches!(result, Value::Number(x) if x.is_nan()));
}

#[test]
fn test_acos_negative_out_of_range_returns_nan() {
    let result = eval_ok("acos(-2);");
    assert!(matches!(result, Value::Number(x) if x.is_nan()));
}

// ============================================================================
// atan() tests
// ============================================================================

#[test]
fn test_atan_zero() {
    let result = eval_ok("atan(0);");
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_atan_one() {
    let result = eval_ok("atan(1);");
    match result {
        Value::Number(n) => {
            let expected = std::f64::consts::PI / 4.0;
            assert!((n - expected).abs() < 0.000001);
        }
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_atan_infinity() {
    let result = eval_ok("atan(1 / 0);");
    match result {
        Value::Number(n) => {
            let expected = std::f64::consts::PI / 2.0;
            assert!((n - expected).abs() < 0.000001);
        }
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_atan_negative_infinity() {
    let result = eval_ok("atan(-1 / 0);");
    match result {
        Value::Number(n) => {
            let expected = -std::f64::consts::PI / 2.0;
            assert!((n - expected).abs() < 0.000001);
        }
        _ => panic!("Expected number"),
    }
}
