//! Tests for default parameter values (B39-P05)
//!
//! Tests the `fn foo(x: T = expr)` syntax for default function parameters.

use super::*;
use atlas_runtime::Atlas;

// ===== Basic default parameter tests =====

#[test]
fn default_param_number_used() {
    let runtime = Atlas::new();
    let code = r#"
fn add(a: number, b: number = 10): number {
    return a + b;
}
add(5);
"#;
    let result = runtime.eval(code).expect("should succeed");
    assert_eq!(result, Value::Number(15.0));
}

#[test]
fn default_param_number_override() {
    let runtime = Atlas::new();
    let code = r#"
fn add(a: number, b: number = 10): number {
    return a + b;
}
add(5, 3);
"#;
    let result = runtime.eval(code).expect("should succeed");
    assert_eq!(result, Value::Number(8.0));
}

#[test]
fn default_param_string_used() {
    let runtime = Atlas::new();
    let code = r#"
fn greet(name: string = "World"): string {
    return "Hello, " + name + "!";
}
greet();
"#;
    let result = runtime.eval(code).expect("should succeed");
    match result {
        Value::String(s) => assert_eq!(s.as_ref(), "Hello, World!"),
        _ => panic!("Expected string, got {:?}", result),
    }
}

#[test]
fn default_param_string_override() {
    let runtime = Atlas::new();
    let code = r#"
fn greet(name: string = "World"): string {
    return "Hello, " + name + "!";
}
greet("Atlas");
"#;
    let result = runtime.eval(code).expect("should succeed");
    match result {
        Value::String(s) => assert_eq!(s.as_ref(), "Hello, Atlas!"),
        _ => panic!("Expected string, got {:?}", result),
    }
}

#[test]
fn default_param_bool_used() {
    let runtime = Atlas::new();
    let code = r#"
fn check(flag: bool = true): bool {
    return flag;
}
check();
"#;
    let result = runtime.eval(code).expect("should succeed");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn default_param_bool_override() {
    let runtime = Atlas::new();
    let code = r#"
fn check(flag: bool = true): bool {
    return flag;
}
check(false);
"#;
    let result = runtime.eval(code).expect("should succeed");
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn multiple_defaults_all_used() {
    let runtime = Atlas::new();
    let code = r#"
fn compute(a: number = 1, b: number = 2, c: number = 3): number {
    return a * 100 + b * 10 + c;
}
compute();
"#;
    let result = runtime.eval(code).expect("should succeed");
    assert_eq!(result, Value::Number(123.0));
}

#[test]
fn multiple_defaults_partial_override() {
    let runtime = Atlas::new();
    let code = r#"
fn compute(a: number = 1, b: number = 2, c: number = 3): number {
    return a * 100 + b * 10 + c;
}
compute(9);
"#;
    let result = runtime.eval(code).expect("should succeed");
    assert_eq!(result, Value::Number(923.0));
}

#[test]
fn multiple_defaults_all_override() {
    let runtime = Atlas::new();
    let code = r#"
fn compute(a: number = 1, b: number = 2, c: number = 3): number {
    return a * 100 + b * 10 + c;
}
compute(7, 8, 9);
"#;
    let result = runtime.eval(code).expect("should succeed");
    assert_eq!(result, Value::Number(789.0));
}

#[test]
fn mixed_required_and_default() {
    let runtime = Atlas::new();
    let code = r#"
fn scale(value: number, factor: number = 2): number {
    return value * factor;
}
scale(10);
"#;
    let result = runtime.eval(code).expect("should succeed");
    assert_eq!(result, Value::Number(20.0));
}

#[test]
fn mixed_required_and_default_override() {
    let runtime = Atlas::new();
    let code = r#"
fn scale(value: number, factor: number = 2): number {
    return value * factor;
}
scale(10, 5);
"#;
    let result = runtime.eval(code).expect("should succeed");
    assert_eq!(result, Value::Number(50.0));
}

// ===== Error handling tests =====

#[test]
fn error_required_after_default() {
    let runtime = Atlas::new();
    let code = r#"
fn bad(a: number = 1, b: number): void {
    console.log(a + b);
}
"#;
    let err = runtime.eval(code).expect_err("should fail");
    assert!(!err.is_empty());
    // AT3062 is the error code for required param after default
    let has_error = err
        .iter()
        .any(|d| d.code == "AT3062" || d.message.contains("required"));
    assert!(
        has_error,
        "Expected AT3062 or 'required' error, got: {:?}",
        err
    );
}

#[test]
fn error_too_few_args() {
    let runtime = Atlas::new();
    let code = r#"
fn need_one(required: number, optional: number = 10): number {
    return required + optional;
}
need_one();
"#;
    let err = runtime.eval(code).expect_err("should fail");
    assert!(!err.is_empty());
}

#[test]
fn error_too_many_args() {
    let runtime = Atlas::new();
    let code = r#"
fn take_two(a: number = 1, b: number = 2): number {
    return a + b;
}
take_two(1, 2, 3);
"#;
    let err = runtime.eval(code).expect_err("should fail");
    assert!(!err.is_empty());
}

// ===== Type checking tests =====

#[test]
fn default_type_mismatch_error() {
    let runtime = Atlas::new();
    let code = r#"
fn bad(x: number = "hello"): number {
    return x;
}
"#;
    let err = runtime.eval(code).expect_err("should fail");
    assert!(!err.is_empty());
}

// ===== Edge cases =====

#[test]
fn default_zero_value() {
    let runtime = Atlas::new();
    let code = r#"
fn with_zero(n: number = 0): number {
    return n * 2 + 1;
}
with_zero();
"#;
    let result = runtime.eval(code).expect("should succeed");
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn default_negative_number() {
    let runtime = Atlas::new();
    let code = r#"
fn from_negative(n: number = -10): number {
    return n + 20;
}
from_negative();
"#;
    let result = runtime.eval(code).expect("should succeed");
    assert_eq!(result, Value::Number(10.0));
}

#[test]
fn default_empty_string() {
    let runtime = Atlas::new();
    let code = r#"
fn with_empty(s: string = ""): string {
    return "prefix" + s + "suffix";
}
with_empty();
"#;
    let result = runtime.eval(code).expect("should succeed");
    match result {
        Value::String(s) => assert_eq!(s.as_ref(), "prefixsuffix"),
        _ => panic!("Expected string, got {:?}", result),
    }
}

#[test]
fn default_called_multiple_times() {
    let runtime = Atlas::new();
    let code = r#"
fn inc(x: number = 0): number {
    return x + 1;
}
let a = inc();
let b = inc();
let c = inc(100);
let d = inc();
a * 1000 + b * 100 + c * 1 + d / 10;
"#;
    let result = runtime.eval(code).expect("should succeed");
    // a=1, b=1, c=101, d=1 => 1*1000 + 1*100 + 101*1 + 1/10 = 1000 + 100 + 101 + 0.1 = 1201.1
    assert_eq!(result, Value::Number(1201.1));
}

#[test]
fn default_with_expression() {
    let runtime = Atlas::new();
    let code = r#"
fn multiply(a: number, factor: number = 2 * 5): number {
    return a * factor;
}
multiply(3);
"#;
    let result = runtime.eval(code).expect("should succeed");
    assert_eq!(result, Value::Number(30.0));
}
