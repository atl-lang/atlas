# Phase 01: AST Node for Anonymous Functions

**Block:** 4 (Closures + Anonymous Functions)
**Depends on:** Block 3 complete ✅

## Current State (verified 2026-02-23)

`Expr` in `ast.rs` has no anonymous function node. Named functions are `Stmt::FunctionDecl`. Closures currently only exist at runtime (`Value::Closure`) — there is no AST representation for an anonymous function *expression*.

`FunctionDecl` at `ast.rs:195` has: `name`, `type_params`, `params`, `return_type`, `return_ownership`, `body`, `doc_comment`, `span`.

## Goal

Add `Expr::AnonFn` to the AST. This is the shared target for both `fn(x: T) -> R { ... }` (Phase 02) and `(x) => expr` arrow syntax (Phase 03).

## Implementation

In `crates/atlas-runtime/src/ast.rs`, add to the `Expr` enum:

```rust
/// Anonymous function expression.
/// Syntax: `fn(x: number, y: number) -> number { x + y }`
/// Arrow:  `(x) => x + 1`  (desugared to this by the parser)
AnonFn {
    params: Vec<Param>,
    return_type: Option<TypeRef>,
    body: Box<Expr>,   // Block expression for fn body, single expr for arrow
    span: Span,
},
```

`Param` already exists at `ast.rs:187` with `name`, `type_ref`, `ownership`, `span` — reuse as-is.

`body` is `Box<Expr>` not `Vec<Stmt>` — the body is a block expression (`Expr::Block`) for the `fn` form, or a bare expression for the arrow form. This keeps the AST uniform.

## Visitor / match exhaustiveness

Add `Expr::AnonFn` to every `match expr` in:
- `interpreter/expr.rs` — add arm returning `Err` / `todo` initially (Phase 06 fills it)
- `compiler/expr.rs` — add arm returning `Err` / `todo` initially (Phase 05 fills it)
- `typechecker/` — add arm returning `Type::Unknown` initially (Phase 04 fills it)
- `binder.rs` — add arm (walk params and body)

**Do not leave `unreachable!()` — use a proper `Diagnostic` error "anonymous functions not yet supported" so the compiler fails gracefully before Phase 05/06 complete.**

## Acceptance Criteria

- [ ] `Expr::AnonFn` compiles with no warnings
- [ ] All match arms updated — no `non_exhaustive_patterns` warnings
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo test` passes (all existing tests unaffected)
