# Phase 10: VM (Bytecode Engine)

## Dependencies

**Required:** Phase 09 complete (interpreter working; multi-thread runtime active; Value: Send confirmed)

**Verification:**
```bash
grep "Opcode::AsyncCall\|Opcode::Await\|Opcode::WrapFuture" crates/atlas-runtime/src/vm/mod.rs
cargo check -p atlas-runtime
```

**If missing:** The interpreter (Phase 09) establishes the multi-thread runtime and the Value: Send contract — the VM depends on both being verified before adding parallel task spawning.

---

## Objective

Implement execution of all four async opcodes in the VM bytecode engine. The VM uses `tokio::spawn` (not `spawn_local`) because it operates on bytecode with no AST references — Value is Send, so true multi-thread spawning is correct here.

---

## Files

**Update:** `crates/atlas-runtime/src/vm/mod.rs` (~80 lines — four new opcode arms in the execute loop)
**Tests:** `crates/atlas-runtime/tests/vm/integration.rs` (~20 test cases)

**Total new code:** ~80 lines VM, ~80 lines tests
**Total tests:** ~20 test cases

---

## Dependencies (Components)

- `vm/mod.rs` — bytecode execute loop (existing)
- `async_runtime/mod.rs` — multi-thread runtime (Phase 09)
- Opcodes: AsyncCall, Await, WrapFuture, SpawnTask (Phase 06)
- Compiler output (Phase 08) — FunctionRef.is_async
- `Value::Future` / `ValueFuture` (Phase 05)

---

## Implementation Notes

**Key patterns to analyze:**
- Find the existing `Opcode::Call` arm in the VM execute loop — `Opcode::AsyncCall` follows the same pop-args-and-fn pattern but dispatches differently
- Find how `Opcode::Return` is handled — `Opcode::Await` needs to integrate at the same level
- Examine how the VM calls `async_runtime::runtime()` — use this reference for `block_on` calls

**Critical requirements:**

`Opcode::AsyncCall`: Pop arg_count arguments and the function reference. Because the VM holds pure bytecode with no AST (all Values are Send), use `tokio::spawn` — not `spawn_local`. Push `Value::Future` wrapping the spawned task handle.

`Opcode::Await`: Pop the top-of-stack value. It must be `Value::Future` — if not, return AT4002 runtime error. Call `async_runtime::runtime().block_on(future.resolve())`. Push the resolved value.

`Opcode::WrapFuture`: Pop any value. Push it wrapped in an immediately-resolved `Value::Future`. No runtime call needed — this is a synchronous wrap.

`Opcode::SpawnTask`: Identical to AsyncCall but the spawned task runs concurrently without blocking the VM thread. The future handle is pushed and the VM continues executing; the caller awaits it explicitly.

**Deadlock prevention**: `block_on` must never be called from within a tokio async context. The VM execute loop runs synchronously — this is safe. Never call `block_on` inside a spawned future.

**Error handling:**
- AT4002 at runtime if `Opcode::Await` pops a non-Future value
- Runtime panics on unresolvable futures must be caught and converted to `RuntimeError`

**Integration points:**
- Uses: Phase 06 (opcodes), Phase 08 (compiler output), Phase 09 (runtime already upgraded)
- The VM and interpreter are independent execution paths — they must produce identical observable output for parity

---

## Tests (TDD Approach)

**VM async execution** (10 tests in `tests/vm/integration.rs`)
1. Compile and run a minimal async fn through the VM — returns Value::Future
2. Compile and run `await async_fn()` through the VM — resolves to inner value
3. Multi-step async program: sequential awaits produce correct final value
4. Error in async fn propagates correctly through VM await
5. SpawnTask creates a Value::Future without blocking
6. WrapFuture produces an immediately-resolved Value::Future
7. VM handles `Future<void>` (async fn with no return) correctly
8. Async fn with parameters — args bound correctly
9. VM async with `?` error propagation
10. Recursive async fn terminates correctly in VM

**Multi-thread verification** (5 tests)
1. Two concurrently spawned tasks both complete via tokio::spawn
2. Value: Send compile-time assertion is present and the project compiles
3. No deadlock: nested await (await inside an awaited task) completes
4. SpawnTask result retrievable after VM continues past spawn point
5. Multi-thread runtime worker count is greater than one (runtime is actually multi-thread)

**Parity against interpreter** (5 tests — same Atlas programs run through both engines)
1. Simple async fn — identical output in interpreter and VM
2. Nested async calls — identical final value
3. Error propagation — identical error message string
4. `type_name()` of awaited result — identical
5. Async with stdlib sleep — identical logical behavior (order of outputs)

**Minimum test count:** 20 tests

---

## Acceptance Criteria

- ✅ `Opcode::AsyncCall` executes correctly — pushes Value::Future
- ✅ `Opcode::Await` resolves future — pushes inner value
- ✅ `Opcode::WrapFuture` wraps plain value in resolved Future
- ✅ `Opcode::SpawnTask` spawns concurrent task via tokio::spawn
- ✅ AT4002 raised when Await pops non-Future value
- ✅ No deadlock risk in block_on usage
- ✅ VM async output matches interpreter output — 100% parity on all 5 parity tests
- ✅ 20+ VM async tests pass
- ✅ `cargo check -p atlas-runtime` clean

---

## References

**Decision Logs:** D-029 (Value CoW confirms Send safety), D-030 (multi-thread mandate)
**Specifications:** docs/language/async.md
**Related phases:** Phase 09 (interpreter baseline), Phase 11 (stdlib wiring uses both engines), Phase 12 (full parity sweep)
