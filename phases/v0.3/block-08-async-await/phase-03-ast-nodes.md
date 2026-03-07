# Phase 03: AST Nodes

## Dependencies

**Required:** Phase 02 complete (keywords in lexer and token)

**Verification:**
```bash
grep "is_async" crates/atlas-runtime/src/ast.rs
grep "Expr::Await\|TypeRef::Future" crates/atlas-runtime/src/ast.rs
cargo check -p atlas-runtime
```

**If missing:** Phase 02 must be complete so keyword variants exist — though AST is independent of lexer, phases must proceed in order.

---

## Objective

Extend the AST with all nodes required for async/await: `is_async` on `FunctionDecl` and `MethodDecl`, `Expr::Await`, and `TypeRef::Future`. Increment `AST_VERSION`.

---

## Files

**Update:** `crates/atlas-runtime/src/ast.rs` (~20 lines added across multiple structs/enums)
**Update:** `crates/atlas-formatter/src/` — formatter constructs `FunctionDecl`, must compile after field addition

**Total new code:** ~20 lines
**Total tests:** 0 (compile verification only — parser tests in Phase 04 exercise these nodes)

---

## Dependencies (Components)

- `ast.rs` — FunctionDecl, MethodDecl, Expr, TypeRef (existing)
- `atlas-formatter` crate — constructs FunctionDecl (blast radius)

---

## Implementation Notes

**Key patterns to analyze:**
- Search `FunctionDecl {` across the entire codebase to find all construction sites — every one must be updated when `is_async: bool` is added
- Check `Expr::span()` match — adding `Expr::Await` requires a new arm
- Examine how `Expr::AnonFn` is structured to understand the pattern for a new Expr variant with a nested expression

**Critical requirements:**
- `FunctionDecl.is_async` defaults to `false` — all existing construction sites set it to `false` explicitly
- `MethodDecl.is_async` follows the same pattern as `FunctionDecl`
- `Expr::Await` wraps a `Box<Expr>` plus a `Span`
- `TypeRef::Future` wraps a `Box<TypeRef>` plus a `Span` — represents `Future<T>` in type position
- `AST_VERSION` constant must be incremented (breaking change to the JSON dump format)
- All Serde derives are already in place — new fields and variants serialize automatically

**Error handling:**
- No AT codes emitted by AST — that is the typechecker's responsibility

**Integration points:**
- Uses: `ast.rs` structs (existing)
- Creates: `is_async` field, `Expr::Await` variant, `TypeRef::Future` variant
- Blast radius: parser (Phase 04), formatter, compiler, interpreter, VM — all will need exhaustiveness fixes

---

## Tests (TDD Approach)

No new test cases in this phase — the Rust compiler enforces exhaustiveness. Verify:
1. `cargo check -p atlas-runtime` passes with zero warnings
2. `cargo check -p atlas-formatter` passes (formatter constructs FunctionDecl)
3. All existing construction sites compile with the new `is_async: false` field

**Minimum test count:** 0 (compile-only gate)

---

## Acceptance Criteria

- ✅ `FunctionDecl.is_async: bool` added, all construction sites updated to `is_async: false`
- ✅ `MethodDecl.is_async: bool` added, all construction sites updated
- ✅ `Expr::Await { expr: Box<Expr>, span: Span }` variant added with span() arm
- ✅ `TypeRef::Future { inner: Box<TypeRef>, span: Span }` variant added
- ✅ `AST_VERSION` incremented
- ✅ `cargo check -p atlas-runtime` clean — zero warnings
- ✅ `cargo check -p atlas-formatter` clean — zero warnings

---

## References

**Decision Logs:** D-030
**Specifications:** docs/language/async.md (type system and syntax sections)
**Related phases:** Phase 02 (keywords), Phase 04 (parser constructs these nodes), Phase 05 (Value::Future is the runtime twin of TypeRef::Future)
