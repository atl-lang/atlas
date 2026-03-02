# CLI Decisions

## DR-001: Full Permissions by Default ⚠️ CRITICAL
**Status:** Active | **Priority:** CRITICAL

`atlas run` grants full system permissions. NO permission flags.

```rust
let runtime = Atlas::new_with_security(SecurityContext::allow_all());
```

**Industry standard:** Go, Rust, Python, Node.js all grant full access.
**NOT like Deno:** No `--allow-net`, `--allow-read` flags.

**SecurityContext is for embedding** (running untrusted code in apps), NOT CLI.

**DO NOT CHANGE:** Future AI must NOT add permission flags to CLI.

## DR-002: Testing Architecture - Rust/Go Model ⚠️ ARCHITECTURE
**Status:** Active | **Priority:** ARCHITECTURE

Testing follows `cargo test` / `go test` model:
- **Stdlib:** Assertion primitives only (`assert()`, `assertEqual()`)
- **CLI:** Full test runner (discovery, execution, reporting)

**NO separate testing framework.** Stdlib = assertions, CLI = orchestration.

**DO NOT:** Create duplicate test runners.
