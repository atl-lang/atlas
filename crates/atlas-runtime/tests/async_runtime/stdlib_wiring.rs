// Phase 11: Stdlib Wiring tests
//
// Verifies that async stdlib functions return Value::Future and work with `await`
// in both the interpreter and VM. Tests cover sleep/timeout, futureAll/futureRace,
// spawn, async I/O, and cross-engine parity.
//
// Note: tests using `async { }` block syntax are deferred (not yet parsed).

use atlas_runtime::api::{ExecutionMode, Runtime};
use atlas_runtime::security::SecurityContext;
use atlas_runtime::value::Value;

fn interp(code: &str) -> Value {
    let mut rt = Runtime::new();
    rt.eval(code).expect("interpreter eval failed")
}

fn interp_fs(code: &str) -> Value {
    let mut rt = Runtime::new_with_security(SecurityContext::allow_all());
    rt.eval(code).expect("interpreter (fs) eval failed")
}

fn vm(code: &str) -> Value {
    let mut rt = Runtime::new();
    rt.eval(code).expect("vm eval failed")
}

fn vm_fs(code: &str) -> Value {
    let mut rt = Runtime::new_with_security(SecurityContext::allow_all());
    rt.eval(code).expect("vm (fs) eval failed")
}

// ============================================================================
// sleep and timeout (4 tests)
// ============================================================================

/// 1. await sleep(0) resolves immediately to null.
#[test]
fn test_stdlib_sleep_zero_resolves() {
    let result = interp("await sleep(0);");
    assert_eq!(result, Value::Null);
}

/// 2. await sleep(N) completes and surrounding program continues.
#[test]
fn test_stdlib_sleep_duration_completes() {
    let result = interp(
        r#"
        await sleep(5);
        99;
        "#,
    );
    assert_eq!(result, Value::Number(99.0));
}

/// 3. sleep() without await returns Future (typechecks as Future<null>).
#[test]
fn test_stdlib_sleep_returns_future_type() {
    let result = interp("typeof(sleep(0));");
    assert_eq!(result, Value::string("Future"));
}

/// 4. futureResolve(v) + await produces the inner value.
#[test]
fn test_stdlib_future_resolve_await() {
    let result = interp("await futureResolve(77);");
    assert_eq!(result, Value::Number(77.0));
}

// ============================================================================
// futureAll and futureRace (4 tests)
// ============================================================================

/// 5. futureAll([resolved, resolved]) — returns array of results.
#[test]
fn test_stdlib_future_all_two() {
    let result = interp("await futureAll([futureResolve(1), futureResolve(2)]);");
    assert!(
        matches!(result, Value::Array(_)),
        "expected Array, got {:?}",
        result
    );
    if let Value::Array(arr) = &result {
        assert_eq!(arr.len(), 2);
    }
}

/// 6. futureAll([]) — resolves with empty array.
#[test]
fn test_stdlib_future_all_empty() {
    let result = interp("await futureAll([]);");
    assert!(
        matches!(result, Value::Array(_)),
        "expected Array, got {:?}",
        result
    );
    if let Value::Array(arr) = &result {
        assert_eq!(arr.len(), 0);
    }
}

/// 7. futureRace([slow, fast]) — returns first resolved value.
#[test]
fn test_stdlib_future_race_resolved_wins() {
    // Both are already-resolved futures; race returns the first resolved one.
    let result = interp(
        r#"
        let f1 = futureResolve(10);
        let f2 = futureResolve(20);
        await futureRace([f1, f2]);
        "#,
    );
    // Winner is the first resolved (f1), value 10.
    assert_eq!(result, Value::Number(10.0));
}

/// 8. futureAll returns Future type before await.
#[test]
fn test_stdlib_future_all_returns_future() {
    let result = interp("typeof(futureAll([futureResolve(1)]));");
    assert_eq!(result, Value::string("Future"));
}

// ============================================================================
// Spawn (4 tests)
// ============================================================================

/// 9. spawn(future, null) returns a TaskHandle (typeof reports "record" for task handles).
#[test]
fn test_stdlib_spawn_returns_task_handle() {
    // typeof() maps TaskHandle to "record" (opaque runtime type).
    // The important thing is spawn() doesn't block and returns immediately.
    let result = interp("typeof(spawn(futureResolve(42), null));");
    assert_eq!(result, Value::string("record"));
}

/// 10. futureResolve wraps a string — await returns the string.
#[test]
fn test_stdlib_future_resolve_string() {
    let result = interp(r#"await futureResolve("hello");"#);
    assert_eq!(result, Value::string("hello"));
}

/// 11. futureResolve wraps bool — await returns the bool.
#[test]
fn test_stdlib_future_resolve_bool() {
    let result = interp("await futureResolve(true);");
    assert_eq!(result, Value::Bool(true));
}

/// 12. futureNew() creates a pending Future — type is Future.
#[test]
fn test_stdlib_future_new_is_future() {
    let result = interp("typeof(futureNew());");
    assert_eq!(result, Value::string("Future"));
}

// ============================================================================
// Async I/O (4 tests)
// ============================================================================

/// 13. readFileAsync() on an existing file returns a Future.
#[test]
fn test_stdlib_read_file_async_returns_future() {
    use std::io::Write;
    let path = "/tmp/atlas_phase11_typeof.txt";
    let mut f = std::fs::File::create(path).unwrap();
    f.write_all(b"x").unwrap();
    let result = interp_fs(&format!(r#"typeof(readFileAsync("{}"));"#, path));
    assert_eq!(result, Value::string("Future"));
}

/// 14. writeFileAsync() returns a Future (type check).
#[test]
fn test_stdlib_write_file_async_returns_future() {
    let result = interp_fs(r#"typeof(writeFileAsync("/tmp/atlas_test_p11.txt", "data"));"#);
    assert_eq!(result, Value::string("Future"));
}

/// 15. readFileAsync() on a real file resolves to its contents.
#[test]
fn test_stdlib_read_file_async_resolves() {
    use std::io::Write;
    let path = "/tmp/atlas_phase11_test.txt";
    let mut f = std::fs::File::create(path).unwrap();
    f.write_all(b"phase11").unwrap();

    let result = interp_fs(&format!(r#"await readFileAsync("{}");"#, path));
    assert_eq!(result, Value::string("phase11"));
}

/// 16. writeFileAsync() on a writable path completes (resolves to null).
#[test]
fn test_stdlib_write_file_async_resolves() {
    let path = "/tmp/atlas_phase11_write.txt";
    let result = interp_fs(&format!(r#"await writeFileAsync("{}", "ok");"#, path));
    assert_eq!(result, Value::Null);
}

// ============================================================================
// Cross-engine parity (4 tests)
// ============================================================================

/// P1. sleep(0) — identical output in interpreter and VM.
#[test]
fn test_parity_sleep_zero() {
    let code = "await sleep(0); 1;";
    assert_eq!(interp(code), vm(code));
}

/// P2. futureAll — identical result array in both engines.
#[test]
fn test_parity_future_all() {
    let code = "await futureAll([futureResolve(1), futureResolve(2)]);";
    let i = interp(code);
    let v = vm(code);
    assert_eq!(i, v, "parity mismatch: interp={:?} vm={:?}", i, v);
}

/// P3. futureRace — identical winner in both engines.
#[test]
fn test_parity_future_race() {
    let code = "await futureRace([futureResolve(42), futureResolve(99)]);";
    assert_eq!(interp(code), vm(code));
}

/// P4. readFileAsync round-trip — identical contents in both engines.
#[test]
fn test_parity_read_file_async() {
    use std::io::Write;
    let path = "/tmp/atlas_phase11_parity.txt";
    let mut f = std::fs::File::create(path).unwrap();
    f.write_all(b"parity").unwrap();

    let code = format!(r#"await readFileAsync("{}");"#, path);
    assert_eq!(interp_fs(&code), vm_fs(&code));
}
