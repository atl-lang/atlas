# Phase 08: LSP — fn Type Hover + Completion

**Block:** 4 (Closures + Anonymous Functions)
**Depends on:** Phase 04 complete

## Current State (verified 2026-02-23)

LSP hover in `atlas-lsp/src/actions.rs` resolves types via the typechecker's symbol table.
`Type::Function { params, return_type, .. }` exists but its `Display` / hover rendering has not been verified for the anonymous fn case.
Completion in `atlas-lsp/src/references.rs` suggests identifiers from scope.

## Goal

1. Hover over an anonymous fn variable shows its type as `fn(number) -> number`
2. Hover over a higher-order fn param shows the expected fn type
3. Completion inside a call site expecting `fn(T) -> R` suggests compatible closures from scope

## Implementation

### Hover rendering

Verify `Type::Function` renders correctly in the LSP hover response. If `Display` for `Type` doesn't handle `Function`, add it:

```rust
Type::Function { params, return_type, .. } => {
    let params_str = params.iter().map(|p| format!("{p}")).collect::<Vec<_>>().join(", ");
    write!(f, "fn({params_str}) -> {return_type}")
}
```

### Variable hover

When hovering over `let f = (x: number) => x + 1`, the hover should show:
```
f: fn(number) -> number
```

Verify the typechecker annotates `f`'s type in the symbol table after Phase 04.

### Completion

When a function parameter expects `fn(number) -> number`, completion should filter scope to show only variables with a compatible function type. This is a best-effort filter — do not error if no compatible completions exist, just show all.

### New LSP tests

Add to `atlas-lsp/tests/lsp_actions_tests.rs`:
- Hover test: `let f = fn(x: number) -> number { x; };` → hover `f` → `fn(number) -> number`
- Hover test: higher-order param `fn apply(f: (number) -> number, x: number) -> number` → hover `f` → `fn(number) -> number`

## Acceptance Criteria

- [ ] Hover on anon fn variable shows correct `fn(T) -> R` type string
- [ ] Hover on higher-order fn parameter shows fn type
- [ ] Existing LSP tests pass (no regressions)
- [ ] Minimum 2 new LSP hover tests added
- [ ] `cargo test` passes
