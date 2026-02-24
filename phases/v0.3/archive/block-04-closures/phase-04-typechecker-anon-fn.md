# Phase 04: Typechecker — Anonymous Function + Capture Validation

**Block:** 4 (Closures + Anonymous Functions)
**Depends on:** Phase 03 complete

## Current State (verified 2026-02-23)

`Type::Function { type_params, params, return_type }` exists in `types.rs:43`.
`TypeRef::Function { params, return_type, span }` exists in `ast.rs:604`.
The typechecker already resolves `TypeRef::Function` → `Type::Function` in `mod.rs:520`.

**What's missing:** handling `Expr::AnonFn` in the typechecker's expression checker.

## Goal

1. Typecheck `Expr::AnonFn` → produce `Type::Function`
2. Validate capture semantics: `Copy` types captured by copy, non-`Copy` by move

## Implementation

In `typechecker/expr.rs` (or wherever `check_expr` dispatches), add:

```rust
Expr::AnonFn { params, return_type, body, span } => {
    // Push a new scope for the anonymous function body
    // Register params as locals in that scope
    // Check body in that scope
    // Return Type::Function { type_params: vec![], params: param_types, return_type }
}
```

### Param type resolution

For typed params (`type_ref = Some(...)`): resolve via `resolve_type_ref()`.
For untyped params (`type_ref = None`): assign `Type::Unknown` — inference handles it later (Block 5). Do not error — arrow fns commonly omit types.

### Return type

If `return_type` annotation present: resolve and check body against it.
If absent: infer from body's last expression type.

### Capture validation

When the body references a variable from an outer scope:
- Check if its type implements `Copy` (via trait registry from Block 3)
- `Copy` type → captured by copy, no ownership transfer
- Non-`Copy` type → captured by move, caller loses ownership
- `borrow` annotated variable → **error**: cannot capture a borrow into a closure (borrows cannot outlive their scope)

### Higher-order context

When an anon fn is passed to a function expecting `fn(T) -> R`, verify the anon fn's param/return types are compatible with the expected function type.

## Acceptance Criteria

- [ ] `let f = fn(x: number) -> number { x + 1; };` typechecks, `f` has type `(number) -> number`
- [ ] `let f = (x) => x + 1;` typechecks without error (untyped params → Unknown, no crash)
- [ ] Passing a non-Copy type into a closure produces a move (no error)
- [ ] Capturing a `borrow` variable in a closure produces a clear diagnostic
- [ ] `cargo test` passes
