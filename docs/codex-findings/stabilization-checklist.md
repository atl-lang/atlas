# Stabilization Checklist (Before New Feature Blocks)

Date: 2026-02-28
Scope: Issues that should be fixed before scaling or adding new phases.

## P0 (Blocker)
1. Enforce runtime limits and sandboxing
- Implement `max_execution_time` and `max_memory_bytes` checks.
- Enforce `allow_network` separately from `allow_io`.
- Thread `security::sandbox` through runtime execution.
- Reference: `docs/codex-findings/runtime-security.md`.

2. Wire JIT into VM or remove it until Block 7
- Integrate `JitEngine::notify_call` in VM call dispatch.
- Ensure fallback behavior on unsupported opcodes.
- Reference: `docs/codex-findings/jit.md`.

3. Implement bytecode serialization
- Define a stable, versioned, checksummed format.
- Add round-trip tests and integrate into build outputs.
- Reference: `docs/codex-findings/bytecode-serialization.md`.

4. Fix LSP navigation and indexing
- Implement span-to-range conversion, go-to-definition, and cross-file indexing.
- Reference: `docs/codex-findings/lsp.md`.

## P1 (High)
1. Reduce panic/unwrap in runtime/cli/build paths
- Replace with structured errors and propagate context.
- Reference: `docs/codex-findings/code-quality.md`.

2. Resolve compiler/VM parity risks
- Fix compound assignment side-effect duplication.
- Implement or remove `And/Or` opcode handling.
- Reference: `docs/codex-findings/runtime-compiler-vm.md`.

3. Tighten unsafe invariants
- Add explicit safety invariants for unsafe blocks in JIT/FFI/VM.
- Reference: `docs/codex-findings/code-quality.md`.

## P2 (Medium)
1. Re-enable ignored tests that no longer apply
- Revisit Arc<Mutex> deadlock ignores under CoW.
- Reference: `docs/codex-findings/tests-ignored.md`.

2. Update docs status flags to match implementation
- Especially LSP, CLI, build-system, interpreter status.
- Reference: `docs/codex-findings/docs-sweep-2026-02-28.md`.

