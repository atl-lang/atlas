---
paths:
  - "**/tests/**/*.rs"
  - "**/*_test.rs"
---

# Atlas Testing Rules

## atlas-runtime: NO NEW TEST FILES

Tests live in `crates/atlas-runtime/tests/` domain files. Every new file = new binary = slower CI.
Add to an existing domain file. See `crates/atlas-runtime/src/CLAUDE.md` for the full domain table.

Exception: genuinely new domain with no existing home → explicit approval required.

## atlas-lsp: INLINE SERVER CREATION ONLY

```rust
#[tokio::test]
async fn test_something() {
    let (service, socket) = LspService::new(|client| AtlasLspServer::new(client));
    // test body here — no extracted helpers
}
```

**No helper functions for server setup. Ever.** Each test is fully self-contained.

## atlas-cli: assert_cmd

```rust
use assert_cmd::Command;
let mut cmd = Command::cargo_bin("atlas").unwrap();
cmd.arg("run").arg("file.atlas").assert().success();
```

## Parity Tests

Every runtime behavior change needs a parity test — same source, both engines, identical output:

```rust
fn assert_parity(source: &str, expected: &str) {
    let interp = eval_interpreter(source);
    let vm = eval_vm(source);
    assert_eq!(interp, vm, "Parity divergence");
    assert_eq!(interp, expected);
}
```

Add to the relevant domain file. Never create a standalone parity test file.

## Build Commands

```bash
cargo nextest run -p atlas-runtime              # runtime tests
cargo nextest run -p atlas-lsp                  # LSP tests
cargo nextest run -p atlas-cli                  # CLI tests
cargo nextest run --workspace                   # everything
cargo clippy -p atlas-runtime -- -D warnings    # lint
cargo fmt --check -p atlas-runtime              # format
```
