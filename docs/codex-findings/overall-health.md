# Atlas Overall Health Snapshot (Repo-Verified)

Date: 2026-02-28
Scope: Full workspace scan (code + config). Tests excluded from size stats.

## Size/Footprint (non-test code)
- Total Rust code across crates: ~72k LOC
- Runtime (`atlas-runtime`): ~41k LOC
- CLI: ~9k LOC
- LSP: ~7.4k LOC
- Build: ~4.6k LOC
- Package: ~3.0k LOC
- JIT: ~1.1k LOC

## Core Components Present
- Lexer, parser, typechecker, compiler, bytecode, VM, interpreter
- Optimizer passes, debugger, profiler, async runtime
- FFI layer + sandbox/policy scaffolding
- JIT crate (Cranelift-backed)
- LSP server and refactor scaffolding
- Formatter

## Status Summary (verified)
- CI is strong (fmt, clippy -D warnings, deny, nextest, matrix) — see `.github/workflows/ci.yml`.
- Test infrastructure is extensive (especially in runtime), but a refactor is in progress.
- Major functionality exists, but several critical systems are incomplete (security enforcement, bytecode serialization, JIT integration, LSP navigation).

## Blocking Risks Before Scaling
- Runtime sandboxing/limits not enforced (see `runtime-security.md`).
- JIT not wired into VM (see `jit.md`).
- Bytecode serialization TODO (see `bytecode-serialization.md`).
- LSP navigation/indexing incomplete (see `lsp.md`).
- Unwrap/panic density in runtime/cli/build (see `code-quality.md`).

