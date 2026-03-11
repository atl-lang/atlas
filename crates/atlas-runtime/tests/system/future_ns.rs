use super::*;

// ============================================================================
// future namespace dispatch tests (B33)
// Verifies future.resolve, future.reject, future.all, future.race,
// future.allSettled, future.any, future.never, future.delay,
// and Future instance methods: .isResolved(), .isPending(), .isRejected()
// Both interpreter and VM paths tested.
// ============================================================================

// --- future.resolve ---

#[test]
fn test_future_resolve_interpreter() {
    let result = eval_ok("future.resolve(42)");
    assert!(
        matches!(result, Value::Future(_)),
        "Expected Future from future.resolve, got {:?}",
        result
    );
}

#[test]
fn test_future_resolve_vm() {
    let result = eval_ok_vm("future.resolve(42)");
    assert!(
        matches!(result, Value::Future(_)),
        "Expected Future from future.resolve (VM), got {:?}",
        result
    );
}

#[test]
fn test_future_resolve_parity() {
    let i = eval_ok("future.resolve(99)");
    let v = eval_ok_vm("future.resolve(99)");
    assert!(matches!(i, Value::Future(_)));
    assert!(matches!(v, Value::Future(_)));
}

// --- future.reject ---

#[test]
fn test_future_reject_interpreter() {
    let result = eval_ok("future.reject(\"err\")");
    assert!(
        matches!(result, Value::Future(_)),
        "Expected Future from future.reject, got {:?}",
        result
    );
}

#[test]
fn test_future_reject_vm() {
    let result = eval_ok_vm("future.reject(\"err\")");
    assert!(
        matches!(result, Value::Future(_)),
        "Expected Future from future.reject (VM), got {:?}",
        result
    );
}

// --- future.all ---

#[test]
fn test_future_all_interpreter() {
    let result = eval_ok("future.all([future.resolve(1), future.resolve(2)])");
    assert!(
        matches!(result, Value::Future(_)),
        "Expected Future from future.all, got {:?}",
        result
    );
}

#[test]
fn test_future_all_vm() {
    let result = eval_ok_vm("future.all([future.resolve(1), future.resolve(2)])");
    assert!(
        matches!(result, Value::Future(_)),
        "Expected Future from future.all (VM), got {:?}",
        result
    );
}

// --- future.race ---

#[test]
fn test_future_race_interpreter() {
    let result = eval_ok("future.race([future.resolve(1), future.resolve(2)])");
    assert!(
        matches!(result, Value::Future(_)),
        "Expected Future from future.race, got {:?}",
        result
    );
}

#[test]
fn test_future_race_vm() {
    let result = eval_ok_vm("future.race([future.resolve(1), future.resolve(2)])");
    assert!(
        matches!(result, Value::Future(_)),
        "Expected Future from future.race (VM), got {:?}",
        result
    );
}

// --- future.allSettled ---

#[test]
fn test_future_all_settled_interpreter() {
    let result =
        eval_ok("future.allSettled([future.resolve(1), future.reject(\"e\"), future.resolve(3)])");
    assert!(
        matches!(result, Value::Future(_)),
        "Expected Future from future.allSettled, got {:?}",
        result
    );
}

#[test]
fn test_future_all_settled_vm() {
    let result = eval_ok_vm(
        "future.allSettled([future.resolve(1), future.reject(\"e\"), future.resolve(3)])",
    );
    assert!(
        matches!(result, Value::Future(_)),
        "Expected Future from future.allSettled (VM), got {:?}",
        result
    );
}

// --- future.any ---

#[test]
fn test_future_any_interpreter() {
    let result = eval_ok("future.any([future.resolve(1), future.resolve(2)])");
    assert!(
        matches!(result, Value::Future(_)),
        "Expected Future from future.any, got {:?}",
        result
    );
}

#[test]
fn test_future_any_vm() {
    let result = eval_ok_vm("future.any([future.resolve(1), future.resolve(2)])");
    assert!(
        matches!(result, Value::Future(_)),
        "Expected Future from future.any (VM), got {:?}",
        result
    );
}

// --- future.never ---

#[test]
fn test_future_never_interpreter() {
    let result = eval_ok("future.never()");
    assert!(
        matches!(result, Value::Future(_)),
        "Expected Future from future.never, got {:?}",
        result
    );
}

#[test]
fn test_future_never_vm() {
    let result = eval_ok_vm("future.never()");
    assert!(
        matches!(result, Value::Future(_)),
        "Expected Future from future.never (VM), got {:?}",
        result
    );
}

// --- future.delay ---

#[test]
fn test_future_delay_interpreter() {
    let result = eval_ok("future.delay(0)");
    assert!(
        matches!(result, Value::Future(_)),
        "Expected Future from future.delay, got {:?}",
        result
    );
}

#[test]
fn test_future_delay_vm() {
    let result = eval_ok_vm("future.delay(0)");
    assert!(
        matches!(result, Value::Future(_)),
        "Expected Future from future.delay (VM), got {:?}",
        result
    );
}

// --- Future instance method: .isResolved() ---

#[test]
fn test_future_instance_is_resolved_interpreter() {
    let result = eval_ok("let f = future.resolve(1); f.isResolved()");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_future_instance_is_resolved_vm() {
    let result = eval_ok_vm("let f = future.resolve(1); f.isResolved()");
    assert_eq!(result, Value::Bool(true));
}

// --- Future instance method: .isPending() ---

#[test]
fn test_future_instance_is_pending_interpreter() {
    let result = eval_ok("let f = future.never(); f.isPending()");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_future_instance_is_pending_vm() {
    let result = eval_ok_vm("let f = future.never(); f.isPending()");
    assert_eq!(result, Value::Bool(true));
}

// --- Future instance method: .isRejected() ---

#[test]
fn test_future_instance_is_rejected_interpreter() {
    let result = eval_ok("let f = future.reject(\"err\"); f.isRejected()");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_future_instance_is_rejected_vm() {
    let result = eval_ok_vm("let f = future.reject(\"err\"); f.isRejected()");
    assert_eq!(result, Value::Bool(true));
}

// --- Parity sweep: isResolved/isPending/isRejected ---

#[test]
fn test_future_status_parity() {
    let cases = [
        (
            "let f = future.resolve(1); f.isResolved()",
            Value::Bool(true),
        ),
        ("let f = future.never(); f.isPending()", Value::Bool(true)),
        (
            "let f = future.reject(\"e\"); f.isRejected()",
            Value::Bool(true),
        ),
        (
            "let f = future.resolve(1); f.isPending()",
            Value::Bool(false),
        ),
    ];
    for (code, expected) in cases {
        let i = eval_ok(code);
        let v = eval_ok_vm(code);
        assert_eq!(i, expected, "Interpreter: {}", code);
        assert_eq!(v, expected, "VM: {}", code);
    }
}
