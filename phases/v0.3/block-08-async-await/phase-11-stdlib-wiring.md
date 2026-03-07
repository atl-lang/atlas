# Phase 11: Stdlib Wiring

## Dependencies

**Required:** Phase 09 (interpreter), Phase 10 (VM)

**Verification:**
```bash
grep "async_io\|future\|async_primitives\|spawn\|sleep\|timeout" crates/atlas-runtime/src/stdlib/mod.rs
cargo check -p atlas-runtime
```

---

## Objective

Wire the existing async stdlib modules (`async_io`, `future`, `async_primitives`) to the language-level async/await system. Atlas programs should be able to call `sleep()`, `fetch()`, `spawn()`, `timeout()`, `all()`, `race()` using `await` syntax.

---

## Files

**Update:** `crates/atlas-runtime/src/stdlib/mod.rs` — register async stdlib functions as async-aware
**Update:** `crates/atlas-runtime/src/stdlib/future.rs` — expose `all()`, `race()`, `spawn()` as Atlas-callable async fns returning `Value::Future`
**Update:** `crates/atlas-runtime/src/stdlib/async_primitives.rs` — `sleep()`, `timeout()`, `interval()` return `Value::Future`
**Update:** `crates/atlas-runtime/src/stdlib/async_io.rs` — file/network async ops return `Value::Future`
**Tests:** `crates/atlas-runtime/tests/async_runtime.rs` (+20 test cases)

**Total new code:** ~150 lines stdlib, ~80 lines tests
**Total tests:** ~20 test cases

---

## Implementation Notes

**All async stdlib functions must return `Value::Future`** when called from Atlas code. The caller then uses `await` to resolve them.

**Pattern for async stdlib function:**
```rust
fn atlas_sleep(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let ms = // parse arg
    let future = AtlasFuture::new(async move {
        tokio::time::sleep(Duration::from_millis(ms)).await;
        Ok(Value::Null)
    });
    Ok(Value::Future(ValueFuture::new(future)))
}
// Usage in Atlas: await sleep(1000)
```

**Functions to expose:**
| Atlas function | Module | Returns |
|---------------|--------|---------|
| `sleep(ms)` | async_primitives | `Future<void>` |
| `timeout(future, ms)` | async_primitives | `Future<Result<T, string>>` |
| `interval(ms, fn)` | async_primitives | `Future<void>` |
| `spawn(async_fn)` | future | `Future<T>` |
| `all(futures)` | future | `Future<Array<T>>` |
| `race(futures)` | future | `Future<T>` |
| `fetch(url)` | async_io | `Future<string>` |
| `read_file_async(path)` | async_io | `Future<string>` |
| `write_file_async(path, data)` | async_io | `Future<void>` |

**Naming convention:** Sync stdlib functions keep existing names. Async variants are exposed directly (Atlas has no sync file I/O — it's async-first). Sync `read_file` in fs.rs stays separate.

---

## Tests

**sleep + await:** (4 tests)
1. `await sleep(0)` resolves immediately
2. `await sleep(100)` resolves after ~100ms
3. sleep returns void
4. sleep in async fn body

**all + race:** (4 tests)
1. `await all([f1, f2])` resolves both
2. `await all([])` resolves immediately with empty array
3. `await race([slow, fast])` returns fast result
4. race with error in one future

**spawn:** (4 tests)
1. `spawn(async_fn)` returns Future
2. Spawned task runs concurrently
3. `await spawn(fn)` retrieves result
4. Multiple spawns

**async_io:** (4 tests)
1. `await read_file_async(path)` reads file
2. `await write_file_async(path, data)` writes file
3. Error: file not found propagates as Result::Err
4. `await fetch(url)` (mock or localhost)

**timeout:** (4 tests)
1. `await timeout(slow_future, 100)` returns error on timeout
2. `await timeout(fast_future, 1000)` returns value
3. timeout with zero ms
4. Nested timeout

**Minimum test count:** 20 tests

---

## Acceptance Criteria

- ✅ `sleep`, `timeout`, `interval` return `Value::Future`
- ✅ `spawn`, `all`, `race` return `Value::Future`
- ✅ `read_file_async`, `write_file_async`, `fetch` return `Value::Future`
- ✅ All functions work with `await` in Atlas code
- ✅ 20+ stdlib async tests pass
- ✅ Both interpreter and VM execute stdlib async functions correctly
- ✅ `cargo check -p atlas-runtime` clean

---

## References

**Decision Logs:** D-030
**Spec:** docs/language/async.md, docs/stdlib/ (async sections)
**Related phases:** Phase 09 (interpreter), Phase 10 (VM), Phase 12 (parity sweep verifies stdlib)
