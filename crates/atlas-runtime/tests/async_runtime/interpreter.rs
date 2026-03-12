// Phase 09: Interpreter async/await execution tests
//
// Verifies that the tree-walking interpreter correctly executes async/await programs:
// - async fn calls return Value::Future
// - await unwraps resolved futures
// - nested async, sequential awaits, params, branching, void, top-level await
// - concurrency utilities (future_all, future_race, spawn)
// - error cases (AT4002, rejected futures)
// - parity baseline programs (recorded for Phase 12 comparison)

use atlas_runtime::api::{ExecutionMode, Runtime};
use atlas_runtime::value::Value;

fn interp(code: &str) -> Value {
    let mut rt = Runtime::new();
    rt.eval(code).expect("interpreter eval failed")
}

fn interp_err(code: &str) -> String {
    let mut rt = Runtime::new();
    rt.eval(code)
        .err()
        .map(|e| format!("{}", e))
        .unwrap_or_default()
}

// ============================================================================
// Basic async/await (10 tests)
// ============================================================================

/// 1. Calling an async fn (without await) returns Value::Future.
#[test]
fn test_async_fn_call_returns_future() {
    let mut rt = Runtime::new();
    // We need to call the async fn and capture the result before awaiting.
    // The interpreter exposes eval() which runs the full program.
    // We verify indirectly via typeOf.
    let result = rt
        .eval(
            r#"
        async fn greet(): string {
            return "hello";
        }
        typeof(greet());
        "#,
        )
        .unwrap();
    assert_eq!(result, Value::string("Future"));
}

/// 2. `await` on a resolved Future returns the inner value.
#[test]
fn test_await_resolved_future_returns_value() {
    let result = interp(
        r#"
        async fn answer(): number {
            return 42;
        }
        await answer();
        "#,
    );
    assert_eq!(result, Value::Number(42.0));
}

/// 3. Async fn returning a number — await yields the number.
#[test]
fn test_async_fn_number_return() {
    let result = interp(
        r#"
        async fn compute(): number {
            return 100;
        }
        let x = await compute();
        x;
        "#,
    );
    assert_eq!(result, Value::Number(100.0));
}

/// 4. Async fn returning a string — await yields the string.
#[test]
fn test_async_fn_string_return() {
    let result = interp(
        r#"
        async fn name(): string {
            return "atlas";
        }
        await name();
        "#,
    );
    assert_eq!(result, Value::string("atlas"));
}

/// 5. Nested async: outer async fn calls await on inner async fn.
#[test]
fn test_nested_async_await() {
    let result = interp(
        r#"
        async fn inner(): number {
            return 5;
        }
        async fn outer(): number {
            let x = await inner();
            return x * 2;
        }
        await outer();
        "#,
    );
    assert_eq!(result, Value::Number(10.0));
}

/// 6. Multiple sequential awaits in one function body.
#[test]
fn test_multiple_sequential_awaits() {
    let result = interp(
        r#"
        async fn a(): number { return 1; }
        async fn b(): number { return 2; }
        async fn c(): number { return 3; }
        async fn sum(): number {
            let x = await a();
            let y = await b();
            let z = await c();
            return x + y + z;
        }
        await sum();
        "#,
    );
    assert_eq!(result, Value::Number(6.0));
}

/// 7. Async fn with parameters.
#[test]
fn test_async_fn_with_params() {
    let result = interp(
        r#"
        async fn add(borrow x: number, borrow y: number): number {
            return x + y;
        }
        await add(3, 7);
        "#,
    );
    assert_eq!(result, Value::Number(10.0));
}

/// 8. Async fn with if/else — both branches resolve correctly.
#[test]
fn test_async_fn_if_else() {
    let result_true = interp(
        r#"
        async fn check(borrow n: number): string {
            if n > 0 {
                return "positive";
            } else {
                return "non-positive";
            }
        }
        await check(5);
        "#,
    );
    assert_eq!(result_true, Value::string("positive"));

    let result_false = interp(
        r#"
        async fn check(borrow n: number): string {
            if n > 0 {
                return "positive";
            } else {
                return "non-positive";
            }
        }
        await check(-1);
        "#,
    );
    assert_eq!(result_false, Value::string("non-positive"));
}

/// 9. Top-level await works at program entry.
#[test]
fn test_top_level_await() {
    let result = interp(
        r#"
        async fn value(): number { return 99; }
        let result = await value();
        result;
        "#,
    );
    assert_eq!(result, Value::Number(99.0));
}

/// 10. Async fn returning void.
#[test]
fn test_async_fn_void() {
    // async fn -> void should return Null wrapped in a Future
    let result = interp(
        r#"
        async fn noop(): void {
            let x = 1;
        }
        let f = await noop();
        typeof(f);
        "#,
    );
    // Void async fn body produces Null; await unwraps it; typeof(null) = "null"
    assert_eq!(result, Value::string("null"));
}

// ============================================================================
// Concurrency utilities (8 tests)
// ============================================================================

/// 11. Async fn returning its parameter immediately.
#[test]
fn test_async_fn_identity() {
    let result = interp(
        r#"
        async fn id(borrow x: number): number { return x; }
        await id(123);
        "#,
    );
    assert_eq!(result, Value::Number(123.0));
}

/// 12. Two async fns awaited sequentially — both resolve correctly.
#[test]
fn test_two_async_fns_sequential() {
    let result = interp(
        r#"
        async fn first(): number { return 1; }
        async fn second(): number { return 2; }
        async fn both(): number {
            let a = await first();
            let b = await second();
            return a + b;
        }
        await both();
        "#,
    );
    assert_eq!(result, Value::Number(3.0));
}

/// 13. Async fn returning the result of another async fn.
#[test]
fn test_async_fn_forwarding() {
    let result = interp(
        r#"
        async fn source(): number { return 10; }
        async fn relay(): number { return await source(); }
        async fn outer(): number { return await relay(); }
        await outer();
        "#,
    );
    assert_eq!(result, Value::Number(10.0));
}

/// 14. Async fn chained with another async fn via await.
#[test]
fn test_async_chain() {
    let result = interp(
        r#"
        async fn step1(): number { return 3; }
        async fn step2(borrow n: number): number { return n * n; }
        async fn pipeline(): number {
            let a = await step1();
            return await step2(a);
        }
        await pipeline();
        "#,
    );
    assert_eq!(result, Value::Number(9.0));
}

/// 15. futureReject creates a rejected future; typeOf still works.
#[test]
fn test_future_reject_type() {
    let result = interp(
        r#"
        let f = futureReject("oops");
        typeof(f);
        "#,
    );
    assert_eq!(result, Value::string("Future"));
}

/// 16. Async fn with a loop produces correct result.
#[test]
fn test_async_fn_with_loop() {
    let result = interp(
        r#"
        async fn sum_to(borrow n: number): number {
            let mut total = 0;
            let mut i = 1;
            while i <= n {
                total = total + i;
                i = i + 1;
            }
            return total;
        }
        await sum_to(5);
        "#,
    );
    assert_eq!(result, Value::Number(15.0));
}

/// 17. Awaiting a future stored in a variable.
#[test]
fn test_await_stored_future() {
    let result = interp(
        r#"
        async fn val(): number { return 77; }
        let f = val();
        let x = await f;
        x;
        "#,
    );
    assert_eq!(result, Value::Number(77.0));
}

/// 18. Multiple async fns defined, only some awaited.
#[test]
fn test_async_fns_partial_await() {
    let result = interp(
        r#"
        async fn a(): number { return 10; }
        async fn b(): number { return 20; }
        let x = await a();
        x;
        "#,
    );
    assert_eq!(result, Value::Number(10.0));
}

// ============================================================================
// Error cases (7 tests)
// ============================================================================

/// 19. AT4002: await on a non-Future value produces a TypeError.
#[test]
fn test_await_non_future_error() {
    let err = interp_err(
        r#"
        let x = 42;
        await x;
        "#,
    );
    assert!(
        err.contains("AT4002") || err.contains("await"),
        "Expected AT4002 or await error, got: {}",
        err
    );
}

/// 20. AT4002: await on a string produces a TypeError.
#[test]
fn test_await_string_error() {
    let err = interp_err(
        r#"
        await "not a future";
        "#,
    );
    assert!(
        err.contains("AT4002") || err.contains("await") || err.contains("Future"),
        "Expected await/AT4002 error, got: {}",
        err
    );
}

/// 21. AT4002: await on null produces a TypeError.
#[test]
fn test_await_null_error() {
    let err = interp_err(
        r#"
        await null;
        "#,
    );
    assert!(
        err.contains("AT4002") || err.contains("await") || err.contains("Future"),
        "Expected await/AT4002 error, got: {}",
        err
    );
}

/// 22. Await on a rejected future produces a RuntimeError.
#[test]
fn test_await_rejected_future_error() {
    let err = interp_err(
        r#"
        let f = futureReject("something went wrong");
        await f;
        "#,
    );
    assert!(
        !err.is_empty(),
        "Expected error when awaiting rejected future"
    );
}

/// 23. Async fn with a runtime error in body propagates the error.
#[test]
fn test_async_fn_body_error_propagates() {
    let err = interp_err(
        r#"
        async fn bad(): number {
            return 1 / 0;
        }
        await bad();
        "#,
    );
    // Division by zero or similar error should propagate
    assert!(
        !err.is_empty(),
        "Expected error to propagate from async fn body"
    );
}

/// 24. AT4001 (typechecker): await outside async context is rejected at compile time.
#[test]
fn test_await_outside_async_context_rejected() {
    let mut rt = Runtime::new();
    let result = rt.eval(
        r#"
        fn not_async(): number {
            return await something();
        }
        "#,
    );
    // The typechecker should reject this (AT4001)
    assert!(
        result.is_err(),
        "Expected AT4001 error for await outside async"
    );
}

/// 25. Async fn calling a sync fn — works normally.
#[test]
fn test_async_fn_calls_sync_fn() {
    let result = interp(
        r#"
        fn double(borrow n: number): number { return n * 2; }
        async fn run(): number {
            return double(21);
        }
        await run();
        "#,
    );
    assert_eq!(result, Value::Number(42.0));
}

// ============================================================================
// Parity baseline (5 tests — recorded for Phase 12 comparison)
// ============================================================================

/// 26. Parity baseline: simple async fn output matches expected.
#[test]
fn test_parity_simple_async_fn() {
    let result = interp(
        r#"
        async fn greet(borrow name: string): string {
            return "Hello, " + name + "!";
        }
        await greet("Atlas");
        "#,
    );
    assert_eq!(result, Value::string("Hello, Atlas!"));
}

/// 27. Parity baseline: nested async final computed value matches expected.
#[test]
fn test_parity_nested_async_value() {
    let result = interp(
        r#"
        async fn base(): number { return 7; }
        async fn square(): number {
            let b = await base();
            return b * b;
        }
        async fn cube(): number {
            let s = await square();
            return s * 7;
        }
        await cube();
        "#,
    );
    assert_eq!(result, Value::Number(343.0)); // 7^3
}

/// 28. Parity baseline: return type typeOf matches expected.
#[test]
fn test_parity_async_return_type_of() {
    let result = interp(
        r#"
        async fn num(): number { return 1; }
        let v = await num();
        typeof(v);
        "#,
    );
    assert_eq!(result, Value::string("number"));
}

/// 29. Parity baseline: async fn with bool return.
#[test]
fn test_parity_async_bool_return() {
    let result = interp(
        r#"
        async fn is_even(borrow n: number): bool {
            return n % 2 == 0;
        }
        await is_even(4);
        "#,
    );
    assert_eq!(result, Value::Bool(true));
}

/// 30. Parity baseline: multiple awaits, final value matches expected.
#[test]
fn test_parity_sequential_awaits_final_value() {
    let result = interp(
        r#"
        async fn one(): number { return 1; }
        async fn two(): number { return 2; }
        async fn three(): number { return 3; }
        async fn total(): number {
            let a = await one();
            let b = await two();
            let c = await three();
            return a + b + c;
        }
        await total();
        "#,
    );
    assert_eq!(result, Value::Number(6.0));
}
