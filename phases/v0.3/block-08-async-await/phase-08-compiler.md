# Phase 08: Compiler (AST → Bytecode)

## Dependencies

**Required:** Phase 07 complete (typechecker annotates call sites; all prior phases stable)

**Verification:**
```bash
grep "is_async\|AsyncCall\|WrapFuture\|Await" crates/atlas-runtime/src/compiler/
cargo check -p atlas-runtime
```

**If missing:** Typechecker must be complete — the compiler reads type annotations produced by the typechecker to decide whether to emit AsyncCall vs Call.

---

## Objective

Emit correct async bytecode from async AST nodes. Async function metadata is marked, call sites dispatch to AsyncCall vs Call based on typechecker annotations, await expressions emit the Await opcode, and WrapFuture normalizes returns from async fn bodies.

---

## Files

**Update:** `crates/atlas-runtime/src/compiler/stmt.rs` (~40 lines)
**Update:** `crates/atlas-runtime/src/compiler/expr.rs` (~30 lines)
**Update:** `crates/atlas-runtime/src/value.rs` — add `is_async: bool` to `FunctionRef` (~3 lines + construction sites)

**Total new code:** ~80 lines
**Total tests:** ~15 lines (5 test cases)

---

## Dependencies (Components)

- `compiler/stmt.rs` — function declaration compilation (existing)
- `compiler/expr.rs` — call expression and return compilation (existing)
- `value.rs` — FunctionRef (existing, needs `is_async` field)
- Opcodes: AsyncCall, Await, WrapFuture, SpawnTask (Phase 06)
- Type annotations from typechecker (Phase 07)

---

## Implementation Notes

**Key patterns to analyze:**
- Find how `compile_function_decl` currently populates `FunctionRef` — `is_async` must be set from `FunctionDecl.is_async`
- Find where `Opcode::Return` is emitted inside function bodies — `WrapFuture` must be inserted before every Return in an async fn
- Find how `Expr::Call` is compiled — the async vs sync dispatch decision (AsyncCall vs Call) is made here using typechecker annotation

**Critical requirements:**
- `FunctionRef.is_async: bool` must be added (small blast radius — grep `FunctionRef {` for all construction sites)
- When compiling an `async fn` body: every path that emits `Opcode::Return` must first emit `Opcode::WrapFuture` — this ensures the return value is always a `Future<T>` on the stack
- When compiling `Expr::Await`: emit code for the inner expression, then emit `Opcode::Await`
- When compiling a call expression: if the typechecker has annotated the callee as an async fn, emit `Opcode::AsyncCall`; otherwise emit the existing `Opcode::Call`
- `spawn(fn)` stdlib calls compile to `Opcode::SpawnTask`

**Error handling:**
- No new AT codes from the compiler — type errors were caught in Phase 07

**Integration points:**
- Uses: Phase 06 (opcodes), Phase 07 (typechecker call-site annotations)
- Updates: FunctionRef in value.rs
- Consumed by: VM (Phase 10) which executes the emitted bytecode

---

## Tests (TDD Approach)

**Bytecode shape verification** (5 tests — compile and inspect emitted opcodes)
1. `async fn foo() -> number` produces a FunctionRef with `is_async: true`
2. `await foo()` in a function body emits an Await opcode after the call
3. A sync `fn bar()` does NOT emit AsyncCall — regression guard
4. An async fn with a plain return value has WrapFuture emitted before Return
5. Nested await compiles to two sequential Await opcodes

**Minimum test count:** 5 tests

**Parity requirement:** The compiler produces bytecode for the VM (Phase 10). The interpreter (Phase 09) does not use this bytecode — it walks the AST directly. Parity is verified in Phase 12.

---

## Acceptance Criteria

- ✅ `FunctionRef.is_async: bool` added, all construction sites updated
- ✅ Async fn bodies compile with `is_async: true` metadata
- ✅ `Expr::Await` emits `Opcode::Await`
- ✅ Async fn calls emit `Opcode::AsyncCall`
- ✅ `WrapFuture` emitted before every Return in async fn body
- ✅ 5+ bytecode shape tests pass
- ✅ `cargo check -p atlas-runtime` clean

---

## References

**Decision Logs:** D-030
**Specifications:** docs/language/async.md
**Related phases:** Phase 07 (typechecker), Phase 09 (interpreter — parallel execution path, does not use this bytecode), Phase 10 (VM executes the bytecode produced here)
