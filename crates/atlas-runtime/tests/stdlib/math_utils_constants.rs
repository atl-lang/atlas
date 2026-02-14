//! Interpreter tests for math utilities and constants
//!
//! Tests: clamp, sign, random + PI, E, SQRT2, LN2, LN10

use crate::stdlib::eval_ok;
use atlas_runtime::value::Value;

// ============================================================================
// clamp() tests
// ============================================================================

#[test]
fn test_clamp_within_range() {
    let result = eval_ok("clamp(5, 1, 10);");
    assert_eq!(result, Value::Number(5.0));
}

#[test]
fn test_clamp_below_min() {
    let result = eval_ok("clamp(-5, 1, 10);");
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_clamp_above_max() {
    let result = eval_ok("clamp(15, 1, 10);");
    assert_eq!(result, Value::Number(10.0));
}

#[test]
fn test_clamp_at_min() {
    let result = eval_ok("clamp(1, 1, 10);");
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_clamp_at_max() {
    let result = eval_ok("clamp(10, 1, 10);");
    assert_eq!(result, Value::Number(10.0));
}

#[test]
fn test_clamp_invalid_range_returns_nan() {
    let result = eval_ok("clamp(5, 10, 1);");
    assert!(matches!(result, Value::Number(x) if x.is_nan()));
}

// ============================================================================
// sign() tests
// ============================================================================

#[test]
fn test_sign_positive() {
    let result = eval_ok("sign(42);");
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_sign_negative() {
    let result = eval_ok("sign(-42);");
    assert_eq!(result, Value::Number(-1.0));
}

#[test]
fn test_sign_zero() {
    let result = eval_ok("sign(0);");
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_sign_nan() {
    let result = eval_ok("sign(0 / 0);");
    assert!(matches!(result, Value::Number(x) if x.is_nan()));
}

// ============================================================================
// random() tests
// ============================================================================

#[test]
fn test_random_in_range() {
    let result = eval_ok("random();");
    match result {
        Value::Number(n) => {
            assert!(n >= 0.0 && n < 1.0, "random() should return [0, 1), got {}", n);
        }
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_random_multiple_calls_differ() {
    let result = eval_ok(r#"
        let r1 = random();
        let r2 = random();
        let r3 = random();
        if (r1 == r2) {
            return false;
        } else {
            return true;
        }
    "#);
    // While technically possible for two random calls to match,
    // it's astronomically unlikely with f64 precision
    assert_eq!(result, Value::Bool(true));
}

// ============================================================================
// Constants tests
// ============================================================================

#[test]
fn test_constant_pi() {
    let result = eval_ok("PI;");
    match result {
        Value::Number(n) => {
            let expected = std::f64::consts::PI;
            assert_eq!(n, expected);
        }
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_constant_e() {
    let result = eval_ok("E;");
    match result {
        Value::Number(n) => {
            let expected = std::f64::consts::E;
            assert_eq!(n, expected);
        }
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_constant_sqrt2() {
    let result = eval_ok("SQRT2;");
    match result {
        Value::Number(n) => {
            let expected = std::f64::consts::SQRT_2;
            assert_eq!(n, expected);
        }
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_constant_ln2() {
    let result = eval_ok("LN2;");
    match result {
        Value::Number(n) => {
            let expected = std::f64::consts::LN_2;
            assert_eq!(n, expected);
        }
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_constant_ln10() {
    let result = eval_ok("LN10;");
    match result {
        Value::Number(n) => {
            let expected = std::f64::consts::LN_10;
            assert_eq!(n, expected);
        }
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_constants_in_expressions() {
    let result = eval_ok("PI * 2;");
    match result {
        Value::Number(n) => {
            let expected = std::f64::consts::PI * 2.0;
            assert_eq!(n, expected);
        }
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_constant_e_with_log() {
    let result = eval_ok("log(E);");
    match result {
        Value::Number(n) => {
            assert!((n - 1.0).abs() < 0.000001);
        }
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_constant_pi_with_sin() {
    let result = eval_ok("sin(PI);");
    match result {
        Value::Number(n) => {
            assert!(n.abs() < 0.000001);
        }
        _ => panic!("Expected number"),
    }
}
