# Docs Sweep Findings (2026-02-28)

Scope: docs/ (non-archive), cross-checked against repo code.
Status: Open

## Finding 1: LSP docs claim production-ready, but core navigation is stubbed

Evidence:
- docs/lsp-status.md: status "Production Ready"
- docs/lsp-features.md: status "Production Ready"
- docs/lsp-navigation.md: status "Complete"
- Code: `goto_definition` stubbed and symbol ranges defaulted in:
  - /Users/proxikal/dev/projects/atlas/crates/atlas-lsp/src/server.rs
  - /Users/proxikal/dev/projects/atlas/crates/atlas-lsp/src/navigation.rs
  - /Users/proxikal/dev/projects/atlas/crates/atlas-lsp/src/index.rs

Impact:
- Editor navigation is incomplete; docs overstate readiness.

Recommendation:
- Downgrade LSP status docs or complete position-based navigation + cross-file indexing.

---

## Finding 2: CLI docs claim production-ready, but install/publish are stubbed

Evidence:
- docs/cli-status.md: status "Production-Ready"
- Code stubs:
  - /Users/proxikal/dev/projects/atlas/crates/atlas-cli/src/commands/install.rs
  - /Users/proxikal/dev/projects/atlas/crates/atlas-cli/src/commands/publish.rs

Impact:
- Users will assume registry behaviors exist; they do not.

Recommendation:
- Mark CLI status as "partial" or implement install/publish behavior.

---

## Finding 3: Build system docs imply full implementation, but bytecode serialization is TODO

Evidence:
- docs/build-system.md: status "Implemented"
- Code: bytecode serialization TODO in
  - /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/bytecode/serialize.rs
  - /Users/proxikal/dev/projects/atlas/crates/atlas-build/src/builder.rs

Impact:
- Artifact stability and caching are not guaranteed.

Recommendation:
- Document the gap or implement serialization.

---

## Finding 4: REPL docs imply production-ready, but stdout capture is TODO

Evidence:
- docs/interpreter-status.md: status "Production-Ready"
- Code: stdout capture TODO in
  - /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/repl.rs

Impact:
- Tooling cannot reliably inspect REPL output.

Recommendation:
- Implement stdout capture or mark limitation in docs.

---

## Finding 5: Embedding guide still references `Arc<Mutex<T>>` for host state

Evidence:
- docs/embedding-guide.md: uses `Arc<Mutex<T>>` for host state management.

Impact:
- This is valid for host state, but may be read as general guidance and conflict with the CoW value model in language semantics.

Recommendation:
- Clarify that `Arc<Mutex<T>>` is for host-side state only, not language value representation.

---

## Finding 6: LSP refactoring doc lists TODOs matching known gaps

Evidence:
- docs/lsp-refactoring.md lists TODOs for captured variables, return type inference, cross-file support, span tracking.
- These match current LSP code limitations.

Impact:
- Documentation is accurate here, but the status line says Phase 04 complete while key items remain.

Recommendation:
- Update status to reflect outstanding TODOs or align the implementation.

