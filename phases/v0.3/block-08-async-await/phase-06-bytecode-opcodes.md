# Phase 06: Bytecode Opcodes

## Dependencies

**Required:** Phase 05 complete (Value::Future exists and compiles cleanly)

**Verification:**
```bash
grep "AsyncCall\|WrapFuture\|SpawnTask" crates/atlas-runtime/src/bytecode/
cargo check -p atlas-runtime
```

**If missing:** Value::Future must exist before opcodes that push it onto the stack can be defined.

---

## Objective

Define the four async bytecode opcodes (`AsyncCall`, `Await`, `WrapFuture`, `SpawnTask`), implement their serialization and deserialization, and add disassembler support.

---

## Files

**Update:** `crates/atlas-runtime/src/bytecode/mod.rs` (or opcode definitions file) (~20 lines)
**Update:** `crates/atlas-runtime/src/bytecode/serialize.rs` (~15 lines)
**Update:** `crates/atlas-runtime/src/bytecode/disasm.rs` (~10 lines)

**Total new code:** ~60 lines
**Total tests:** ~15 lines (5 test cases)

---

## Dependencies (Components)

- `bytecode/mod.rs` — opcode enum (existing)
- `bytecode/serialize.rs` — encode/decode logic (existing)
- `bytecode/disasm.rs` — human-readable disassembly (existing)
- `Value::Future` / `ValueFuture` (Phase 05)

---

## Implementation Notes

**Key patterns to analyze:**
- Examine how an existing opcode with operands (e.g., `Call` with fn_offset and arg_count) is defined, serialized, and disassembled — follow that exact pattern for the four new opcodes
- Check the opcode discriminant assignment scheme to pick non-colliding values for the four new opcodes

**Critical requirements — four opcodes and their semantics:**

`AsyncCall` (fn_offset, arg_count): Calls an async function. Pops arg_count arguments and the function reference off the stack, wraps the invocation in a Future, and pushes a `Value::Future` result. The future is not yet resolved — execution is deferred.

`Await` (no operands): Pops a `Value::Future` off the stack and blocks until it resolves, then pushes the resolved inner value. If the top of stack is not a Future, this is a runtime error (AT4002).

`WrapFuture` (no operands): Pops any value off the stack and pushes it back wrapped in an immediately-resolved `Value::Future`. Used when an async fn returns a plain value — the return contract requires a Future on the stack.

`SpawnTask` (fn_offset, arg_count): Like AsyncCall but spawns the task on the multi-thread runtime for true concurrent execution. Pushes a `Value::Future` handle that the caller can await later.

**Error handling:**
- AT4002 is a runtime error path in the VM for `Await` when the top of stack is not a Future — the opcode definition itself does not emit diagnostics, only the VM execution does (Phase 10)

**Integration points:**
- Uses: Value::Future (Phase 05)
- Creates: four new opcode variants
- Consumed by: compiler (Phase 08), interpreter (Phase 09), VM (Phase 10)

---

## Tests (TDD Approach)

**Opcode serialization round-trips** (5 tests)
1. `AsyncCall { fn_offset: 0, arg_count: 2 }` encodes and decodes to the same value
2. `Await` encodes and decodes correctly (no operands)
3. `WrapFuture` encodes and decodes correctly (no operands)
4. `SpawnTask { fn_offset: 5, arg_count: 0 }` encodes and decodes to the same value
5. Disassembler renders all four opcodes as non-empty human-readable strings

**Minimum test count:** 5 tests

**Parity requirement:** N/A — bytecode is shared infrastructure, not engine-specific.

---

## Acceptance Criteria

- ✅ Four async opcodes defined: AsyncCall, Await, WrapFuture, SpawnTask
- ✅ All four serialize and deserialize correctly (round-trip)
- ✅ Disassembler covers all four with human-readable output
- ✅ 5+ serialization tests pass
- ✅ `cargo check -p atlas-runtime` clean

---

## References

**Decision Logs:** D-030 (SpawnTask requires multi-thread runtime and Value: Send)
**Specifications:** docs/language/async.md
**Related phases:** Phase 05 (Value::Future), Phase 08 (compiler emits these), Phase 09 (interpreter handles semantics without bytecode), Phase 10 (VM executes these opcodes)
