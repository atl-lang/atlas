# IMPORTANT BEFORE CONTINUING (Read First)

Last Updated: 2026-02-28
Scope: Atlas overall health + required stabilization work before new phases

## Current Block Context (verified)
- Block 5 (Type Inference) is COMPLETE per `STATUS.md`.
- Next block is Block 6 (Error Handling, `?` operator) and Block 7 (JIT Integration) is unblocked.

## Non-Negotiable Fixes Before Scaling Further
1) **Runtime security enforcement is incomplete**
- Time/memory limits and `allow_network` are not enforced; sandbox integration is missing. See `docs/codex-findings/runtime-security.md`.

2) **JIT is dead code unless wired into VM**
- JIT exists but is not called by the runtime. See `docs/codex-findings/jit.md`.

3) **Bytecode serialization is TODO**
- Build outputs cannot be stable or cached without serialization. See `docs/codex-findings/bytecode-serialization.md`.

4) **LSP navigation/indexing is stubbed**
- Go-to-definition and cross-file indexing are incomplete. See `docs/codex-findings/lsp.md`.

5) **Error handling + unsafe perimeter need hardening**
- Non-test code has high `unwrap`/`panic` density and `unsafe` blocks concentrated in JIT/FFI/VM.
- See `docs/codex-findings/code-quality.md` for counts and hotspots.

6) **Compiler/VM parity risks remain**
- See `docs/codex-findings/runtime-compiler-vm.md`.

## Contextual Note
- Atlas is transitioning from a script-first language toward a systems-level language. That change **raises expectations** for correctness, determinism, sandboxing, and error discipline. These fixes should be treated as foundational.

## Action Gate (before new feature phases)
- Ship enforcement for runtime limits + security policy.
- Wire JIT or remove it until Block 7 begins.
- Finish bytecode serialization.
- Implement LSP position-based navigation and cross-file indexing.
- Reduce panic/unwrap in runtime/cli/build paths.

