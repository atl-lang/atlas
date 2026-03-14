use super::*;

// ============================================================================
// sync namespace factory tests — interpreter via eval_ok
// ============================================================================

#[test]
fn test_sync_atomic_create() {
    let result = eval_ok("sync.atomic(0)");
    // atomic returns an opaque handle (Array with tag + id)
    assert!(matches!(result, Value::Array(_)));
}

#[test]
fn test_sync_rwlock_create() {
    let result = eval_ok("sync.rwLock(42)");
    assert!(matches!(result, Value::Array(_)));
}

#[test]
fn test_sync_semaphore_create() {
    let result = eval_ok("sync.semaphore(3)");
    assert!(matches!(result, Value::Array(_)));
}

// ============================================================================
// Atomic instance methods via namespace dispatch
// ============================================================================

#[test]
fn test_atomic_get_set() {
    let result = eval_ok(
        r#"
        let a = sync.atomic(10);
        a.get()
    "#,
    );
    assert_eq!(result, Value::Number(10.0));
}

#[test]
fn test_atomic_add_returns_previous() {
    let result = eval_ok(
        r#"
        let a = sync.atomic(5);
        a.add(3)
    "#,
    );
    // fetch_add returns previous value (5)
    assert_eq!(result, Value::Number(5.0));
}

#[test]
fn test_atomic_sub_returns_previous() {
    let result = eval_ok(
        r#"
        let a = sync.atomic(10);
        a.sub(4)
    "#,
    );
    assert_eq!(result, Value::Number(10.0));
}

#[test]
fn test_atomic_store() {
    let result = eval_ok(
        r#"
        let a = sync.atomic(0);
        a.set(99);
        a.get()
    "#,
    );
    assert_eq!(result, Value::Number(99.0));
}

#[test]
fn test_atomic_compare_exchange_success() {
    let result = eval_ok(
        r#"
        let a = sync.atomic(5);
        a.compareSwap(5, 10)
    "#,
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_atomic_compare_exchange_fail() {
    let result = eval_ok(
        r#"
        let a = sync.atomic(5);
        a.compareSwap(99, 10)
    "#,
    );
    assert_eq!(result, Value::Bool(false));
}

// ============================================================================
// RwLock instance methods via stdlib dispatch
// ============================================================================

#[test]
fn test_rwlock_read_returns_value() {
    let result = eval_ok(
        r#"
        let lock = sync.rwLock(42);
        lock.read()
    "#,
    );
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_rwlock_write_updates_value() {
    let result = eval_ok(
        r#"
        let lock = sync.rwLock(1);
        lock.write(99);
        lock.read()
    "#,
    );
    assert_eq!(result, Value::Number(99.0));
}

#[test]
fn test_rwlock_try_read_returns_option() {
    let result = eval_ok(
        r#"
        let lock = sync.rwLock("hello");
        lock.tryRead()
    "#,
    );
    assert!(matches!(result, Value::Option(Some(_))));
}

#[test]
fn test_rwlock_try_write_returns_bool() {
    let result = eval_ok(
        r#"
        let lock = sync.rwLock(0);
        lock.tryWrite(5)
    "#,
    );
    assert_eq!(result, Value::Bool(true));
}

// ============================================================================
// Semaphore instance methods via stdlib dispatch
// ============================================================================

#[test]
fn test_semaphore_available_initial() {
    let result = eval_ok(
        r#"
        let s = sync.semaphore(3);
        s.available()
    "#,
    );
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn test_semaphore_try_acquire_success() {
    let result = eval_ok(
        r#"
        let s = sync.semaphore(2);
        s.tryAcquire()
    "#,
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_semaphore_try_acquire_fail_when_zero() {
    let result = eval_ok(
        r#"
        let s = sync.semaphore(1);
        s.tryAcquire();
        s.tryAcquire()
    "#,
    );
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_semaphore_release_restores_permit() {
    let result = eval_ok(
        r#"
        let s = sync.semaphore(1);
        s.tryAcquire();
        s.release();
        s.available()
    "#,
    );
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_semaphore_acquire_and_release() {
    let result = eval_ok(
        r#"
        let s = sync.semaphore(2);
        s.acquire();
        s.available()
    "#,
    );
    assert_eq!(result, Value::Number(1.0));
}
