# Phase 01: AST + Parser — Optional Return Type

**Block:** 5 (Type Inference)
**Depends on:** Block 4 complete

## Goal

Make `-> ReturnType` optional on named function declarations. This is the highest blast-radius
change in the block and must land first so all subsequent phases build on it.

## AST Change

`FunctionDecl.return_type: TypeRef` → `Option<TypeRef>`:

```rust
pub struct FunctionDecl {
    pub name: Identifier,
    pub type_params: Vec<TypeParam>,
    pub params: Vec<Param>,
    pub return_type: Option<TypeRef>,   // None = inferred
    pub return_ownership: Option<OwnershipAnnotation>,
    pub predicate: Option<TypePredicate>,
    pub body: Block,
    pub span: Span,
}
```

## Parser Change

`fn foo(params)` (no `->`) is valid. Parser emits `return_type: None`.
`fn foo(params) -> T` is still valid. Parser emits `return_type: Some(T)`.

No syntax change — `->` remains valid. This is purely additive.

## Consumer Fixups (~20 sites)

Every place that accesses `func.return_type` must be updated:

| File | Pattern | Fix |
|------|---------|-----|
| `typechecker/mod.rs` | `self.resolve_type_ref(&func.return_type)` | `func.return_type.as_ref().map_or(Type::Unknown, ...)` |
| `binder.rs` (hoist_function x2, bind_function) | same | same |
| `compiler/mod.rs` | `func.return_type` reference | `func.return_type.as_ref().map_or(...)` |
| `interpreter/mod.rs` | `func.return_type` reference | same |
| `lsp/hover.rs` | render return type | `Option` → show nothing or "inferred" |
| `lsp/inlay_hints.rs` | return type inlay | handle None |
| `lsp/semantic_tokens.rs` | if any | handle None |
| `typechecker/expr.rs` (check_anon_fn) | `return_type_ref: Option<&TypeRef>` | already Option — no change |

**Strategy:** `func.return_type.as_ref().map_or(Type::Unknown, |t| self.resolve_type_ref(t))`
wherever the return type is needed before inference runs. Inference wires in Phase 03.

## Test

```atlas
fn double(x: number) { return x * 2; }   // return type omitted → inferred
double(5);                                 // → 10
```

Both engines must run this without error.

## Acceptance Criteria

- [ ] `FunctionDecl.return_type: Option<TypeRef>`
- [ ] Parser accepts `fn foo(params) { body }` with no `->` annotation
- [ ] All consumers compile (no `E0308` type errors)
- [ ] Existing tests still pass (no regressions)
- [ ] `cargo build --workspace` clean
