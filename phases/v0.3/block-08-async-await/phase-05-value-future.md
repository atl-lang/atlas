# Phase 05: Value::Future

## Dependencies

**Required:** Phase 03 (AST has TypeRef::Future)

**Verification:**
```bash
grep "Value::Future\|ValueFuture" crates/atlas-runtime/src/value.rs
cargo check -p atlas-runtime
```

---

## Objective

Add `Value::Future(ValueFuture)` as a first-class runtime value, with full blast-radius coverage: type_name, Display, PartialEq, bytecode serialization, and all pattern-match exhaustiveness.

---

## Files

**Update:** `crates/atlas-runtime/src/value.rs`
  - New struct `ValueFuture` wrapping `Arc<Mutex<AtlasFuture>>`
  - New `Value::Future(ValueFuture)` variant
  - `type_name()`: returns `"Future"`
  - `Display`: `"<Future>"`
  - `PartialEq`: futures are never equal (like functions)
**Update:** `crates/atlas-runtime/src/bytecode/serialize.rs` — serialize/deserialize Future tag
**Update:** `crates/atlas-runtime/src/bytecode/disasm.rs` — disassemble Future value
**Update:** `crates/atlas-runtime/src/api/conversion.rs` — conversion impl
**Update:** `crates/atlas-runtime/src/stdlib/types.rs` — type reflection
**Update:** All interpreter/VM pattern matches (compiler will flag exhaustiveness)

**Total new code:** ~80 lines
**Total tests:** ~10 lines (type_name, display, PartialEq)

---

## Implementation Notes

**`ValueFuture` struct:**
```rust
#[derive(Debug, Clone)]
pub struct ValueFuture(Arc<Mutex<AtlasFuture>>);
```
Uses `Mutex` for interior mutability on the future state (`Pending`/`Ready`). This is intentional shared mutation — `ValueFuture` is a handle, not a value (like `ClosureRef`). Cloning shares the same future handle.

**PartialEq:** Always `false` — two future handles are never equal even if same task. Same as functions.

**Serialization:** Futures cannot be serialized to bytecode (they're runtime-only). Serialize as a sentinel/error — deserializing a Future is a panic (should never occur).

**Blast radius checklist** (must be exhaustive):
- `value.rs`: type_name, Display, PartialEq, Clone
- `bytecode/serialize.rs`: match arm
- `bytecode/disasm.rs`: match arm  
- `api/conversion.rs`: match arm
- `stdlib/types.rs`: `type_of()` function
- `interpreter/expr.rs`: any Value pattern matches
- `interpreter/mod.rs`: any Value pattern matches
- `vm/mod.rs`: any Value pattern matches
- `stdlib/mod.rs`: any dispatch pattern matches

**Read `crates/atlas-runtime/src/CLAUDE.md` blast radius section before starting.**

---

## Tests

**Value::Future basics:** (5 tests in tests/async_runtime.rs)
1. `type_name()` returns `"Future"`
2. `Display` renders `"<Future>"`
3. Two ValueFuture instances are not equal
4. Clone shares the same future handle (Arc identity)
5. `Value::Future` round-trips through type dispatch (type_of returns "Future")

**Minimum test count:** 5 tests

---

## Acceptance Criteria

- ✅ `Value::Future(ValueFuture)` added
- ✅ type_name, Display, PartialEq all correct
- ✅ All pattern matches exhaustive (no `_ =>` wildcards hiding this)
- ✅ Bytecode serialization handles Future sentinel
- ✅ 5+ tests pass
- ✅ `cargo check -p atlas-runtime` clean — zero new warnings
- ✅ No wildcard match arms hiding the new variant

---

## References

**Decision Logs:** D-029 (CoW), D-030 (multi-threaded async)
**Spec:** docs/language/async.md (type system section)
**Related phases:** Phase 06 (opcodes reference ValueFuture), Phase 09/10 (engines produce/consume it)
