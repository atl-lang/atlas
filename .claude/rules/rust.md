---
globs: ["**/*.rs"]
---

# Rust Patterns — Atlas

## Production Code

- `Result<T, E>` everywhere — no `unwrap()` in production code
- `?` for error propagation — no `.unwrap()` chains
- `Arc::make_mut()` for CoW mutation (never `Arc::try_unwrap` on shared data)
- `#[cfg(debug_assertions)]` for debug-only checks (ownership enforcement, bounds)
- Derive order: `Debug, Clone, PartialEq` (add `Serialize, Deserialize` for AST nodes)

## Clippy Standards

All crates run `cargo clippy -- -D warnings`. Zero tolerance. Fix before committing.
Common Atlas-specific issues:
- CoW collections: clippy may warn on `Arc::make_mut` patterns — these are intentional
- `match` on `Value` variants must be exhaustive — no `_ =>` catchalls in new code

## Value Enum Rules

Adding a `Value` variant requires updating ALL of:
1. `type_name()` in `value.rs`
2. `Display` impl
3. `PartialEq` impl + equality semantics table in `memory-model.md`
4. Bytecode serialization (`bytecode/serialize.rs`)
5. Interpreter eval (`interpreter/mod.rs` + `expr.rs`)
6. VM execution (`vm/mod.rs`)
7. Any stdlib function that pattern-matches `Value`

Never add a variant without updating all sites — the compiler will catch missing match arms.

## Test Code

- `rstest` for parameterized tests
- `insta` for snapshot tests
- `proptest` for property-based tests
- `assert_cmd` for CLI integration tests
- No `unwrap()` in test assertions — use `expect("context")` for clarity
