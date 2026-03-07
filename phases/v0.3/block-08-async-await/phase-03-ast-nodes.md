# Phase 03: AST Nodes

## Dependencies

**Required:** Phase 02 (keywords in lexer/token)

**Verification:**
```bash
grep "is_async\|Expr::Await\|TypeRef::Future" crates/atlas-runtime/src/ast.rs
cargo check -p atlas-runtime
```

---

## Objective

Extend the AST with all nodes required for async/await: `is_async` flag on `FunctionDecl` and `MethodDecl`, `Expr::Await`, and `TypeRef::Future<T>`.

---

## Files

**Update:** `crates/atlas-runtime/src/ast.rs`
  - `FunctionDecl`: add `pub is_async: bool` (+1 field)
  - `MethodDecl`: add `pub is_async: bool` (+1 field)
  - `Expr` enum: add `Await { expr: Box<Expr>, span: Span }` variant
  - `TypeRef` enum: add `Future { inner: Box<TypeRef>, span: Span }` variant
  - Update `Expr::span()` match arm for `Await`
  - Update `Stmt::span()` if needed

**Total new code:** ~20 lines

---

## Implementation Notes

**`FunctionDecl.is_async`:** Default `false`. Serializes in JSON dump (Serde derives already present). Adding a field to `FunctionDecl` requires updating all construction sites — grep for `FunctionDecl {` to find all.

**`Expr::Await`:** Wraps a single expression (the future). Box to avoid recursive size issues.

**`TypeRef::Future`:** Box inner type. Printed as `Future<T>` in diagnostics. Used by typechecker to express async fn return types.

**Blast radius of `FunctionDecl` field addition:**
- Parser construction sites (parser/mod.rs)
- Formatter (atlas-formatter)
- Any pattern exhaustiveness — Rust compiler will flag these

**AST version:** Increment `AST_VERSION` constant (breaking change to JSON dump format).

---

## Tests

No new runtime tests needed for pure AST additions. The parser tests (Phase 04) will exercise these nodes. Verify:
1. `FunctionDecl { is_async: false, .. }` compiles at all existing construction sites
2. `Expr::Await { .. }` arm is handled in span() match

**Minimum test count:** 0 (compile-only verification, tests in Phase 04)

---

## Acceptance Criteria

- ✅ `FunctionDecl.is_async: bool` added, all construction sites updated
- ✅ `MethodDecl.is_async: bool` added, all construction sites updated
- ✅ `Expr::Await` variant added with span()
- ✅ `TypeRef::Future` variant added
- ✅ `AST_VERSION` incremented
- ✅ `cargo check -p atlas-runtime` clean — NO new warnings
- ✅ `cargo check -p atlas-formatter` clean (formatter uses FunctionDecl)

---

## References

**Decision Logs:** D-030
**Spec:** docs/language/async.md
**Related phases:** Phase 02 (keywords), Phase 04 (parser constructs these), Phase 07 (typechecker)
