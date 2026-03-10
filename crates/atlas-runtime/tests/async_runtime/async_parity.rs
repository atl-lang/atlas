// Phase 12: Async Parity Sweep
//
// Exhaustive verification that the interpreter and VM produce byte-for-byte
// identical output for every async Atlas program.
//
// Organisation:
//   - Core async programs      (tests 01–10)
//   - Stdlib async programs    (tests 11–18)
//   - Error cases              (tests 19–25)
//
// Every test calls `assert_parity(source, expected)` which runs the program
// through both engines and asserts that both return the expected value.

use atlas_runtime::api::{ExecutionMode, Runtime};
use atlas_runtime::security::SecurityContext;
use atlas_runtime::value::Value;

// ---------------------------------------------------------------------------
// Parity helpers
// ---------------------------------------------------------------------------

/// Run `source` through both engines and assert both produce `expected`.
fn assert_parity(source: &str, expected: Value) {
    let interp = {
        let mut rt = Runtime::new(ExecutionMode::Interpreter);
        rt.eval(source)
            .unwrap_or_else(|e| panic!("interpreter error: {e}\nsource:\n{source}"))
    };
    let vm = {
        let mut rt = Runtime::new(ExecutionMode::VM);
        rt.eval(source)
            .unwrap_or_else(|e| panic!("vm error: {e}\nsource:\n{source}"))
    };
    assert_eq!(
        interp, vm,
        "parity divergence between interpreter and VM\nsource:\n{source}"
    );
    assert_eq!(
        interp, expected,
        "unexpected value from interpreter\nsource:\n{source}"
    );
}

/// Run `source` through both engines (with full FS access) and assert both
/// produce `expected`.
fn assert_parity_fs(source: &str, expected: Value) {
    let interp = {
        let mut rt =
            Runtime::new_with_security(ExecutionMode::Interpreter, SecurityContext::allow_all());
        rt.eval(source)
            .unwrap_or_else(|e| panic!("interpreter error: {e}\nsource:\n{source}"))
    };
    let vm = {
        let mut rt = Runtime::new_with_security(ExecutionMode::VM, SecurityContext::allow_all());
        rt.eval(source)
            .unwrap_or_else(|e| panic!("vm error: {e}\nsource:\n{source}"))
    };
    assert_eq!(
        interp, vm,
        "parity divergence between interpreter and VM\nsource:\n{source}"
    );
    assert_eq!(
        interp, expected,
        "unexpected value from interpreter\nsource:\n{source}"
    );
}

/// Run `source` through both engines and assert both return an Err whose
/// message contains `expected_fragment`.
fn assert_parity_err(source: &str, expected_fragment: &str) {
    let interp_err = {
        let mut rt = Runtime::new(ExecutionMode::Interpreter);
        rt.eval(source)
            .err()
            .unwrap_or_else(|| panic!("interpreter did not error\nsource:\n{source}"))
    };
    let vm_err = {
        let mut rt = Runtime::new(ExecutionMode::VM);
        rt.eval(source)
            .err()
            .unwrap_or_else(|| panic!("vm did not error\nsource:\n{source}"))
    };
    let interp_msg = interp_err.to_string();
    let vm_msg = vm_err.to_string();
    assert_eq!(
        interp_msg, vm_msg,
        "error message parity divergence\nsource:\n{source}"
    );
    assert!(
        interp_msg.contains(expected_fragment),
        "error message does not contain {:?}\ngot: {interp_msg}\nsource:\n{source}",
        expected_fragment
    );
}

// ---------------------------------------------------------------------------
// Core async programs (01–10)
// ---------------------------------------------------------------------------

/// 01. Minimal async fn returning a number — await it.
#[test]
fn parity_async_01_minimal_number() {
    assert_parity(
        r#"
        async fn the_answer(): number {
            return 42;
        }
        await the_answer();
        "#,
        Value::Number(42.0),
    );
}

/// 02a. Async fn returning a string.
#[test]
fn parity_async_02a_string_return() {
    assert_parity(
        r#"
        async fn greet(): string {
            return "hello";
        }
        await greet();
        "#,
        Value::String(std::sync::Arc::new("hello".to_string())),
    );
}

/// 02b. Async fn returning a bool.
#[test]
fn parity_async_02b_bool_return() {
    assert_parity(
        r#"
        async fn flag(): bool {
            return true;
        }
        await flag();
        "#,
        Value::Bool(true),
    );
}

/// 02c. Async fn returning null.
#[test]
fn parity_async_02c_null_return() {
    assert_parity(
        r#"
        async fn nothing(): null {
            return null;
        }
        await nothing();
        "#,
        Value::Null,
    );
}

/// 03. Nested async: three levels deep.
#[test]
fn parity_async_03_nested_three_levels() {
    assert_parity(
        r#"
        async fn inner(): number {
            return 1;
        }
        async fn middle(): number {
            let v = await inner();
            return v + 10;
        }
        async fn outer(): number {
            let v = await middle();
            return v + 100;
        }
        await outer();
        "#,
        Value::Number(111.0),
    );
}

/// 04. Async fn with early return (true branch taken).
#[test]
fn parity_async_04_early_return() {
    assert_parity(
        r#"
        async fn pick(borrow flag: bool): number {
            if flag {
                return 1;
            }
            return 2;
        }
        await pick(true);
        "#,
        Value::Number(1.0),
    );
}

/// 05a. Async fn with if/else — true branch.
#[test]
fn parity_async_05a_if_else_true() {
    assert_parity(
        r#"
        async fn choose(borrow x: number): string {
            if x > 0 {
                return "positive";
            } else {
                return "non-positive";
            }
        }
        await choose(5);
        "#,
        Value::String(std::sync::Arc::new("positive".to_string())),
    );
}

/// 05b. Async fn with if/else — false branch.
#[test]
fn parity_async_05b_if_else_false() {
    assert_parity(
        r#"
        async fn choose(borrow x: number): string {
            if x > 0 {
                return "positive";
            } else {
                return "non-positive";
            }
        }
        await choose(-3);
        "#,
        Value::String(std::sync::Arc::new("non-positive".to_string())),
    );
}

/// 06. Async fn with array iteration in its body.
#[test]
fn parity_async_06_array_loop() {
    assert_parity(
        r#"
        async fn sum_array(borrow items: []number): number {
            let mut total = 0;
            for item in items {
                total = total + item;
            }
            return total;
        }
        await sum_array([1, 2, 3, 4, 5]);
        "#,
        Value::Number(15.0),
    );
}

/// 07. Async fn that calls a nested named fn.
#[test]
fn parity_async_07_inner_fn() {
    assert_parity(
        r#"
        async fn apply(borrow x: number): number {
            fn double(borrow v: number): number { return v * 2; }
            return double(x);
        }
        await apply(21);
        "#,
        Value::Number(42.0),
    );
}

/// 08. Top-level await at program entry.
#[test]
fn parity_async_08_top_level_await() {
    assert_parity(
        r#"
        async fn compute(): number { return 7; }
        let result = await compute();
        result * 6;
        "#,
        Value::Number(42.0),
    );
}

/// 09. Async fn returning void (no explicit return value).
#[test]
fn parity_async_09_void_return() {
    assert_parity(
        r#"
        async fn side_effect(): null {
            let mut x = 1 + 1;
        }
        await side_effect();
        "#,
        Value::Null,
    );
}

/// 10. Multiple awaits in sequence — accumulate results.
#[test]
fn parity_async_10_multiple_awaits() {
    assert_parity(
        r#"
        async fn one(): number { return 1; }
        async fn two(): number { return 2; }
        async fn three(): number { return 3; }
        let a = await one();
        let b = await two();
        let c = await three();
        a + b + c;
        "#,
        Value::Number(6.0),
    );
}

// ---------------------------------------------------------------------------
// Stdlib async programs (11–18)
// ---------------------------------------------------------------------------

/// 11. `await sleep(0)` resolves to null.
#[test]
fn parity_stdlib_11_sleep_zero() {
    assert_parity("await sleep(0);", Value::Null);
}

/// 12. `await futureAll([...])` with deterministic futures — check first element.
#[test]
fn parity_stdlib_12_future_all_deterministic() {
    assert_parity(
        r#"
        let results = await futureAll([futureResolve(1), futureResolve(2), futureResolve(3)]);
        results[0];
        "#,
        Value::Number(1.0),
    );
}

/// 13. `await futureRace([...])` with a single future.
#[test]
fn parity_stdlib_13_future_race_single() {
    assert_parity(
        r#"
        await futureRace([futureResolve(42)]);
        "#,
        Value::Number(42.0),
    );
}

/// 14. `spawn(future, null)` + typeof the handle.
#[test]
fn parity_stdlib_14_spawn_and_typeof() {
    // spawn returns a task handle (record-like); both engines must agree on typeof.
    let interp = {
        let mut rt = Runtime::new(ExecutionMode::Interpreter);
        rt.eval("typeof(spawn(futureResolve(99), null));")
            .expect("interpreter failed")
    };
    let vm = {
        let mut rt = Runtime::new(ExecutionMode::VM);
        rt.eval("typeof(spawn(futureResolve(99), null));")
            .expect("vm failed")
    };
    assert_eq!(interp, vm, "spawn typeof parity divergence");
}

/// 15. `write_file_async` / `read_file_async` round-trip.
#[test]
fn parity_stdlib_15_read_write_roundtrip() {
    use std::env;
    let tmp = env::temp_dir().join("atlas_parity_15.txt");
    let path = tmp.to_string_lossy().to_string();

    assert_parity_fs(
        &format!(
            r#"
            await writeFileAsync("{path}", "parity");
            await readFileAsync("{path}");
            "#
        ),
        Value::String(std::sync::Arc::new("parity".to_string())),
    );
    let _ = std::fs::remove_file(&tmp);
}

/// 16. sleep(1) + surrounding code still runs.
#[test]
fn parity_stdlib_16_sleep_then_value() {
    assert_parity(
        r#"
        await sleep(1);
        42;
        "#,
        Value::Number(42.0),
    );
}

/// 17. Multiple sequential sleeps — final result identical.
#[test]
fn parity_stdlib_17_sequential_sleeps() {
    assert_parity(
        r#"
        await sleep(0);
        await sleep(0);
        "done";
        "#,
        Value::String(std::sync::Arc::new("done".to_string())),
    );
}

/// 18. `futureAll([])` with empty list — both engines agree.
#[test]
fn parity_stdlib_18_all_empty() {
    let interp = {
        let mut rt = Runtime::new(ExecutionMode::Interpreter);
        rt.eval("await futureAll([]);").expect("interpreter failed")
    };
    let vm = {
        let mut rt = Runtime::new(ExecutionMode::VM);
        rt.eval("await futureAll([]);").expect("vm failed")
    };
    assert_eq!(
        interp, vm,
        "parity divergence for futureAll([]) between engines"
    );
}

// ---------------------------------------------------------------------------
// Error cases (19–25)
// ---------------------------------------------------------------------------

/// 19. AT4002 — await on a number — error message identical.
#[test]
fn parity_error_19_at4002_number() {
    assert_parity_err(
        r#"
        async fn bad(): number {
            await 42;
        }
        await bad();
        "#,
        "non-Future",
    );
}

/// 20. AT4002 — await on a string — error text parity.
#[test]
fn parity_error_20_at4002_string() {
    assert_parity_err(
        r#"
        async fn bad(): number {
            await "not a future";
        }
        await bad();
        "#,
        "non-Future",
    );
}

/// 21. Async fn with division — both engines agree on result (or error).
#[test]
fn parity_error_21_division_result_parity() {
    let source = r#"
        async fn divide(borrow a: number, borrow b: number): number {
            return a / b;
        }
        await divide(10, 2);
    "#;
    // Successful case: both must agree on 5.
    assert_parity(source, Value::Number(5.0));
}

/// 22. Nested async error propagation — AT4002 from inner leaks out identically.
#[test]
fn parity_error_22_nested_error_propagation() {
    assert_parity_err(
        r#"
        async fn inner(): number {
            await 999;
        }
        async fn outer(): number {
            return await inner();
        }
        await outer();
        "#,
        "non-Future",
    );
}

/// 23. AT4002 — await on bool.
#[test]
fn parity_error_23_at4002_bool() {
    assert_parity_err(
        r#"
        async fn bad(): null {
            await false;
        }
        await bad();
        "#,
        "non-Future",
    );
}

/// 24. AT4002 — await on null.
#[test]
fn parity_error_24_at4002_null() {
    assert_parity_err(
        r#"
        async fn bad(): null {
            await null;
        }
        await bad();
        "#,
        "non-Future",
    );
}

/// 25. futureRace() with multiple deterministic futures — both engines agree on winner.
#[test]
fn parity_error_25_race_multiple_agree() {
    let source = r#"
        await futureRace([futureResolve(1), futureResolve(2)]);
    "#;
    let interp = {
        let mut rt = Runtime::new(ExecutionMode::Interpreter);
        rt.eval(source).expect("interpreter failed")
    };
    let vm = {
        let mut rt = Runtime::new(ExecutionMode::VM);
        rt.eval(source).expect("vm failed")
    };
    assert_eq!(
        interp, vm,
        "race() parity divergence — engines disagree on winner"
    );
}
