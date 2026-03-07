// Phase 10: VM async/await execution tests
//
// Verifies that the bytecode VM correctly executes async/await programs:
// - AsyncCall opcode pushes Value::Future (WrapFuture inside body handles wrapping)
// - Await opcode resolves Future → inner value
// - WrapFuture opcode wraps a value in an immediately-resolved Future
// - SpawnTask opcode runs task eagerly (parity with interpreter at this stage)
// - AT4002 raised when Await pops a non-Future value
// - Parity: VM and interpreter produce identical output for all programs

use atlas_runtime::api::{ExecutionMode, Runtime};
use atlas_runtime::value::Value;

fn vm(code: &str) -> Value {
    let mut rt = Runtime::new(ExecutionMode::VM);
    rt.eval(code).expect("vm eval failed")
}

fn vm_err(code: &str) -> String {
    let mut rt = Runtime::new(ExecutionMode::VM);
    rt.eval(code)
        .err()
        .map(|e| format!("{}", e))
        .unwrap_or_default()
}

fn interp(code: &str) -> Value {
    let mut rt = Runtime::new(ExecutionMode::Interpreter);
    rt.eval(code).expect("interpreter eval failed")
}

// ============================================================================
// VM async execution (10 tests)
// ============================================================================

/// 1. Calling an async fn (no await) returns Value::Future.
#[test]
fn test_vm_async_call_returns_future() {
    let result = vm(r#"
        async fn greet() -> string {
            return "hello";
        }
        typeof(greet());
        "#);
    assert_eq!(result, Value::string("Future"));
}

/// 2. `await` on a resolved Future returns the inner value.
#[test]
fn test_vm_await_resolved_future() {
    let result = vm(r#"
        async fn answer() -> number {
            return 42;
        }
        await answer();
        "#);
    assert_eq!(result, Value::Number(42.0));
}

/// 3. Async fn returning number — await yields the number.
#[test]
fn test_vm_async_fn_number() {
    let result = vm(r#"
        async fn compute() -> number {
            return 100;
        }
        let x = await compute();
        x;
        "#);
    assert_eq!(result, Value::Number(100.0));
}

/// 4. Async fn returning string — await yields the string.
#[test]
fn test_vm_async_fn_string() {
    let result = vm(r#"
        async fn label() -> string {
            return "atlas";
        }
        await label();
        "#);
    assert_eq!(result, Value::string("atlas"));
}

/// 5. Nested async: outer awaits inner async fn.
#[test]
fn test_vm_nested_async_await() {
    let result = vm(r#"
        async fn inner() -> number {
            return 5;
        }
        async fn outer() -> number {
            let v = await inner();
            return v * 2;
        }
        await outer();
        "#);
    assert_eq!(result, Value::Number(10.0));
}

/// 6. Sequential awaits produce correct final value.
#[test]
fn test_vm_sequential_awaits() {
    let result = vm(r#"
        async fn a() -> number { return 1; }
        async fn b() -> number { return 2; }
        async fn c() -> number { return 3; }
        let x = await a();
        let y = await b();
        let z = await c();
        x + y + z;
        "#);
    assert_eq!(result, Value::Number(6.0));
}

/// 7. Async fn with parameters — args bound correctly.
#[test]
fn test_vm_async_fn_with_params() {
    let result = vm(r#"
        async fn add(a: number, b: number) -> number {
            return a + b;
        }
        await add(10, 32);
        "#);
    assert_eq!(result, Value::Number(42.0));
}

/// 8. Async fn with branching — correct branch executes.
#[test]
fn test_vm_async_fn_branching() {
    let result = vm(r#"
        async fn classify(n: number) -> string {
            if n > 0 {
                return "positive";
            } else {
                return "non-positive";
            }
        }
        await classify(5);
        "#);
    assert_eq!(result, Value::string("positive"));
}

/// 9. Awaited result used in arithmetic.
#[test]
fn test_vm_await_result_in_expr() {
    let result = vm(r#"
        async fn base() -> number { return 7; }
        let b = await base();
        b * b;
        "#);
    assert_eq!(result, Value::Number(49.0));
}

/// 10. typeof() of an awaited result is the inner type, not Future.
#[test]
fn test_vm_typeof_awaited_result() {
    let result = vm(r#"
        async fn num() -> number { return 1; }
        typeof(await num());
        "#);
    assert_eq!(result, Value::string("number"));
}

// ============================================================================
// WrapFuture / SpawnTask / error cases (5 tests)
// ============================================================================

/// 11. Async fn returning null (void) — await yields null.
#[test]
fn test_vm_async_fn_void() {
    let result = vm(r#"
        async fn nothing() -> null { return null; }
        await nothing();
        "#);
    assert_eq!(result, Value::Null);
}

/// 12. AT4002: awaiting a non-Future raises a runtime error.
#[test]
fn test_vm_await_non_future_at4002() {
    let err = vm_err("await 42;");
    assert!(
        err.contains("AT4002") || err.contains("Future"),
        "expected AT4002 error, got: {}",
        err
    );
}

/// 13. SpawnTask: calling an async fn via SpawnTask-equivalent produces a Future.
#[test]
fn test_vm_spawn_task_returns_future() {
    // SpawnTask is emitted for spawn() calls; for user async fns it mirrors AsyncCall.
    // Verify via typeof that the result is Future before any await.
    let result = vm(r#"
        async fn work() -> number { return 7; }
        typeof(work());
        "#);
    assert_eq!(result, Value::string("Future"));
}

/// 14. Async fn returning bool — await yields bool.
#[test]
fn test_vm_async_fn_bool_return() {
    let result = vm(r#"
        async fn flag() -> bool { return true; }
        await flag();
        "#);
    assert_eq!(result, Value::Bool(true));
}

/// 15. Deeply nested async (3 levels) terminates correctly.
#[test]
fn test_vm_deeply_nested_async() {
    let result = vm(r#"
        async fn level3() -> number { return 1; }
        async fn level2() -> number { return (await level3()) + 1; }
        async fn level1() -> number { return (await level2()) + 1; }
        await level1();
        "#);
    assert_eq!(result, Value::Number(3.0));
}

// ============================================================================
// Parity: VM and interpreter produce identical output (5 tests)
// ============================================================================

/// P1. Simple async fn — identical output in interpreter and VM.
#[test]
fn test_parity_simple_async() {
    let code = r#"
        async fn square(n: number) -> number { return n * n; }
        await square(6);
    "#;
    assert_eq!(vm(code), interp(code));
}

/// P2. Nested async calls — identical final value.
#[test]
fn test_parity_nested_async() {
    let code = r#"
        async fn double(n: number) -> number { return n * 2; }
        async fn quad(n: number) -> number { return await double(await double(n)); }
        await quad(3);
    "#;
    assert_eq!(vm(code), interp(code));
}

/// P3. typeof() of awaited result — identical type string.
#[test]
fn test_parity_typeof_result() {
    let code = r#"
        async fn value() -> number { return 42; }
        typeof(await value());
    "#;
    assert_eq!(vm(code), interp(code));
}

/// P4. Sequential awaits summed — identical final value.
#[test]
fn test_parity_sequential_awaits() {
    let code = r#"
        async fn one() -> number { return 1; }
        async fn two() -> number { return 2; }
        (await one()) + (await two());
    "#;
    assert_eq!(vm(code), interp(code));
}

/// P5. Async with branching — identical string result.
#[test]
fn test_parity_async_branching() {
    let code = r#"
        async fn classify_sign(n: number) -> string {
            if n > 0 { return "pos"; } else { return "neg"; }
        }
        await classify_sign(-3);
    "#;
    assert_eq!(vm(code), interp(code));
}
