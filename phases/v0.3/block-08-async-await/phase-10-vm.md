# Phase 10: VM (Bytecode Engine) + Multi-Thread Upgrade

## Dependencies

**Required:** Phase 06 (opcodes), Phase 08 (compiler), Phase 09 (interpreter — for parity baseline)

**Verification:**
```bash
grep "Opcode::AsyncCall\|Opcode::Await\|Opcode::WrapFuture" crates/atlas-runtime/src/vm/mod.rs
grep "new_multi_thread\|spawn(" crates/atlas-runtime/src/async_runtime/mod.rs
cargo check -p atlas-runtime
```

---

## Objective

Implement async opcode execution in the VM bytecode engine, and upgrade the tokio runtime to `new_multi_thread`. This is the most critical phase: VM must be 100% parity with the interpreter, and must use true multi-threaded execution (D-030).

---

## Files

**Update:** `crates/atlas-runtime/src/vm/mod.rs` — execute `AsyncCall`, `Await`, `WrapFuture`, `SpawnTask`
**Update:** `crates/atlas-runtime/src/async_runtime/mod.rs` — `new_multi_thread()` runtime, `tokio::spawn` for Send futures
**Tests:** `crates/atlas-runtime/tests/async_runtime.rs` — parity tests (shared with Phase 09)
         `crates/atlas-runtime/tests/vm/integration.rs` — VM-specific async integration tests

**Total new code:** ~120 lines VM, ~60 lines async_runtime, ~80 lines tests
**Total tests:** ~20 new test cases (+ parity tests from Phase 09)

---

## Implementation Notes

### Multi-Thread Runtime Upgrade (critical)

```rust
// BEFORE (Phase 09 uses this for interpreter):
tokio::runtime::Builder::new_current_thread()

// AFTER (D-030 — for VM and all global runtime):
tokio::runtime::Builder::new_multi_thread()
    .worker_threads(num_cpus::get())
    .enable_all()
    .build()
```

**Consequence:** All futures spawned via `tokio::spawn` must be `Send`. The VM works from bytecode — `Value` is `Send` (Arc-based, no Rc/RefCell in value.rs). VM async uses `tokio::spawn`.

**Interpreter exception:** Interpreter uses `spawn_local` in a `LocalSet` because AST holds `RefCell`. This is correct — interpreter is single-threaded internally but benefits from the multi-thread runtime for I/O.

**`Value: Send` assertion (add as compile-time check):**
```rust
fn _assert_value_send() {
    fn assert_send<T: Send>() {}
    assert_send::<Value>();
}
```

### VM Opcode Execution

**`AsyncCall`:**
```rust
Opcode::AsyncCall { fn_offset, arg_count } => {
    let args = self.pop_n(arg_count);
    let func = self.bytecode_fn_at(fn_offset);
    // Clone everything needed — Value: Send
    let handle = tokio::spawn(async move {
        execute_async_body(func, args).await
    });
    self.push(Value::Future(ValueFuture::from_join_handle(handle)));
}
```

**`Await`:**
```rust
Opcode::Await => {
    let val = self.pop();
    match val {
        Value::Future(f) => {
            let result = async_runtime::runtime().block_on(f.resolve())?;
            self.push(result);
        }
        _ => return Err(RuntimeError::new(ErrorCode::AT4002, "await on non-Future")),
    }
}
```

**`WrapFuture`:**
```rust
Opcode::WrapFuture => {
    let val = self.pop();
    let future = AtlasFuture::resolved(val);
    self.push(Value::Future(ValueFuture::new(future)));
}
```

**`SpawnTask`:** Same as AsyncCall but using `tokio::spawn` without blocking — caller gets a Future handle for explicit await later.

### Infinite Loop Risk (CRITICAL)
VM execute loop must handle async futures correctly:
- `block_on` MUST NOT be called from within the tokio runtime thread (deadlock)
- Use `Runtime::block_on` from a non-async context (main thread or dedicated thread)
- Never call `block_on` inside an `async` block

---

## Tests

**VM async execution:** (10 tests)
1. `async fn` compiled + executed in VM returns Future
2. `await` in VM resolves future
3. Multi-step async: compile + run a complete async program
4. Error in VM async fn propagates correctly
5. SpawnTask creates concurrent future
6. WrapFuture creates immediately-resolved future
7. VM async with stdlib async_io
8. VM handles Future<void> correctly
9. Recursive async fn in VM
10. VM async with error propagation via ?

**Multi-thread verification:** (5 tests)
1. Concurrent spawns complete independently
2. Two async tasks run in parallel (timing check)
3. `Value: Send` compile-time assertion present
4. No deadlock on nested await
5. Runtime uses multi-thread worker threads

**Parity with interpreter:** (5 tests — run same programs through both engines)
1. Simple async fn — identical output
2. Nested async — identical output
3. Error propagation — identical error message
4. Future type_name — identical
5. async + stdlib (sleep) — identical logical behavior

**Minimum test count:** 20 tests

---

## Acceptance Criteria

- ✅ `new_multi_thread()` runtime active (D-030)
- ✅ `Value: Send` compile-time assertion added
- ✅ `Opcode::AsyncCall`, `Await`, `WrapFuture`, `SpawnTask` all execute correctly
- ✅ No deadlock risk in block_on usage
- ✅ VM async output matches interpreter output (100% parity)
- ✅ 20+ VM async tests pass
- ✅ `cargo check -p atlas-runtime` clean

---

## References

**Decision Logs:** D-030 (multi-thread mandate), D-029 (Value CoW — confirms Send safety)
**Spec:** docs/language/async.md
**Related phases:** Phase 09 (interpreter baseline for parity), Phase 11 (stdlib wiring), Phase 12 (parity sweep)
