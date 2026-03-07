# Phase 09: Interpreter (Tree-Walk Engine)

## Dependencies

**Required:** Phase 08 complete (compiler stable; all AST, typechecker, Value::Future, and opcodes in place)

**Verification:**
```bash
grep "Expr::Await\|is_async\|Value::Future" crates/atlas-runtime/src/interpreter/
cargo check -p atlas-runtime
```

**If missing:** All prior phases must be complete — the interpreter handles Expr::Await and is_async from the AST, and produces Value::Future from value.rs.

---

## Objective

Implement async/await execution in the tree-walking interpreter. Async functions wrap their body in an AtlasFuture. Await expressions resolve futures via the runtime. The multi-threaded tokio runtime (D-030) is activated here.

---

## Files

**Update:** `crates/atlas-runtime/src/async_runtime/mod.rs` (~5 lines — switch to new_multi_thread, add Value: Send assertion)
**Update:** `crates/atlas-runtime/src/interpreter/expr.rs` (~40 lines)
**Update:** `crates/atlas-runtime/src/interpreter/mod.rs` (~30 lines)
**Tests:** `crates/atlas-runtime/tests/async_runtime.rs` (~30 test cases)

**Total new code:** ~100 lines interpreter, ~120 lines tests
**Total tests:** ~30 test cases

---

## Dependencies (Components)

- `interpreter/expr.rs` — Expr evaluation (existing)
- `interpreter/mod.rs` — function call execution (existing)
- `async_runtime/mod.rs` — tokio runtime init (existing, needs upgrade)
- `AtlasFuture` from `async_runtime/future.rs` (existing)
- `Value::Future` / `ValueFuture` (Phase 05)

---

## Implementation Notes

**Key patterns to analyze:**
- Find how the interpreter currently evaluates `Expr::Call` — the async fn path branches off from here
- Find how function body execution works (`eval_body` or equivalent) — async fn wraps this in a future
- Examine `async_runtime/mod.rs` to understand the current `new_current_thread` setup and what must change for D-030

**Critical requirements:**
- Multi-thread runtime upgrade (D-030): change `new_current_thread()` to `new_multi_thread()` in `async_runtime/mod.rs`
- Add a compile-time `Value: Send` assertion in `async_runtime/mod.rs` — this documents and enforces the threading contract
- The AST contains `RefCell` (in type annotation caches on certain nodes) making the full AST `!Send` — the interpreter must keep AST evaluation on a single thread. Use `spawn_local` with a `LocalSet` for interpreter async execution; the multi-thread runtime still powers I/O concurrency
- `Expr::Await` evaluation: evaluate the inner expression, match on `Value::Future`, call `block_on` to resolve it, push the result
- Async fn call: detect `is_async` on the function, wrap body evaluation in `AtlasFuture::new`, return `Value::Future`
- Top-level await must work: the top-level scope treats await as a blocking call via `block_on`
- Every future must have a guaranteed resolution path — verify no async fn body can loop forever without a termination condition (infinite loop risk)

**Error handling:**
- AT4002 at runtime if `Expr::Await` operand resolves to a non-Future value

**Integration points:**
- Uses: all Phase 01–08 artifacts
- Updates: `async_runtime/mod.rs` (runtime upgrade affects both engines)
- Consumed by: Phase 12 (parity sweep compares interpreter output against VM output)

---

## Tests (TDD Approach)

**Basic async/await** (10 tests in `tests/async_runtime.rs`)
1. `async fn` returns `Value::Future` when called (not the resolved value)
2. `await` on a resolved Future returns the inner value
3. Async fn returning a number — await yields the number
4. Async fn returning a string — await yields the string
5. Nested async: outer async fn calls await inner async fn
6. Multiple sequential awaits in one function body
7. Async fn with parameters
8. Async fn with if/else branch — both paths resolve correctly
9. Top-level await works at program entry
10. Async fn returning void

**Concurrency** (8 tests)
1. Two concurrent futures both complete
2. Spawning a task and awaiting its result
3. `future_all` with two futures — both results collected
4. `future_race` — first-resolving future wins
5. Async sleep via stdlib resolves after the expected delay
6. Error in an async fn propagates correctly through await
7. Multiple spawned tasks complete independently
8. Async fn with a closure captures values correctly

**Error cases** (7 tests)
1. AT4002 raised at runtime: `await` on a non-Future value
2. Async fn that returns an error value — await propagates it
3. Nested error propagation with `?` inside async fn
4. Future that resolves to `Result::Err` — handled via match
5. Timeout primitive with a slow future
6. Awaiting a future that was already resolved
7. Async fn recursive call terminates correctly

**Parity baseline** (5 tests — same programs, recorded for comparison in Phase 12)
1. Simple async fn: output string matches expected
2. Nested async: final computed value matches expected
3. Concurrent tasks: result array matches expected
4. Error propagation: error message matches expected
5. Return type: `type_name()` of awaited result matches expected

**Minimum test count:** 30 tests

---

## Acceptance Criteria

- ✅ `new_multi_thread()` runtime active in `async_runtime/mod.rs` (D-030)
- ✅ `Value: Send` compile-time assertion present
- ✅ `Expr::Await` evaluates correctly in interpreter
- ✅ Async fn calls return `Value::Future`
- ✅ Top-level await works
- ✅ 30+ interpreter async tests pass
- ✅ No infinite loop risk — all futures have guaranteed resolution paths
- ✅ `cargo check -p atlas-runtime` clean

---

## References

**Decision Logs:** D-029 (CoW — confirms futures hold owned Values), D-030 (multi-thread mandate)
**Specifications:** docs/language/async.md (semantics section)
**Related phases:** Phase 08 (compiler), Phase 10 (VM must match this behavior exactly for parity)
