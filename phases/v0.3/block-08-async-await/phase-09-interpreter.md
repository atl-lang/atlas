# Phase 09: Interpreter (Tree-Walk Engine)

## Dependencies

**Required:** Phase 05 (Value::Future), Phase 07 (typechecker), Phase 04 (parser)

**Verification:**
```bash
grep "Expr::Await\|is_async\|Value::Future" crates/atlas-runtime/src/interpreter/
cargo check -p atlas-runtime
```

---

## Objective

Implement async/await execution in the tree-walking interpreter. The interpreter handles `async fn` by wrapping execution in `AtlasFuture`, and `await` by blocking on the future using the multi-threaded tokio runtime.

---

## Files

**Update:** `crates/atlas-runtime/src/interpreter/expr.rs`
  - `Expr::Await`: evaluate inner expr, expect Value::Future, call runtime block_on
  - `Expr::Call` on async fn: wrap execution in AtlasFuture via spawn_local
**Update:** `crates/atlas-runtime/src/interpreter/mod.rs`
  - Track async context (for top-level await handling)
  - `eval_function`: detect `is_async`, wrap body evaluation in future
**Tests:** `crates/atlas-runtime/tests/async_runtime.rs` (+30 test cases)

**Total new code:** ~100 lines interpreter, ~120 lines tests
**Total tests:** ~30 test cases

---

## Implementation Notes

**Multi-threaded runtime (D-030):** Switch `async_runtime/mod.rs` from `new_current_thread()` to `new_multi_thread()`. Audit all `spawn_local` calls — `spawn_local` is `!Send`, must switch to `tokio::spawn` where `Value: Send`. Verify `Value: Send` holds (no Rc, no RefCell in value.rs — confirmed in Phase 05 prerequisites).

**`!Send` exception:** The AST (used by interpreter) contains `Cell<Option<TypeTag>>` and `RefCell` — the AST is NOT Send. The interpreter keeps AST on a single thread. Async fn bodies evaluated by the interpreter must use `spawn_local` (thread-local), not `tokio::spawn`. The multi-threaded runtime still powers I/O — but interpreter AST evaluation stays thread-local.

**This is the critical parity point:** The VM (Phase 10) will execute bytecode (no AST, all Send-safe). The interpreter uses AST (not Send). Solution: interpreter async uses `LocalSet + spawn_local`; VM async uses `tokio::spawn`. Observable behavior identical.

**`Expr::Await` in interpreter:**
```rust
Expr::Await { expr, .. } => {
    let val = self.eval_expr(expr)?;
    match val {
        Value::Future(f) => async_runtime::block_on(f.resolve()),
        _ => Err(RuntimeError::new(ErrorCode::AT4002, "await on non-Future")),
    }
}
```

**Async fn call:**
```rust
if func.is_async {
    let future = AtlasFuture::new(async move { self.eval_body(body, args) });
    Ok(Value::Future(ValueFuture::new(future)))
} else {
    self.eval_body(body, args)
}
```

---

## Tests (tests/async_runtime.rs)

**Basic async/await:** (10 tests)
1. `async fn` returns `Value::Future`
2. `await` on resolved future returns value
3. `async fn` returning number — await yields number
4. `async fn` returning string — await yields string
5. Nested async: `await outer()` where outer calls `await inner()`
6. Multiple awaits in sequence
7. async fn with parameters
8. async fn with if/else branch
9. Top-level await works
10. async fn returning void

**Concurrency:** (8 tests)
1. Two concurrent tasks — both complete
2. Task order: later-resolving task doesn't block earlier
3. `future_all([f1, f2])` — all complete
4. `future_race([f1, f2])` — first wins
5. Async sleep (stdlib) — resolves after delay
6. spawn + await pattern
7. Error in async fn — propagates through await
8. Panic in async task — contained, returns error

**Error cases:** (7 tests)
1. AT4001: await outside async context (interpreter enforces)
2. AT4002: await on non-Future value
3. Async fn that errors — await propagates the error
4. Timeout on future (stdlib async_primitives)
5. Cancelled future
6. Nested error propagation with ?
7. async fn recursive call

**Parity guards:** (5 tests — shared with VM Phase 10)
1. Same Atlas program produces same output in interpreter and VM
2. Async timing: both engines complete in same logical order
3. Future value type_name identical
4. Error messages identical
5. Return value types identical

**Minimum test count:** 30 tests

---

## Acceptance Criteria

- ✅ `new_multi_thread()` runtime in async_runtime/mod.rs
- ✅ `Expr::Await` evaluates correctly in interpreter
- ✅ async fn calls return `Value::Future`
- ✅ `block_on` bridges async → sync correctly
- ✅ 30+ interpreter async tests pass
- ✅ `cargo check -p atlas-runtime` clean
- ✅ No infinite loop risk (all futures have guaranteed resolution paths)

---

## References

**Decision Logs:** D-030 (multi-thread), D-029 (CoW — futures hold Values, not shared refs)
**Spec:** docs/language/async.md (semantics section)
**Related phases:** Phase 10 (VM must match this behavior exactly — parity)
