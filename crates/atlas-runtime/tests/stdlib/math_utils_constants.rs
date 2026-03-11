//! Interpreter tests for math utilities and constants
//!
//! Tests: Math.clamp, Math.sign, Math.random, Math.trunc, Math.exp, Math.cbrt, Math.hypot
//! Constants: Math.PI, Math.E, Math.SQRT2, Math.LN2, Math.LN10
//! New B22: Math.log2, Math.log10

use crate::stdlib::eval_ok;
use atlas_runtime::value::Value;

// ============================================================================
// Math.clamp() tests — returns Result<number, string>
// ============================================================================

#[test]
fn test_clamp_within_range() {
    let result = eval_ok("unwrap(Math.clamp(5, 1, 10));");
    assert_eq!(result, Value::Number(5.0));
}

#[test]
fn test_clamp_below_min() {
    let result = eval_ok("unwrap(Math.clamp(-5, 1, 10));");
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_clamp_above_max() {
    let result = eval_ok("unwrap(Math.clamp(15, 1, 10));");
    assert_eq!(result, Value::Number(10.0));
}

#[test]
fn test_clamp_at_min() {
    let result = eval_ok("unwrap(Math.clamp(1, 1, 10));");
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_clamp_at_max() {
    let result = eval_ok("unwrap(Math.clamp(10, 1, 10));");
    assert_eq!(result, Value::Number(10.0));
}

#[test]
fn test_clamp_invalid_range_returns_err() {
    let result = eval_ok("isErr(Math.clamp(5, 10, 1));");
    assert_eq!(result, Value::Bool(true));
}

// ============================================================================
// Math.sign() tests
// ============================================================================

#[test]
fn test_sign_positive() {
    let result = eval_ok("Math.sign(42);");
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_sign_negative() {
    let result = eval_ok("Math.sign(-42);");
    assert_eq!(result, Value::Number(-1.0));
}

#[test]
fn test_sign_zero() {
    let result = eval_ok("Math.sign(0);");
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_sign_nan() {
    let result = eval_ok("Math.sign(0 / 0);");
    assert!(matches!(result, Value::Number(x) if x.is_nan()));
}

// ============================================================================
// Math.random() tests
// ============================================================================

#[test]
fn test_random_in_range() {
    let result = eval_ok("Math.random();");
    match result {
        Value::Number(n) => {
            assert!(n >= 0.0 && n < 1.0, "Math.random() should return [0, 1), got {}", n);
        }
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_random_multiple_calls_differ() {
    let result = eval_ok(r#"
        let r1 = Math.random();
        let r2 = Math.random();
        let r3 = Math.random();
        if (r1 == r2) {
            return false;
        } else {
            return true;
        }
    "#);
    assert_eq!(result, Value::Bool(true));
}

// ============================================================================
// Math.trunc() tests
// ============================================================================

#[test]
fn test_trunc_positive() {
    let result = eval_ok("Math.trunc(4.9);");
    assert_eq!(result, Value::Number(4.0));
}

#[test]
fn test_trunc_negative() {
    let result = eval_ok("Math.trunc(-4.9);");
    assert_eq!(result, Value::Number(-4.0));
}

#[test]
fn test_trunc_integer() {
    let result = eval_ok("Math.trunc(5.0);");
    assert_eq!(result, Value::Number(5.0));
}

// ============================================================================
// Math.exp() tests
// ============================================================================

#[test]
fn test_exp_zero() {
    let result = eval_ok("Math.exp(0);");
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_exp_one() {
    let result = eval_ok("Math.exp(1);");
    match result {
        Value::Number(n) => assert!((n - std::f64::consts::E).abs() < 0.000001),
        _ => panic!("Expected number"),
    }
}

// ============================================================================
// Math.cbrt() tests
// ============================================================================

#[test]
fn test_cbrt_positive() {
    let result = eval_ok("Math.cbrt(8);");
    match result {
        Value::Number(n) => assert!((n - 2.0).abs() < 0.000001),
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_cbrt_negative() {
    let result = eval_ok("Math.cbrt(-8);");
    match result {
        Value::Number(n) => assert!((n - (-2.0)).abs() < 0.000001),
        _ => panic!("Expected number"),
    }
}

// ============================================================================
// Math.hypot() tests
// ============================================================================

#[test]
fn test_hypot_3_4() {
    let result = eval_ok("Math.hypot(3, 4);");
    assert_eq!(result, Value::Number(5.0));
}

#[test]
fn test_hypot_zero() {
    let result = eval_ok("Math.hypot(0, 0);");
    assert_eq!(result, Value::Number(0.0));
}

// ============================================================================
// Math.log2() / Math.log10() tests — return Result<number, string>
// ============================================================================

#[test]
fn test_log2_of_8() {
    let result = eval_ok("unwrap(Math.log2(8));");
    match result {
        Value::Number(n) => assert!((n - 3.0).abs() < 0.000001),
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_log2_negative_returns_err() {
    let result = eval_ok("isErr(Math.log2(-1));");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_log10_of_100() {
    let result = eval_ok("unwrap(Math.log10(100));");
    match result {
        Value::Number(n) => assert!((n - 2.0).abs() < 0.000001),
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_log10_negative_returns_err() {
    let result = eval_ok("isErr(Math.log10(-1));");
    assert_eq!(result, Value::Bool(true));
}

// ============================================================================
// Constants tests
// ============================================================================

#[test]
fn test_constant_pi() {
    let result = eval_ok("Math.PI();");
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
    let result = eval_ok("Math.E();");
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
    let result = eval_ok("Math.SQRT2();");
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
    let result = eval_ok("Math.LN2();");
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
    let result = eval_ok("Math.LN10();");
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
    let result = eval_ok("Math.PI() * 2;");
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
    let result = eval_ok("unwrap(Math.log(Math.E()));");
    match result {
        Value::Number(n) => {
            assert!((n - 1.0).abs() < 0.000001);
        }
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_constant_pi_with_sin() {
    let result = eval_ok("Math.sin(Math.PI());");
    match result {
        Value::Number(n) => {
            assert!(n.abs() < 0.000001);
        }
        _ => panic!("Expected number"),
    }
}
