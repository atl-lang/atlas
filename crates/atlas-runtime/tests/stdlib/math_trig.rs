//! Interpreter tests for trigonometric functions
//!
//! Tests: Math.sin, Math.cos, Math.tan, Math.asin, Math.acos, Math.atan, Math.atan2

use crate::stdlib::eval_ok;
use atlas_runtime::value::Value;

// ============================================================================
// Math.sin() tests
// ============================================================================

#[test]
fn test_sin_zero() {
    let result = eval_ok("Math.sin(0);");
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_sin_pi_over_2() {
    let result = eval_ok("Math.sin(Math.PI / 2);");
    match result {
        Value::Number(n) => assert!((n - 1.0).abs() < 0.000001),
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_sin_pi() {
    let result = eval_ok("Math.sin(Math.PI);");
    match result {
        Value::Number(n) => assert!(n.abs() < 0.000001),
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_sin_nan() {
    let result = eval_ok("Math.sin(0 / 0);");
    assert!(matches!(result, Value::Number(x) if x.is_nan()));
}

// ============================================================================
// Math.cos() tests
// ============================================================================

#[test]
fn test_cos_zero() {
    let result = eval_ok("Math.cos(0);");
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_cos_pi() {
    let result = eval_ok("Math.cos(Math.PI);");
    match result {
        Value::Number(n) => assert!((n - (-1.0)).abs() < 0.000001),
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_cos_pi_over_2() {
    let result = eval_ok("Math.cos(Math.PI / 2);");
    match result {
        Value::Number(n) => assert!(n.abs() < 0.000001),
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_cos_nan() {
    let result = eval_ok("Math.cos(0 / 0);");
    assert!(matches!(result, Value::Number(x) if x.is_nan()));
}

// ============================================================================
// Math.tan() tests
// ============================================================================

#[test]
fn test_tan_zero() {
    let result = eval_ok("Math.tan(0);");
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_tan_pi_over_4() {
    let result = eval_ok("Math.tan(Math.PI / 4);");
    match result {
        Value::Number(n) => assert!((n - 1.0).abs() < 0.000001),
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_tan_nan() {
    let result = eval_ok("Math.tan(0 / 0);");
    assert!(matches!(result, Value::Number(x) if x.is_nan()));
}

// ============================================================================
// Math.asin() tests — returns Result<number, string>
// ============================================================================

#[test]
fn test_asin_zero() {
    let result = eval_ok("unwrap(Math.asin(0));");
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_asin_one() {
    let result = eval_ok("unwrap(Math.asin(1));");
    match result {
        Value::Number(n) => {
            let expected = std::f64::consts::PI / 2.0;
            assert!((n - expected).abs() < 0.000001);
        }
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_asin_out_of_range_returns_err() {
    let result = eval_ok("isErr(Math.asin(2));");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_asin_negative_out_of_range_returns_err() {
    let result = eval_ok("isErr(Math.asin(-2));");
    assert_eq!(result, Value::Bool(true));
}

// ============================================================================
// Math.acos() tests — returns Result<number, string>
// ============================================================================

#[test]
fn test_acos_one() {
    let result = eval_ok("unwrap(Math.acos(1));");
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_acos_zero() {
    let result = eval_ok("unwrap(Math.acos(0));");
    match result {
        Value::Number(n) => {
            let expected = std::f64::consts::PI / 2.0;
            assert!((n - expected).abs() < 0.000001);
        }
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_acos_out_of_range_returns_err() {
    let result = eval_ok("isErr(Math.acos(2));");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_acos_negative_out_of_range_returns_err() {
    let result = eval_ok("isErr(Math.acos(-2));");
    assert_eq!(result, Value::Bool(true));
}

// ============================================================================
// Math.atan() tests
// ============================================================================

#[test]
fn test_atan_zero() {
    let result = eval_ok("Math.atan(0);");
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_atan_one() {
    let result = eval_ok("Math.atan(1);");
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
    let result = eval_ok("Math.atan(1 / 0);");
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
    let result = eval_ok("Math.atan(-1 / 0);");
    match result {
        Value::Number(n) => {
            let expected = -std::f64::consts::PI / 2.0;
            assert!((n - expected).abs() < 0.000001);
        }
        _ => panic!("Expected number"),
    }
}

// ============================================================================
// Math.atan2() tests
// ============================================================================

#[test]
fn test_atan2_zero_zero() {
    let result = eval_ok("Math.atan2(0, 0);");
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_atan2_one_one() {
    let result = eval_ok("Math.atan2(1, 1);");
    match result {
        Value::Number(n) => {
            let expected = std::f64::consts::PI / 4.0;
            assert!((n - expected).abs() < 0.000001);
        }
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_atan2_y_zero_negative_x() {
    let result = eval_ok("Math.atan2(0, -1);");
    match result {
        Value::Number(n) => {
            assert!((n - std::f64::consts::PI).abs() < 0.000001);
        }
        _ => panic!("Expected number"),
    }
}
