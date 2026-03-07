# Phase 08: Compiler (AST → Bytecode)

## Dependencies

**Required:** Phase 06 (opcodes), Phase 07 (typechecker), Phase 04 (parser)

**Verification:**
```bash
grep "AsyncCall\|Await\|WrapFuture\|is_async" crates/atlas-runtime/src/compiler/
cargo check -p atlas-runtime
```

---

## Objective

Emit correct async bytecode from the AST. Async function bodies are compiled into resumable units; `await` emits the Await opcode; async calls emit AsyncCall or WrapFuture as appropriate.

---

## Files

**Update:** `crates/atlas-runtime/src/compiler/stmt.rs`
  - `compile_function_decl`: detect `is_async`, mark function as async in bytecode, emit `WrapFuture` at return points
**Update:** `crates/atlas-runtime/src/compiler/expr.rs`
  - `Expr::Await`: emit `Await` opcode after compiling inner expr
  - `Expr::Call` on async fn: emit `AsyncCall` instead of `Call`
  - Async fn call detection: typechecker must have annotated the call site

**Total new code:** ~80 lines

---

## Implementation Notes

**Async fn compilation strategy:**
The async fn body compiles to normal bytecode. The key difference: the entire body execution is wrapped by the VM in a Future at call time (via `AsyncCall` opcode). The compiler's job is to:
1. Mark the function's metadata as `is_async: true` (in `FunctionRef`)
2. Emit `WrapFuture` before any `Return` opcode inside an async fn (so the returned value becomes `Future<T>`)

**`FunctionRef.is_async`:** Must add this field to `FunctionRef` in `value.rs` — the VM needs to know at call time whether to use `AsyncCall` semantics.

**`Expr::Await` compilation:**
```
compile inner expr      → pushes Future<T> onto stack
emit Opcode::Await      → pops Future<T>, pushes T
```

**Call site detection:** The compiler asks typechecker: "is this call target an async fn?" If yes, emit `AsyncCall`. If no, emit `Call`. This requires the typechecker to annotate call sites (phase 07 must expose this).

**`WrapFuture` for sync returns inside async:** If an async fn returns a plain value (e.g., early `return 42`), the compiler emits `WrapFuture` to box it into a resolved `Future<number>` — preserving the return type contract.

---

## Tests

Compiler tests are integration tests (compile + run). These are covered by Phase 09/10. Unit tests here focus on bytecode output:

**Bytecode shape tests:** (5 tests)
1. `async fn foo() -> number { 42 }` compiles with AsyncCall-compatible metadata
2. `await foo()` emits Await opcode after Call
3. Sync fn `fn bar()` does NOT emit AsyncCall
4. WrapFuture emitted at return in async fn
5. Nested await compiles to two Await opcodes

**Minimum test count:** 5 tests (integration coverage in Phase 09/10)

---

## Acceptance Criteria

- ✅ `FunctionRef.is_async` field added
- ✅ Async fn bodies compile with correct metadata
- ✅ `Expr::Await` emits `Await` opcode
- ✅ Async calls emit `AsyncCall`
- ✅ `WrapFuture` emitted at return sites in async fn
- ✅ 5+ bytecode tests pass
- ✅ `cargo check -p atlas-runtime` clean

---

## References

**Decision Logs:** D-030
**Related phases:** Phase 06 (opcodes), Phase 09 (interpreter), Phase 10 (VM executes compiled output)
