//! Testing primitives — assertion functions for Atlas
//!
//! Follows the Rust/Go model: stdlib provides assertion primitives only.
//! Test discovery, execution, and reporting belong to the CLI (CLI/phase-02).
//!
//! # API
//!
//! ## Basic
//! - `assert(condition, message)` — assert condition is true
//! - `assertFalse(condition, message)` — assert condition is false
//!
//! ## Equality
//! - `assertEqual(actual, expected)` — assert deep equality
//! - `assertNotEqual(actual, expected)` — assert not equal
//!
//! ## Result
//! - `assertOk(result)` — assert `Result` is `Ok`, return unwrapped value
//! - `assertErr(result)` — assert `Result` is `Err`, return unwrapped error
//!
//! ## Option
//! - `assertSome(option)` — assert `Option` is `Some`, return unwrapped value
//! - `assertNone(option)` — assert `Option` is `None`
//!
//! ## Collections
//! - `assertContains(array, value)` — assert array contains value
//! - `assertEmpty(array)` — assert array is empty
//! - `assertLength(array, expected)` — assert array length matches
//!
//! ## Error
//! - `assertThrows(fn)` — assert NativeFunction throws (returns Err)
//! - `assertNoThrow(fn)` — assert NativeFunction does not throw

use crate::span::Span;
use crate::value::{RuntimeError, Value};

// ============================================================================
// Internal helpers
// ============================================================================

/// Build a TypeError used for assertion failures (clear message, helpful diff).
fn assertion_error(msg: impl Into<String>, span: Span) -> RuntimeError {
    RuntimeError::TypeError {
        msg: msg.into(),
        span,
    }
}

/// Build a TypeError used for wrong argument types.
fn type_error(expected: &str, got: &str, span: Span) -> RuntimeError {
    RuntimeError::TypeError {
        msg: format!("expected {}, got {}", expected, got),
        span,
    }
}

/// Verify exact arity; returns an `InvalidStdlibArgument` error on mismatch.
fn check_arity(
    fn_name: &str,
    args: &[Value],
    expected: usize,
    span: Span,
) -> Result<(), RuntimeError> {
    if args.len() != expected {
        return Err(RuntimeError::TypeError {
            msg: format!(
                "{} expects {} argument{}, got {}",
                fn_name,
                expected,
                if expected == 1 { "" } else { "s" },
                args.len()
            ),
            span,
        });
    }
    Ok(())
}

/// Deep equality for Atlas values.
///
/// Arrays are compared element-by-element (not by pointer identity), which is
/// the semantics users expect in test assertions.
fn values_deep_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x == y,
        (Value::String(x), Value::String(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Null, Value::Null) => true,
        (Value::Array(x), Value::Array(y)) => {
            let xs = x.as_slice();
            let ys = y.as_slice();
            if xs.len() != ys.len() {
                return false;
            }
            xs.iter()
                .zip(ys.iter())
                .all(|(a, b)| values_deep_equal(a, b))
        }
        (Value::Option(x), Value::Option(y)) => match (x, y) {
            (None, None) => true,
            (Some(a), Some(b)) => values_deep_equal(a, b),
            _ => false,
        },
        (Value::Result(x), Value::Result(y)) => match (x, y) {
            (Ok(a), Ok(b)) => values_deep_equal(a, b),
            (Err(a), Err(b)) => values_deep_equal(a, b),
            _ => false,
        },
        _ => false,
    }
}

/// Render a Value for display in assertion failure messages.
fn display(v: &Value) -> String {
    match v {
        Value::Array(arr) => {
            let items: Vec<String> = arr.as_slice().iter().map(display).collect();
            format!("[{}]", items.join(", "))
        }
        Value::Option(opt) => match opt {
            Some(inner) => format!("Some({})", display(inner)),
            None => "None".to_string(),
        },
        Value::Result(res) => match res {
            Ok(inner) => format!("Ok({})", display(inner)),
            Err(inner) => format!("Err({})", display(inner)),
        },
        other => other.to_string(),
    }
}

// ============================================================================
// Basic assertions
// ============================================================================

/// `assert(condition: bool, message: string) -> void`
///
/// Panics with the given message if `condition` is false.
pub fn assert(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    check_arity("assert", args, 2, span)?;

    let condition = match &args[0] {
        Value::Bool(b) => *b,
        other => return Err(type_error("bool", other.type_name(), span)),
    };
    let message = match &args[1] {
        Value::String(s) => s.as_ref().clone(),
        other => return Err(type_error("string", other.type_name(), span)),
    };

    if !condition {
        return Err(assertion_error(
            format!("Assertion failed: {}", message),
            span,
        ));
    }
    Ok(Value::Null)
}

/// `assertFalse(condition: bool, message: string) -> void`
///
/// Panics with the given message if `condition` is true.
pub fn assert_false(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    check_arity("assertFalse", args, 2, span)?;

    let condition = match &args[0] {
        Value::Bool(b) => *b,
        other => return Err(type_error("bool", other.type_name(), span)),
    };
    let message = match &args[1] {
        Value::String(s) => s.as_ref().clone(),
        other => return Err(type_error("string", other.type_name(), span)),
    };

    if condition {
        return Err(assertion_error(
            format!("Assertion failed (expected false): {}", message),
            span,
        ));
    }
    Ok(Value::Null)
}

// ============================================================================
// Equality assertions
// ============================================================================

/// `assertEqual(actual: T, expected: T) -> void`
///
/// Compares using deep equality. Shows a diff on failure.
pub fn assert_equal(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    check_arity("assertEqual", args, 2, span)?;

    let actual = &args[0];
    let expected = &args[1];

    if !values_deep_equal(actual, expected) {
        return Err(assertion_error(
            format!(
                "Assertion failed: values not equal\n  Actual:   {}\n  Expected: {}",
                display(actual),
                display(expected)
            ),
            span,
        ));
    }
    Ok(Value::Null)
}

/// `assertNotEqual(actual: T, expected: T) -> void`
///
/// Succeeds if `actual` and `expected` are not deeply equal.
pub fn assert_not_equal(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    check_arity("assertNotEqual", args, 2, span)?;

    let actual = &args[0];
    let expected = &args[1];

    if values_deep_equal(actual, expected) {
        return Err(assertion_error(
            format!(
                "Assertion failed: values are equal (expected them to differ)\n  Value: {}",
                display(actual)
            ),
            span,
        ));
    }
    Ok(Value::Null)
}

// ============================================================================
// Result assertions
// ============================================================================

/// `assertOk(result: Result<T, E>) -> T`
///
/// Asserts the result is `Ok` and returns the unwrapped value.
pub fn assert_ok(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    check_arity("assertOk", args, 1, span)?;

    match &args[0] {
        Value::Result(res) => match res {
            Ok(val) => Ok(*val.clone()),
            Err(err) => Err(assertion_error(
                format!("assertOk: expected Ok, got Err({})", display(err)),
                span,
            )),
        },
        other => Err(type_error("Result", other.type_name(), span)),
    }
}

/// `assertErr(result: Result<T, E>) -> E`
///
/// Asserts the result is `Err` and returns the unwrapped error value.
pub fn assert_err(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    check_arity("assertErr", args, 1, span)?;

    match &args[0] {
        Value::Result(res) => match res {
            Err(err) => Ok(*err.clone()),
            Ok(val) => Err(assertion_error(
                format!("assertErr: expected Err, got Ok({})", display(val)),
                span,
            )),
        },
        other => Err(type_error("Result", other.type_name(), span)),
    }
}

// ============================================================================
// Option assertions
// ============================================================================

/// `assertSome(option: Option<T>) -> T`
///
/// Asserts the option is `Some` and returns the unwrapped value.
pub fn assert_some(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    check_arity("assertSome", args, 1, span)?;

    match &args[0] {
        Value::Option(opt) => match opt {
            Some(val) => Ok(*val.clone()),
            None => Err(assertion_error("assertSome: expected Some, got None", span)),
        },
        other => Err(type_error("Option", other.type_name(), span)),
    }
}

/// `assertNone(option: Option<T>) -> void`
///
/// Asserts the option is `None`.
pub fn assert_none(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    check_arity("assertNone", args, 1, span)?;

    match &args[0] {
        Value::Option(opt) => match opt {
            None => Ok(Value::Null),
            Some(val) => Err(assertion_error(
                format!("assertNone: expected None, got Some({})", display(val)),
                span,
            )),
        },
        other => Err(type_error("Option", other.type_name(), span)),
    }
}

// ============================================================================
// Collection assertions
// ============================================================================

/// `assertContains(array: array, value: T) -> void`
///
/// Asserts that `array` contains `value` using deep equality.
pub fn assert_contains(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    check_arity("assertContains", args, 2, span)?;

    let arr = match &args[0] {
        Value::Array(a) => a.clone(),
        other => return Err(type_error("array", other.type_name(), span)),
    };
    let needle = &args[1];

    let found = arr.as_slice().iter().any(|v| values_deep_equal(v, needle));

    if !found {
        return Err(assertion_error(
            format!("assertContains: array does not contain {}", display(needle)),
            span,
        ));
    }
    Ok(Value::Null)
}

/// `assertEmpty(array: array) -> void`
///
/// Asserts that `array` has zero elements.
pub fn assert_empty(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    check_arity("assertEmpty", args, 1, span)?;

    match &args[0] {
        Value::Array(arr) => {
            let len = arr.len();
            if len != 0 {
                return Err(assertion_error(
                    format!("assertEmpty: expected empty array, got length {}", len),
                    span,
                ));
            }
            Ok(Value::Null)
        }
        other => Err(type_error("array", other.type_name(), span)),
    }
}

/// `assertLength(array: array, expected: number) -> void`
///
/// Asserts that `array` has exactly `expected` elements.
pub fn assert_length(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    check_arity("assertLength", args, 2, span)?;

    let arr = match &args[0] {
        Value::Array(a) => a.clone(),
        other => return Err(type_error("array", other.type_name(), span)),
    };
    let expected_len = match &args[1] {
        Value::Number(n) => {
            if n.fract() != 0.0 || *n < 0.0 {
                return Err(RuntimeError::TypeError {
                    msg: format!(
                        "assertLength: expected a non-negative integer length, got {}",
                        n
                    ),
                    span,
                });
            }
            *n as usize
        }
        other => return Err(type_error("number", other.type_name(), span)),
    };

    let actual_len = arr.len();
    if actual_len != expected_len {
        return Err(assertion_error(
            format!(
                "assertLength: expected length {}, got {}",
                expected_len, actual_len
            ),
            span,
        ));
    }
    Ok(Value::Null)
}

// ============================================================================
// Error assertions
// ============================================================================

/// `assertThrows(fn: NativeFunction) -> void`
///
/// Calls `fn` with no arguments and asserts it returns an error.
/// Works with `NativeFunction` values (Rust closures passed via the Atlas API).
///
/// Note: Bytecode functions (defined in Atlas code) require interpreter context
/// and cannot be called directly from stdlib. Use NativeFunction for this assertion.
pub fn assert_throws(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    check_arity("assertThrows", args, 1, span)?;

    match &args[0] {
        Value::NativeFunction(f) => {
            match f(&[]) {
                Ok(_) => Err(assertion_error(
                    "assertThrows: expected function to throw, but it returned successfully",
                    span,
                )),
                Err(_) => Ok(Value::Null), // threw as expected
            }
        }
        Value::Function(_) => Err(RuntimeError::TypeError {
            msg: "assertThrows requires a NativeFunction (Rust closure). \
                  Bytecode functions defined in Atlas code need interpreter context. \
                  Wrap your test logic in a native function via the Atlas embedding API."
                .to_string(),
            span,
        }),
        other => Err(type_error("function", other.type_name(), span)),
    }
}

/// `assertNoThrow(fn: NativeFunction) -> void`
///
/// Calls `fn` with no arguments and asserts it does NOT return an error.
/// Works with `NativeFunction` values (Rust closures passed via the Atlas API).
pub fn assert_no_throw(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    check_arity("assertNoThrow", args, 1, span)?;

    match &args[0] {
        Value::NativeFunction(f) => {
            match f(&[]) {
                Ok(_) => Ok(Value::Null), // no throw = success
                Err(e) => Err(assertion_error(
                    format!(
                        "assertNoThrow: expected function to succeed, but it threw: {}",
                        e
                    ),
                    span,
                )),
            }
        }
        Value::Function(_) => Err(RuntimeError::TypeError {
            msg: "assertNoThrow requires a NativeFunction (Rust closure). \
                  Bytecode functions defined in Atlas code need interpreter context. \
                  Wrap your test logic in a native function via the Atlas embedding API."
                .to_string(),
            span,
        }),
        other => Err(type_error("function", other.type_name(), span)),
    }
}

// ============================================================================
// Unit tests
// ============================================================================
