# Phase 06: Bytecode Opcodes

## Dependencies

**Required:** Phase 05 (Value::Future exists)

**Verification:**
```bash
grep "AsyncCall\|Await\|WrapFuture\|SpawnTask" crates/atlas-runtime/src/bytecode/
cargo check -p atlas-runtime
```

---

## Objective

Define and implement the bytecode opcodes for async execution: `AsyncCall`, `Await`, `WrapFuture`, `SpawnTask`. Include serialization, deserialization, and disassembly.

---

## Files

**Update:** `crates/atlas-runtime/src/bytecode/mod.rs` (or opcode definition file) — add 4 opcodes
**Update:** `crates/atlas-runtime/src/bytecode/serialize.rs` — encode/decode new opcodes
**Update:** `crates/atlas-runtime/src/bytecode/disasm.rs` — human-readable disassembly

**Total new code:** ~60 lines

---

## Opcodes

| Opcode | Operands | Stack effect | Semantics |
|--------|----------|-------------|-----------|
| `AsyncCall` | fn_offset: u32, arg_count: u8 | pops args + fn, pushes Future | Call async fn, wrap result in Future<T> |
| `Await` | — | pops Future, pushes T | Block until future resolves, push result |
| `WrapFuture` | — | pops Value, pushes Future<Value> | Wrap a plain value in a resolved Future |
| `SpawnTask` | fn_offset: u32, arg_count: u8 | pops args + fn, pushes Future | Spawn concurrent task, return Future handle |

---

## Implementation Notes

**`AsyncCall` vs `Call`:** The existing `Call` opcode is synchronous — VM calls function and gets value. `AsyncCall` instead:
1. Calls the async fn body
2. Wraps execution in an `AtlasFuture` via `spawn_local`
3. Pushes `Value::Future(handle)` onto stack

**`Await` opcode:**
1. Pops `Value::Future(f)` from stack
2. Calls `block_on(f.resolve())` — blocks current thread until resolved
3. Pushes the resolved `Value::T`
4. Error if top of stack is not Future → AT4002

**`WrapFuture` opcode:** Used to normalize: when an async fn returns a plain value, `WrapFuture` boxes it into a `Future<T>` that immediately resolves. Keeps the return type contract: async fns always push a Future onto the stack.

**`SpawnTask` opcode:** For `spawn()` stdlib call. Spawns on the multi-threaded runtime (D-030) via `tokio::spawn`, returns Future handle. Requires `Value: Send` (verified in Phase 10).

**Serialization format:** Each opcode gets a unique u8/u16 discriminant. Maintain a version-stable encoding.

---

## Tests

Opcode serialization round-trip tests (in bytecode tests or integration):
1. `AsyncCall` encodes/decodes correctly
2. `Await` encodes/decodes correctly
3. `WrapFuture` encodes/decodes correctly
4. `SpawnTask` encodes/decodes correctly
5. Disassembler renders all 4 opcodes as human-readable strings

**Minimum test count:** 5 tests

---

## Acceptance Criteria

- ✅ 4 async opcodes defined: AsyncCall, Await, WrapFuture, SpawnTask
- ✅ All serialize/deserialize correctly
- ✅ Disassembler covers all 4
- ✅ 5+ tests pass
- ✅ `cargo check -p atlas-runtime` clean

---

## References

**Decision Logs:** D-030 (multi-thread, Value: Send needed for SpawnTask)
**Spec:** docs/language/async.md
**Related phases:** Phase 08 (compiler emits these), Phase 09 (interpreter handles semantics), Phase 10 (VM executes these)
