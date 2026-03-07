# Phase 05: Value::Future

## Dependencies

**Required:** Phase 04 complete (parser can produce async AST; codebase compiles cleanly)

**Verification:**
```bash
grep "Value::Future\|ValueFuture" crates/atlas-runtime/src/value.rs
cargo check -p atlas-runtime
```

**If missing:** Parser and AST must be stable before touching value.rs — blast radius is too wide to layer on a broken build.

---

## Objective

Add `Value::Future(ValueFuture)` as a first-class runtime value. Cover the full blast radius so no wildcard match arms silently hide the new variant anywhere in the codebase.

---

## Files

**Update:** `crates/atlas-runtime/src/value.rs` (~40 lines)
**Update:** `crates/atlas-runtime/src/bytecode/serialize.rs` (~5 lines)
**Update:** `crates/atlas-runtime/src/bytecode/disasm.rs` (~3 lines)
**Update:** `crates/atlas-runtime/src/api/conversion.rs` (~5 lines)
**Update:** `crates/atlas-runtime/src/stdlib/types.rs` (~3 lines)
**Update:** All interpreter and VM pattern matches flagged by the compiler

**Total new code:** ~80 lines
**Total tests:** ~15 lines (5 test cases)

---

## Dependencies (Components)

- `value.rs` — Value enum (existing)
- `async_runtime/future.rs` — AtlasFuture (existing)
- `bytecode/serialize.rs`, `disasm.rs` — serialization layer (existing)
- `stdlib/types.rs` — type reflection function (existing)

---

## Implementation Notes

**Key patterns to analyze:**
- Read the `CLAUDE.md` blast radius section for `value.rs` before writing a single line — it lists every file that must be updated when a new Value variant is added
- Compare `ClosureRef` structure for the pattern: a wrapper struct holding an `Arc`-based inner type, with `Clone` sharing the handle rather than copying the data
- Examine how `Value::Function` is handled in `type_name()`, `Display`, and `PartialEq` — futures follow the same non-equality model as functions

**Critical requirements:**
- `ValueFuture` is a handle type — cloning shares the same underlying future, not a new one
- `PartialEq` for futures must always return `false` — two future handles are never equal, same as functions
- `type_name()` must return `"Future"`
- `Display` must render as `"<Future>"` — same convention as other opaque values
- Bytecode serialization: futures cannot be serialized; add a sentinel/unreachable arm — futures are runtime-only and should never appear in a bytecode stream
- Zero wildcard `_ =>` arms may hide the new variant — the compiler must see every match exhaustiveness gap

**Error handling:**
- AT codes not relevant here — Value is not the error-emitting layer

**Integration points:**
- Uses: `AtlasFuture` from `async_runtime/future.rs` (existing)
- Creates: `ValueFuture` struct, `Value::Future` variant
- Blast radius checklist: `type_name()`, `Display`, `PartialEq`, `Clone`, `bytecode/serialize.rs`, `bytecode/disasm.rs`, `api/conversion.rs`, `stdlib/types.rs`, interpreter expr match, interpreter mod match, VM mod match, stdlib dispatch match

---

## Tests (TDD Approach)

**Value::Future basics** (5 tests in `tests/async_runtime.rs`)
1. `type_name()` on a Value::Future returns `"Future"`
2. `Display` renders a Value::Future as `"<Future>"`
3. Two distinct `ValueFuture` instances are not equal
4. Cloning a `ValueFuture` shares the same underlying handle (Arc identity check)
5. `type_of(Value::Future)` in the stdlib returns the string `"Future"`

**Minimum test count:** 5 tests

**Parity requirement:** `type_name()` and `Display` results must be identical whether called from interpreter or VM context — both read from the same `value.rs` implementation, so this is automatically satisfied.

---

## Acceptance Criteria

- ✅ `ValueFuture` struct and `Value::Future` variant added
- ✅ `type_name()` returns `"Future"`
- ✅ `Display` renders `"<Future>"`
- ✅ `PartialEq` always false for futures
- ✅ All pattern matches exhaustive — zero wildcard arms hiding the new variant
- ✅ Bytecode serialization handles the Future sentinel
- ✅ 5+ tests pass
- ✅ `cargo check -p atlas-runtime` clean — zero new warnings

---

## References

**Decision Logs:** D-029 (CoW value model), D-030 (multi-threaded async)
**Specifications:** docs/language/async.md (type system section)
**Related phases:** Phase 04 (parser), Phase 06 (bytecode opcodes reference ValueFuture), Phase 09 and 10 (engines produce and consume Value::Future)
