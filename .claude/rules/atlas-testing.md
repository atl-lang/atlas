---
globs: ["crates/atlas-runtime/tests/**", "crates/atlas-lsp/tests/**", "crates/**/tests/**"]
---

# Atlas Testing Rules

Auto-loaded when touching test files. Full patterns in auto-memory `testing-patterns.md`.

## Cardinal Rule: No New Test Files in atlas-runtime

Every new test file = a new binary = more link time + slower CI. **Add to existing domain files.**

| Domain | File |
|--------|------|
| Lexer, parser, syntax | `tests/frontend_syntax.rs` |
| Diagnostics, error spans | `tests/diagnostics.rs` |
| Full frontend pipeline | `tests/frontend_integration.rs` |
| Type inference, generics | `tests/typesystem.rs` |
| Interpreter execution | `tests/interpreter.rs` |
| VM execution | `tests/vm.rs` |
| Stdlib functions | `tests/stdlib.rs` |
| Collections (HashMap, Set, Queue) | `tests/collections.rs` |
| Bytecode compiler, optimizer | `tests/bytecode.rs` |
| Async, futures, channels | `tests/async_runtime.rs` |
| Closures | `tests/closures.rs` |
| Pattern matching | `tests/pattern_matching.rs` |
| FFI | `tests/ffi.rs` |
| Security, permissions | `tests/security.rs` |
| Regression (bug reproductions) | `tests/regression.rs` |

Exception: explicit approval required for genuinely new domains.

## Preferred: Corpus Tests

New language behavior → write `.atlas` files in `crates/atlas-runtime/tests/corpus/`:
- `pass/foo.atlas` + `pass/foo.stdout` — must run and produce expected output
- `fail/bar.atlas` + `fail/bar.stderr` — must produce specific error
- Generate expected: `UPDATE_CORPUS=1 cargo nextest run -p atlas-runtime --test corpus`

Corpus tests auto-verify parity (runs both interpreter and VM). Prefer corpus over Rust tests.

## Parity Pattern (mandatory for interpreter/VM work)

```rust
#[test]
fn test_feature_parity() {
    assert_parity(r#"len("hello")"#, "5");
}
```

Never write separate `test_feature_interpreter` + `test_feature_vm` functions.

## `#[ignore]` Rules

Bare `#[ignore]` is banned. Always give a reason:
```rust
#[ignore = "requires network"]
#[ignore = "not yet implemented: closure-capture"]
```

## LSP Tests — Different Pattern

LSP tests **cannot** use helper functions for server creation (lifetime error). Every test inlines:
```rust
#[tokio::test]
async fn test_feature() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();
    // inline all setup here
}
```

Add LSP tests to existing files in `crates/atlas-lsp/tests/`. Creating new LSP test files is allowed (different from runtime — no binary bloat issue).

## Run Commands

```bash
cargo nextest run -p atlas-runtime --test closures   # one domain file
cargo nextest run -p atlas-runtime                   # full suite (~15-20s)
cargo nextest run -p atlas-runtime --test corpus     # corpus only
```
