use super::*;

// ============================================================================
// task namespace dispatch tests (B31)
// Verifies task.sleep, task.spawn, task.id, task.status, task.cancel
// and channel send/receive dispatch through instance methods.
// Both interpreter and VM paths tested.
// ============================================================================

// --- task.sleep ---

#[test]
fn test_task_sleep_interpreter() {
    // task.sleep(0) should return a Future
    let result = eval_ok("task.sleep(0)");
    assert!(
        matches!(result, Value::Future(_)),
        "Expected Future from task.sleep, got {:?}",
        result
    );
}

#[test]
fn test_task_sleep_vm() {
    let result = eval_ok_vm("task.sleep(0)");
    assert!(
        matches!(result, Value::Future(_)),
        "VM: Expected Future from task.sleep, got {:?}",
        result
    );
}

// --- task.interval ---

#[test]
fn test_task_interval_interpreter() {
    let result = eval_ok("task.interval(100)");
    assert!(
        matches!(result, Value::Future(_)),
        "Expected Future from task.interval, got {:?}",
        result
    );
}

// --- task.spawn (with Future arg) ---

#[test]
fn test_task_spawn_interpreter() {
    // spawn takes a Future, return TaskHandle
    // task.sleep(0) produces a Future, then task.spawn wraps it
    let result = eval_ok("task.spawn(task.sleep(0), null)");
    assert!(
        matches!(result, Value::TaskHandle(_)),
        "Expected TaskHandle from task.spawn, got {:?}",
        result
    );
}

#[test]
fn test_task_spawn_vm() {
    let result = eval_ok_vm("task.spawn(task.sleep(0), null)");
    assert!(
        matches!(result, Value::TaskHandle(_)),
        "VM: Expected TaskHandle from task.spawn, got {:?}",
        result
    );
}

// --- task.id ---

#[test]
fn test_task_id_interpreter() {
    let result = eval_ok("task.id(task.spawn(task.sleep(0), null))");
    assert!(
        matches!(result, Value::Number(_)),
        "Expected Number from task.id, got {:?}",
        result
    );
}

// --- task.status ---

#[test]
fn test_task_status_interpreter() {
    let result = eval_ok("task.status(task.spawn(task.sleep(0), null))");
    assert!(
        matches!(result, Value::String(_)),
        "Expected String from task.status, got {:?}",
        result
    );
}

// --- task.cancel ---

#[test]
fn test_task_cancel_interpreter() {
    let result = eval_ok("task.cancel(task.spawn(task.sleep(0), null))");
    assert!(
        matches!(result, Value::Null),
        "Expected Null from task.cancel, got {:?}",
        result
    );
}

// --- channel instance methods: sender.send / receiver.receive ---

#[test]
fn test_channel_sender_send_interpreter() {
    let code = r#"
        let pair = channelUnbounded();
        let sender = pair[0];
        sender.send(42)
    "#;
    let result = eval_ok(code);
    assert!(
        matches!(result, Value::Bool(_)),
        "Expected Bool from sender.send, got {:?}",
        result
    );
}

// Note: channel instance method VM dispatch requires typechecker to annotate the TypeTag.
// Since channelUnbounded() return type is not registered, pair[0] resolves to Unknown in VM mode.
// This is tracked for future work when channelUnbounded return type is registered.
// The interpreter-mode test above verifies the dispatch wiring works at runtime.

// Note: channelReceiver.receive() calls spawn_local internally which requires a tokio LocalSet.
// These tests are excluded from synchronous test execution — covered in async_runtime test suite.

// --- channel sender.isClosed ---

#[test]
fn test_channel_sender_is_closed_interpreter() {
    let code = r#"
        let pair = channelUnbounded();
        let sender = pair[0];
        sender.isClosed()
    "#;
    let result = eval_ok(code);
    assert!(
        matches!(result, Value::Bool(_)),
        "Expected Bool from sender.isClosed, got {:?}",
        result
    );
}

// --- AsyncMutex instance methods ---

#[test]
fn test_async_mutex_get_interpreter() {
    let code = r#"
        let m = asyncMutex(100);
        m.get()
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Number(100.0), "Expected 100 from mutex.get");
}

#[test]
fn test_async_mutex_set_interpreter() {
    let code = r#"
        let m = asyncMutex(0);
        m.set(42)
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Null, "Expected Null from mutex.set");
}
