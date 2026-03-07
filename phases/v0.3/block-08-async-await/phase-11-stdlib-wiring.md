# Phase 11: Stdlib Wiring

## Dependencies

**Required:** Phase 10 complete (both interpreter and VM execute async; multi-thread runtime active)

**Verification:**
```bash
grep "Value::Future\|spawn\|sleep\|timeout\|all\|race" crates/atlas-runtime/src/stdlib/future.rs
grep "Value::Future" crates/atlas-runtime/src/stdlib/async_primitives.rs
cargo check -p atlas-runtime
```

**If missing:** Both engines must be working before wiring stdlib — the tests for this phase run Atlas programs through both interpreter and VM.

---

## Objective

Connect the existing async stdlib modules (`async_primitives`, `future`, `async_io`) to the language-level async system. Every async stdlib function must return `Value::Future` so Atlas programs can use `await` on them.

---

## Files

**Update:** `crates/atlas-runtime/src/stdlib/future.rs` (~50 lines — return Value::Future from all, expose all, race, spawn)
**Update:** `crates/atlas-runtime/src/stdlib/async_primitives.rs` (~40 lines — return Value::Future from sleep, timeout, interval)
**Update:** `crates/atlas-runtime/src/stdlib/async_io.rs` (~40 lines — return Value::Future from read_file_async, write_file_async, fetch)
**Update:** `crates/atlas-runtime/src/stdlib/mod.rs` (~20 lines — register new Atlas-callable names)
**Tests:** `crates/atlas-runtime/tests/async_runtime.rs` (~20 test cases)

**Total new code:** ~150 lines stdlib, ~80 lines tests
**Total tests:** ~20 test cases

---

## Dependencies (Components)

- `stdlib/future.rs` — AtlasFuture orchestration (existing)
- `stdlib/async_primitives.rs` — sleep, timeout, interval (existing)
- `stdlib/async_io.rs` — file and network async ops (existing)
- `stdlib/mod.rs` — function registration (existing)
- `Value::Future` / `ValueFuture` (Phase 05)

---

## Implementation Notes

**Key patterns to analyze:**
- Examine how the existing async stdlib functions are currently called (they may return `Value` directly today) — understand what change is needed to make them return `Value::Future` instead
- Check `stdlib/mod.rs` registration to understand how function names are exported to Atlas code
- Review `AtlasFuture::new` usage in `async_runtime/` to understand the correct construction pattern

**Critical requirements:**
- Every async stdlib function callable from Atlas must return `Value::Future(ValueFuture::new(AtlasFuture::new(...)))` — the caller uses `await` to resolve it
- Naming convention: async variants keep their existing names; Atlas is async-first for I/O operations
- Functions to expose as `Value::Future`-returning: `sleep`, `timeout`, `interval`, `spawn`, `all`, `race`, `read_file_async`, `write_file_async`, `fetch`
- `all` takes an array of `Value::Future` values and returns `Value::Future` wrapping an array of resolved values
- `race` takes an array of `Value::Future` values and returns `Value::Future` wrapping the first resolved value
- `spawn` takes a callable (async fn or closure) and a `Value::Future` handle to it

**Error handling:**
- `all` and `race` must handle a Future that resolves to a `RuntimeError` — propagate the first error
- `read_file_async` and `write_file_async` propagate file-not-found and permission errors through the Future

**Integration points:**
- Uses: async_runtime infrastructure (Phase 09/10), Value::Future (Phase 05)
- Both engines execute stdlib — interpreter and VM both call these same Rust functions

---

## Tests (TDD Approach)

**sleep and timeout** (4 tests)
1. `await sleep(0)` resolves immediately to void
2. `await sleep(100)` resolves (timing not asserted exactly, just completes)
3. `await timeout(fast_future, 1000)` returns the resolved value
4. `await timeout(slow_future, 1)` resolves to a timeout error

**all and race** (4 tests)
1. `await all([f1, f2])` resolves both and returns an array of results
2. `await all([])` resolves immediately with an empty array
3. `await race([slow, fast])` returns the fast result
4. `await race([f_err, f_ok])` where error arrives first — error propagates

**spawn** (4 tests)
1. `spawn(async_fn)` returns a Value::Future without blocking
2. `await spawn(async_fn)` retrieves the result
3. Multiple spawns complete independently
4. Spawned task that errors — error retrievable via await

**async_io** (4 tests)
1. `await read_file_async(path)` reads a file and returns its contents
2. `await write_file_async(path, data)` writes a file
3. `await read_file_async(nonexistent)` returns an error result
4. `await fetch(url)` — tested with a local server or mock; returns a string

**Cross-engine** (4 tests — same program through interpreter and VM)
1. `await sleep(0)` produces identical output in both engines
2. `await all([f1, f2])` — identical result array
3. `await race([f1, f2])` — identical winner
4. `await read_file_async` — identical file contents

**Minimum test count:** 20 tests

---

## Acceptance Criteria

- ✅ `sleep`, `timeout`, `interval` return `Value::Future`
- ✅ `spawn`, `all`, `race` return `Value::Future`
- ✅ `read_file_async`, `write_file_async`, `fetch` return `Value::Future`
- ✅ All functions work with `await` in Atlas programs
- ✅ Both interpreter and VM execute stdlib async functions correctly
- ✅ 20+ stdlib async tests pass
- ✅ `cargo check -p atlas-runtime` clean

---

## References

**Decision Logs:** D-030
**Specifications:** docs/language/async.md, docs/stdlib/ async sections
**Related phases:** Phase 10 (both engines), Phase 12 (parity sweep validates stdlib async)
