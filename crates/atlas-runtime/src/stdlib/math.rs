//! Math standard library functions
//!
//! Complete math API with:
//! - Basic operations (abs, floor, ceil, round, min, max)
//! - Exponential/power (sqrt, pow, log)
//! - Trigonometry (sin, cos, tan, asin, acos, atan)
//! - Utilities (clamp, sign, random)
//! - Constants (PI, E, SQRT2, LN2, LN10)
//!
//! All functions follow IEEE 754 semantics:
//! - NaN propagates through operations
//! - Infinities handled correctly
//! - Signed zero preserved
//! - Domain errors return NaN (not panic)

use crate::span::Span;
use crate::value::{RuntimeError, Value};
use rand::RngExt;

// ============================================================================
// Math Constants
// ============================================================================

/// π (pi) - ratio of circle's circumference to diameter
pub const PI: f64 = std::f64::consts::PI;

/// e - Euler's number, base of natural logarithm
pub const E: f64 = std::f64::consts::E;

/// √2 - square root of 2
pub const SQRT2: f64 = std::f64::consts::SQRT_2;

/// ln(2) - natural logarithm of 2
pub const LN2: f64 = std::f64::consts::LN_2;

/// ln(10) - natural logarithm of 10
pub const LN10: f64 = std::f64::consts::LN_10;

// ============================================================================
// Basic Operations
// ============================================================================

/// abs(x: number) -> number
///
/// Returns absolute value of x.
/// Preserves signed zero: abs(-0) = +0
/// abs(±∞) = +∞
/// abs(NaN) = NaN
pub fn abs(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "abs() expects 1 argument".to_string(),
            span,
        });
    }

    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.abs())),
        _ => Err(RuntimeError::TypeError {
            msg: "abs() expects number argument".to_string(),
            span,
        }),
    }
}

/// floor(x: number) -> number
///
/// Returns largest integer ≤ x.
/// floor(1.9) = 1, floor(-1.1) = -2
/// floor(±∞) = ±∞, floor(NaN) = NaN
pub fn floor(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "floor() expects 1 argument".to_string(),
            span,
        });
    }

    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.floor())),
        _ => Err(RuntimeError::TypeError {
            msg: "floor() expects number argument".to_string(),
            span,
        }),
    }
}

/// ceil(x: number) -> number
///
/// Returns smallest integer ≥ x.
/// ceil(1.1) = 2, ceil(-1.9) = -1
/// ceil(±∞) = ±∞, ceil(NaN) = NaN
pub fn ceil(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "ceil() expects 1 argument".to_string(),
            span,
        });
    }

    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.ceil())),
        _ => Err(RuntimeError::TypeError {
            msg: "ceil() expects number argument".to_string(),
            span,
        }),
    }
}

/// round(x: number) -> number
///
/// Rounds to nearest integer using ties-to-even (banker's rounding).
/// round(2.5) = 2, round(3.5) = 4
/// round(±∞) = ±∞, round(NaN) = NaN
pub fn round(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "round() expects 1 argument".to_string(),
            span,
        });
    }

    match &args[0] {
        Value::Number(n) => {
            // Rust's round() uses ties-away-from-zero, we need ties-to-even
            // Implement banker's rounding manually
            let rounded = if n.is_nan() || n.is_infinite() {
                *n
            } else {
                let floor_val = n.floor();
                let fract = n - floor_val;
                if fract < 0.5 {
                    floor_val
                } else if fract > 0.5 {
                    floor_val + 1.0
                } else {
                    // Exactly 0.5 - round to even
                    if floor_val % 2.0 == 0.0 {
                        floor_val
                    } else {
                        floor_val + 1.0
                    }
                }
            };
            Ok(Value::Number(rounded))
        }
        _ => Err(RuntimeError::TypeError {
            msg: "round() expects number argument".to_string(),
            span,
        }),
    }
}

/// min(a: number, b: number) -> number
///
/// Returns smaller of two numbers.
/// If either is NaN, returns NaN.
/// min(-0, +0) is implementation-defined
pub fn min(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "min() expects 2 arguments".to_string(),
            span,
        });
    }

    match (&args[0], &args[1]) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a.min(*b))),
        _ => Err(RuntimeError::TypeError {
            msg: "min() expects number arguments".to_string(),
            span,
        }),
    }
}

/// max(a: number, b: number) -> number
///
/// Returns larger of two numbers.
/// If either is NaN, returns NaN.
/// max(-0, +0) is implementation-defined
pub fn max(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "max() expects 2 arguments".to_string(),
            span,
        });
    }

    match (&args[0], &args[1]) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a.max(*b))),
        _ => Err(RuntimeError::TypeError {
            msg: "max() expects number arguments".to_string(),
            span,
        }),
    }
}

// ============================================================================
// Exponential/Power Operations
// ============================================================================

/// sqrt(x: number) -> Result<number, string>
///
/// Returns Result: Ok(sqrt) for non-negative x, Err for negative x.
/// sqrt(+∞) = Ok(+∞), sqrt(NaN) = Err
pub fn sqrt(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "sqrt() expects 1 argument".to_string(),
            span,
        });
    }

    match &args[0] {
        Value::Number(n) => {
            if n.is_nan() || *n < 0.0 {
                Ok(Value::Result(Err(Box::new(Value::string(
                    "sqrt() domain error: argument must be non-negative",
                )))))
            } else {
                Ok(Value::Result(Ok(Box::new(Value::Number(n.sqrt())))))
            }
        }
        _ => Err(RuntimeError::TypeError {
            msg: "sqrt() expects number argument".to_string(),
            span,
        }),
    }
}

/// pow(base: number, exponent: number) -> number
///
/// Returns base raised to exponent power.
/// pow(x, 0) = 1 for any x (including NaN)
/// pow(1, y) = 1 for any y (including NaN)
/// pow(NaN, y) = NaN (except y=0)
pub fn pow(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "pow() expects 2 arguments".to_string(),
            span,
        });
    }

    match (&args[0], &args[1]) {
        (Value::Number(base), Value::Number(exp)) => Ok(Value::Number(base.powf(*exp))),
        _ => Err(RuntimeError::TypeError {
            msg: "pow() expects number arguments".to_string(),
            span,
        }),
    }
}

/// log(x: number) -> Result<number, string>
///
/// Returns Result: Ok(ln(x)) for positive x, Err for non-positive x.
pub fn log(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "log() expects 1 argument".to_string(),
            span,
        });
    }

    match &args[0] {
        Value::Number(n) => {
            if n.is_nan() || *n <= 0.0 {
                Ok(Value::Result(Err(Box::new(Value::string(
                    "log() domain error: argument must be positive",
                )))))
            } else {
                Ok(Value::Result(Ok(Box::new(Value::Number(n.ln())))))
            }
        }
        _ => Err(RuntimeError::TypeError {
            msg: "log() expects number argument".to_string(),
            span,
        }),
    }
}

// ============================================================================
// Trigonometric Functions (all use radians)
// ============================================================================

/// sin(x: number) -> number
///
/// Returns sine of x (x in radians).
/// sin(±∞) = NaN, sin(NaN) = NaN
pub fn sin(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "sin() expects 1 argument".to_string(),
            span,
        });
    }

    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.sin())),
        _ => Err(RuntimeError::TypeError {
            msg: "sin() expects number argument".to_string(),
            span,
        }),
    }
}

/// cos(x: number) -> number
///
/// Returns cosine of x (x in radians).
/// cos(±∞) = NaN, cos(NaN) = NaN
pub fn cos(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "cos() expects 1 argument".to_string(),
            span,
        });
    }

    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.cos())),
        _ => Err(RuntimeError::TypeError {
            msg: "cos() expects number argument".to_string(),
            span,
        }),
    }
}

/// tan(x: number) -> number
///
/// Returns tangent of x (x in radians).
/// tan(±∞) = NaN, tan(NaN) = NaN
pub fn tan(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "tan() expects 1 argument".to_string(),
            span,
        });
    }

    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.tan())),
        _ => Err(RuntimeError::TypeError {
            msg: "tan() expects number argument".to_string(),
            span,
        }),
    }
}

/// asin(x: number) -> Result<number, string>
///
/// Returns Result: Ok(asin(x)) for x in [-1, 1], Err otherwise.
pub fn asin(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "asin() expects 1 argument".to_string(),
            span,
        });
    }

    match &args[0] {
        Value::Number(n) => {
            if n.is_nan() || *n < -1.0 || *n > 1.0 {
                Ok(Value::Result(Err(Box::new(Value::string(
                    "asin() domain error: argument must be in [-1, 1]",
                )))))
            } else {
                Ok(Value::Result(Ok(Box::new(Value::Number(n.asin())))))
            }
        }
        _ => Err(RuntimeError::TypeError {
            msg: "asin() expects number argument".to_string(),
            span,
        }),
    }
}

/// acos(x: number) -> Result<number, string>
///
/// Returns Result: Ok(acos(x)) for x in [-1, 1], Err otherwise.
pub fn acos(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "acos() expects 1 argument".to_string(),
            span,
        });
    }

    match &args[0] {
        Value::Number(n) => {
            if n.is_nan() || *n < -1.0 || *n > 1.0 {
                Ok(Value::Result(Err(Box::new(Value::string(
                    "acos() domain error: argument must be in [-1, 1]",
                )))))
            } else {
                Ok(Value::Result(Ok(Box::new(Value::Number(n.acos())))))
            }
        }
        _ => Err(RuntimeError::TypeError {
            msg: "acos() expects number argument".to_string(),
            span,
        }),
    }
}

/// atan(x: number) -> number
///
/// Returns arctangent of x in radians.
/// Range: (-π/2, π/2)
/// atan(±∞) = ±π/2, atan(NaN) = NaN
pub fn atan(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "atan() expects 1 argument".to_string(),
            span,
        });
    }

    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.atan())),
        _ => Err(RuntimeError::TypeError {
            msg: "atan() expects number argument".to_string(),
            span,
        }),
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// clamp(value: number, min: number, max: number) -> Result<number, string>
///
/// Restricts value to [min, max] range.
/// Returns Err if min > max or any argument is NaN.
pub fn clamp(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::TypeError {
            msg: "clamp() expects 3 arguments".to_string(),
            span,
        });
    }

    match (&args[0], &args[1], &args[2]) {
        (Value::Number(value), Value::Number(min_val), Value::Number(max_val)) => {
            if value.is_nan() || min_val.is_nan() || max_val.is_nan() {
                return Ok(Value::Result(Err(Box::new(Value::string(
                    "clamp() domain error: NaN argument",
                )))));
            }
            if min_val > max_val {
                return Ok(Value::Result(Err(Box::new(Value::string(
                    "clamp() domain error: min > max",
                )))));
            }
            let clamped = value.max(*min_val).min(*max_val);
            Ok(Value::Result(Ok(Box::new(Value::Number(clamped)))))
        }
        _ => Err(RuntimeError::TypeError {
            msg: "clamp() expects number arguments".to_string(),
            span,
        }),
    }
}

/// sign(x: number) -> number
///
/// Returns sign of x: -1 for negative, 0 for zero, 1 for positive.
/// Preserves signed zero: sign(-0) = -0, sign(+0) = +0
/// sign(NaN) = NaN
pub fn sign(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "sign() expects 1 argument".to_string(),
            span,
        });
    }

    match &args[0] {
        Value::Number(n) => {
            let result = if n.is_nan() {
                f64::NAN
            } else if *n > 0.0 {
                1.0
            } else if *n < 0.0 {
                -1.0
            } else {
                // Preserve signed zero
                *n
            };
            Ok(Value::Number(result))
        }
        _ => Err(RuntimeError::TypeError {
            msg: "sign() expects number argument".to_string(),
            span,
        }),
    }
}

/// atan2(y: number, x: number) -> number
///
/// Returns arctangent of y/x in radians, using the signs to determine quadrant.
/// Range: (-π, π]
/// atan2(±0, +0) = ±0, atan2(±0, -0) = ±π
/// atan2(NaN, x) = NaN, atan2(y, NaN) = NaN
pub fn atan2(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "atan2() expects 2 arguments".to_string(),
            span,
        });
    }

    match (&args[0], &args[1]) {
        (Value::Number(y), Value::Number(x)) => Ok(Value::Number(y.atan2(*x))),
        _ => Err(RuntimeError::TypeError {
            msg: "atan2() expects number arguments".to_string(),
            span,
        }),
    }
}

/// trunc(x: number) -> number
///
/// Returns integer part of x by removing any fractional digits.
/// trunc(1.9) = 1, trunc(-1.9) = -1
/// trunc(±∞) = ±∞, trunc(NaN) = NaN
pub fn trunc(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "trunc() expects 1 argument".to_string(),
            span,
        });
    }

    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.trunc())),
        _ => Err(RuntimeError::TypeError {
            msg: "trunc() expects number argument".to_string(),
            span,
        }),
    }
}

/// log2(x: number) -> Result<number, string>
///
/// Returns Result: Ok(log2(x)) for positive x, Err for non-positive x.
pub fn log2(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "log2() expects 1 argument".to_string(),
            span,
        });
    }

    match &args[0] {
        Value::Number(n) => {
            if n.is_nan() || *n <= 0.0 {
                Ok(Value::Result(Err(Box::new(Value::string(
                    "log2() domain error: argument must be positive",
                )))))
            } else {
                Ok(Value::Result(Ok(Box::new(Value::Number(n.log2())))))
            }
        }
        _ => Err(RuntimeError::TypeError {
            msg: "log2() expects number argument".to_string(),
            span,
        }),
    }
}

/// log10(x: number) -> Result<number, string>
///
/// Returns Result: Ok(log10(x)) for positive x, Err for non-positive x.
pub fn log10(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "log10() expects 1 argument".to_string(),
            span,
        });
    }

    match &args[0] {
        Value::Number(n) => {
            if n.is_nan() || *n <= 0.0 {
                Ok(Value::Result(Err(Box::new(Value::string(
                    "log10() domain error: argument must be positive",
                )))))
            } else {
                Ok(Value::Result(Ok(Box::new(Value::Number(n.log10())))))
            }
        }
        _ => Err(RuntimeError::TypeError {
            msg: "log10() expects number argument".to_string(),
            span,
        }),
    }
}

/// exp(x: number) -> number
///
/// Returns e raised to the power x.
/// exp(±∞) = +∞/0, exp(NaN) = NaN
pub fn exp(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "exp() expects 1 argument".to_string(),
            span,
        });
    }

    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.exp())),
        _ => Err(RuntimeError::TypeError {
            msg: "exp() expects number argument".to_string(),
            span,
        }),
    }
}

/// cbrt(x: number) -> number
///
/// Returns cube root of x.
/// cbrt(-8) = -2, cbrt(±∞) = ±∞, cbrt(NaN) = NaN
pub fn cbrt(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "cbrt() expects 1 argument".to_string(),
            span,
        });
    }

    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.cbrt())),
        _ => Err(RuntimeError::TypeError {
            msg: "cbrt() expects number argument".to_string(),
            span,
        }),
    }
}

/// hypot(x: number, y: number) -> number
///
/// Returns sqrt(x² + y²) without intermediate overflow/underflow.
/// hypot(±∞, y) = +∞ (even if y is NaN)
pub fn hypot(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "hypot() expects 2 arguments".to_string(),
            span,
        });
    }

    match (&args[0], &args[1]) {
        (Value::Number(x), Value::Number(y)) => Ok(Value::Number(x.hypot(*y))),
        _ => Err(RuntimeError::TypeError {
            msg: "hypot() expects number arguments".to_string(),
            span,
        }),
    }
}

// ============================================================================
// Math Constant Accessors (0-arg functions returning the constant)
// ============================================================================

/// Math.PI — ratio of circle's circumference to diameter (~3.14159)
pub fn math_pi(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::TypeError {
            msg: "Math.PI expects no arguments".to_string(),
            span,
        });
    }
    Ok(Value::Number(std::f64::consts::PI))
}

/// Math.E — Euler's number, base of natural logarithm (~2.71828)
pub fn math_e(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::TypeError {
            msg: "Math.E expects no arguments".to_string(),
            span,
        });
    }
    Ok(Value::Number(std::f64::consts::E))
}

/// Math.SQRT2 — square root of 2 (~1.41421)
pub fn math_sqrt2(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::TypeError {
            msg: "Math.SQRT2 expects no arguments".to_string(),
            span,
        });
    }
    Ok(Value::Number(std::f64::consts::SQRT_2))
}

/// Math.LN2 — natural logarithm of 2 (~0.69315)
pub fn math_ln2(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::TypeError {
            msg: "Math.LN2 expects no arguments".to_string(),
            span,
        });
    }
    Ok(Value::Number(std::f64::consts::LN_2))
}

/// Math.LN10 — natural logarithm of 10 (~2.30259)
pub fn math_ln10(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::TypeError {
            msg: "Math.LN10 expects no arguments".to_string(),
            span,
        });
    }
    Ok(Value::Number(std::f64::consts::LN_10))
}

/// random() -> number
///
/// Returns pseudo-random number in [0, 1) with uniform distribution.
/// Uses thread-local rng for randomness.
pub fn random(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::TypeError {
            msg: "random() expects no arguments".to_string(),
            span,
        });
    }

    let mut rng = rand::rng();
    let value: f64 = rng.random(); // random() for f64 returns [0.0, 1.0)
    Ok(Value::Number(value))
}
