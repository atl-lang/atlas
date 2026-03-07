# Phase 14: LSP Support

## Dependencies

**Required:** Phase 07 (typechecker), Phase 04 (parser)

**Verification:**
```bash
grep "async\|Await\|Future" crates/atlas-lsp/src/
cargo check -p atlas-lsp
```

---

## Objective

Full LSP support for async/await: hover shows `async fn` and `Future<T>` types, completion suggests `await` in async context, diagnostics surface AT4001/AT4002 in-editor.

---

## Files

**Update:** `crates/atlas-lsp/src/hover.rs` (or equivalent)
  - async fn hover: show `async fn name(...) -> Future<T>`
  - Future<T> variable hover: show resolved inner type
  - `await expr` hover: show T (the resolved type)
**Update:** `crates/atlas-lsp/src/completion.rs` (or equivalent)
  - Suggest `await` keyword when in async context
  - Complete `Future<T>` in type position
**Update:** `crates/atlas-lsp/src/diagnostics.rs` (or equivalent)
  - Surface AT4001, AT4002, AT4003 as LSP diagnostics with ranges

**Total new code:** ~80 lines LSP

---

## Implementation Notes

**Hover for async fn:** When hovering the function name of an `async fn`, display: `async fn name(params...) -> Future<ReturnType>`. When hovering a call to an async fn, show the `Future<T>` return type.

**Hover for `await`:** Show the resolved type `T` — the user wants to know what type they get after awaiting, not that it's a Future.

**Completion:** In async fn body, `await` should appear in keyword completions. In type position, `Future<` triggers generic type completion.

**Diagnostics:** AT4001 (await outside async) and AT4002 (await non-Future) should appear as LSP error squiggles inline.

---

## Tests

LSP tests use the existing LSP test harness:
1. Hover on async fn name → shows `async fn ... -> Future<T>`
2. Hover on `await expr` → shows resolved type T
3. Hover on Future<T> variable → shows `Future<number>` etc.
4. Completion in async fn body includes `await`
5. AT4001 appears as LSP diagnostic
6. AT4002 appears as LSP diagnostic
7. Non-async fn unaffected by async hover

**Minimum test count:** 7 tests

---

## Acceptance Criteria

- ✅ async fn hover correct
- ✅ await expr hover shows resolved type
- ✅ AT4001/AT4002 surface as LSP diagnostics
- ✅ `await` completion in async context
- ✅ 7+ LSP tests pass
- ✅ `cargo check -p atlas-lsp` clean

---

## References

**Decision Logs:** D-030
**Spec:** docs/language/async.md
**Related phases:** Phase 07 (typechecker provides type info), Phase 15 (AC check)
