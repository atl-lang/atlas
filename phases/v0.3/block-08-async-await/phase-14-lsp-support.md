# Phase 14: LSP Support

## Dependencies

**Required:** Phase 13 complete (all runtime async features verified and battle-tested)

**Verification:**
```bash
grep "async\|Await\|Future" crates/atlas-lsp/src/
cargo check -p atlas-lsp
```

**If missing:** LSP reads type information produced by the typechecker (Phase 07) and AST from the parser (Phase 04) — both must be stable. Placing this phase after battle tests ensures the underlying implementation is solid before the editor layer is built on top.

---

## Objective

Full LSP support for async/await: hover shows correct types for async fns and awaited expressions, completion offers `await` in async contexts, and inline diagnostics surface AT4001/AT4002.

---

## Files

**Update:** `crates/atlas-lsp/src/hover.rs` (or equivalent hover handler) (~30 lines)
**Update:** `crates/atlas-lsp/src/completion.rs` (or equivalent completion handler) (~20 lines)
**Update:** `crates/atlas-lsp/src/diagnostics.rs` (or equivalent diagnostic publisher) (~15 lines)
**Tests:** `crates/atlas-lsp/tests/` or `crates/atlas-lsp/src/` inline tests (~7 test cases)

**Total new code:** ~80 lines LSP, ~30 lines tests
**Total tests:** ~7 test cases

---

## Dependencies (Components)

- `atlas-lsp` crate — hover, completion, diagnostics handlers (existing)
- Typechecker type annotations — async fn return types, await resolved types (Phase 07)
- AST nodes — `FunctionDecl.is_async`, `Expr::Await` (Phase 03)

---

## Implementation Notes

**Key patterns to analyze:**
- Examine how the LSP currently handles hover for regular functions — async fn hover follows the same path but prepends `async` and shows `Future<T>` as the return type
- Find how the LSP publishes diagnostics from typechecker results — AT4001/AT4002 are already in the typechecker output and just need to flow through
- Check the completion handler for keyword suggestions — `await` needs to appear when the cursor is inside an async fn body or at top-level

**Critical requirements:**

Hover on an async fn name: display `async fn name(params) -> Future<ReturnType>` — not the implicit `T`, but the actual return type of calling the function.

Hover on an `await` expression: display the resolved type `T` — the user cares what they get after awaiting, not that it was a Future.

Hover on a variable holding a `Future<T>`: display `Future<T>` with the inner type shown.

Completion: when the cursor is inside an `async fn` body or at top-level, `await` should appear in the keyword completion list. In type annotation position, `Future<` should trigger generic type completion.

Diagnostics: AT4001 (await outside async) and AT4002 (await non-Future) must appear as error squiggles with the correct source range and message text. These should already be in the typechecker output — the work here is ensuring they flow to the LSP diagnostic publisher.

**Error handling:**
- LSP must not crash on malformed async programs — graceful degradation if type information is unavailable

**Integration points:**
- Uses: typechecker type annotations (Phase 07), AST async nodes (Phase 03)
- Consumed by: VS Code extension or any LSP client

---

## Tests (TDD Approach)

**Hover** (3 tests)
1. Hovering the name of an `async fn` shows the signature with `async` and `Future<T>` return type
2. Hovering an `await expr` shows the resolved inner type `T`, not `Future<T>`
3. Hovering a `Future<number>` variable shows `Future<number>`

**Completion** (2 tests)
1. Inside an async fn body, `await` appears in keyword completions
2. In type annotation position, `Future` appears and triggers generic argument completion

**Diagnostics** (2 tests)
1. AT4001 (await outside async fn) appears as an LSP error diagnostic with the correct range
2. AT4002 (await on non-Future value) appears as an LSP error diagnostic with the correct range

**Minimum test count:** 7 tests

---

## Acceptance Criteria

- ✅ Async fn hover correct — shows `async fn ... -> Future<T>`
- ✅ `await` expression hover shows resolved type `T`
- ✅ AT4001 and AT4002 surface as LSP error diagnostics
- ✅ `await` keyword offered in completions inside async context
- ✅ 7+ LSP tests pass
- ✅ `cargo check -p atlas-lsp` clean

---

## References

**Decision Logs:** D-030
**Specifications:** docs/language/async.md
**Related phases:** Phase 07 (typechecker provides type info), Phase 15 (final AC gate)
