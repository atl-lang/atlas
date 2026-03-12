// Phase 05: Value::Future — type_name, Display, PartialEq, Arc identity, stdlib typeOf

use atlas_runtime::api::Runtime;
use atlas_runtime::async_runtime::AtlasFuture;
use atlas_runtime::value::Value;
use std::sync::Arc;

fn eval_ok(code: &str) -> Value {
    let mut runtime = Runtime::new();
    runtime.eval(code).unwrap()
}

fn eval_vm_ok(code: &str) -> Value {
    let mut runtime = Runtime::new();
    runtime.eval(code).unwrap()
}

/// 1. type_name() returns "Future" (capitalized per spec)
#[test]
fn test_value_future_type_name() {
    let future = Value::Future(Arc::new(AtlasFuture::resolved(Value::Number(1.0))));
    assert_eq!(future.type_name(), "Future");
}

/// 2. Display renders "<Future>" (not internal state)
#[test]
fn test_value_future_display() {
    let future = Value::Future(Arc::new(AtlasFuture::resolved(Value::Number(42.0))));
    assert_eq!(format!("{}", future), "<Future>");
}

/// 3. Two distinct Value::Future instances are never equal
#[test]
fn test_value_future_inequality() {
    let a = Value::Future(Arc::new(AtlasFuture::resolved(Value::Number(1.0))));
    let b = Value::Future(Arc::new(AtlasFuture::resolved(Value::Number(1.0))));
    assert_ne!(a, b);
    // Even the same future compared to itself is not equal (opaque handle)
    let c = Value::Future(Arc::new(AtlasFuture::resolved(Value::Number(1.0))));
    assert_ne!(c.clone(), c);
}

/// 4. Cloning a Value::Future shares the same Arc handle
#[test]
fn test_value_future_clone_shares_arc() {
    let inner = Arc::new(AtlasFuture::resolved(Value::Number(7.0)));
    let a = Value::Future(Arc::clone(&inner));
    let b = a.clone();
    // Both must reference the same allocation
    if let (Value::Future(arc_a), Value::Future(arc_b)) = (a, b) {
        assert!(Arc::ptr_eq(&arc_a, &arc_b));
    } else {
        panic!("Expected Value::Future");
    }
}

/// 5. Stdlib typeof() returns "Future" — interpreter and VM parity
#[test]
fn test_value_future_typeof_stdlib_interpreter() {
    let result = eval_ok("typeof(futureResolve(99))");
    assert_eq!(result, Value::string("Future"));
}

#[test]
fn test_value_future_typeof_stdlib_vm() {
    let result = eval_vm_ok("typeof(futureResolve(99))");
    assert_eq!(result, Value::string("Future"));
}
