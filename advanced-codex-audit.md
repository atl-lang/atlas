# Atlas Advanced Codex Audit (Brutal Assessment)

Date: 2026-03-02
Repo: /Users/proxikal/dev/projects/atlas
Scope: Atlas compiler/runtime/tooling/docs against AI-first + systems-level vision

## Methodology
- Repo-wide scans for `unwrap/expect/panic/todo` in non-test code.
- Review of existing audit docs in `docs/codex-findings/`.
- Targeted review of core runtime/compiler/VM/LSP/JIT/security surfaces.
- CI pipeline review for quality gates.

**Honesty note:** This is a deep audit based on repo-wide scans and critical surface reviews. It is not a manual, line-by-line inspection of every file. If you want a true line-by-line audit, I can run that next and append the results here.

---

## Executive Summary (Critical Misalignments)
Atlas is currently **over-promising** on sandbox security, determinism, and AI-first tooling. The gaps below are **not cosmetic**; they will constrain future systems-level ambitions and AI-first reliability. The highest-impact issues are:

1) Security enforcement is missing (sandbox is mostly declared, not enforced).
2) Bytecode serialization is incomplete (no stable compiler artifacts).
3) JIT exists but is not wired and is ABI-incorrect (dangerous, worse than absent).
4) Compiler/VM parity is broken in side-effecting paths.
5) LSP navigation/indexing is stubbed (tooling is not AI-first yet).
6) Error discipline is not systems-grade (panic/unwrap density in prod paths).
7) Docs/spec drift breaks AI-first trust and tooling correctness.

---

## A. Security & Sandbox (Vision-Critical, High Risk)
**Constraint gap:** Sandbox claims are not enforced; FFI escapes policy.

Findings:
- `RuntimeConfig` exposes `allow_network`, `max_execution_time`, and `max_memory_bytes`, but enforcement is missing. This violates the “AI-safe sandbox” claim.
  - Files: `crates/atlas-runtime/src/api/runtime.rs`, `crates/atlas-runtime/src/api/config.rs`
- Sandbox/quota system exists but is not integrated into execution.
  - File: `crates/atlas-runtime/src/security/sandbox.rs`
- FFI calls are executed without permission checks in both interpreter and VM. This is a direct policy bypass.
  - Files: `crates/atlas-runtime/src/interpreter/expr.rs`, `crates/atlas-runtime/src/vm/mod.rs`
- `ResourceType` variants (FFI, NetworkListen, Reflection) are not mapped to permissions and are ignored in policy evaluation.
  - File: `crates/atlas-runtime/src/security/policy.rs`

What Atlas should be doing instead:
- Enforce time/memory quotas in VM/interpreter loops and allocation paths.
- Split IO/network permissions and enforce them independently.
- Gate all FFI calls behind explicit policy checks (deny by default).
- Ensure every resource type is enforced or rejected at validation.

---

## B. Compiler Correctness & Determinism (Systems-Level Baseline)
**Constraint gap:** Runtime semantics are not guaranteed to match compiler semantics.

Findings:
- Compound assignment on indexed targets re-evaluates side-effecting expressions, violating semantics and determinism.
  - File: `crates/atlas-runtime/src/compiler/stmt.rs`
- VM defines `And/Or` opcodes but executes them as unknown; any bytecode using them will fail.
  - Files: `crates/atlas-runtime/src/vm/mod.rs`, `crates/atlas-runtime/src/vm/dispatch.rs`

What Atlas should be doing instead:
- Use stack rotation or temp locals so targets are evaluated exactly once.
- Either implement `And/Or` opcodes or remove them and prevent emission.

---

## C. Artifact Stability (Real Compiler Pipeline)
**Constraint gap:** No stable bytecode format exists, blocking caching and reproducibility.

Findings:
- Bytecode serialization panics on many constant types; format is explicitly TODO.
  - File: `crates/atlas-runtime/src/bytecode/serialize.rs`
- Build pipeline acknowledges TODO for bytecode serialization.
  - File: `crates/atlas-build/src/builder.rs`

What Atlas should be doing instead:
- Define a versioned, checksummed bytecode format.
- Implement round-trip serialization tests.
- Use serialized bytecode for build outputs and caches.

---

## D. JIT Integration (Currently Unsafe + Non-Functional)
**Constraint gap:** JIT exists but is unused and ABI-incorrect.

Findings:
- JIT is not wired into runtime/VM; it is dead code.
  - Files: `crates/atlas-jit/src/CLAUDE.md`, `crates/atlas-jit/JIT_STATUS.md`
- JIT compiles all functions as zero-arg, ignoring parameters. This is ABI-incorrect and can crash.
  - File: `crates/atlas-jit/src/lib.rs`
- Cache size accounting is hard-coded and inaccurate.
  - File: `crates/atlas-jit/src/cache.rs`

What Atlas should be doing instead:
- Wire JIT into VM dispatch or remove it until Block 7.
- Compile with correct signature per arity.
- Track code size realistically.

---

## E. AI-First Tooling Gaps (LSP/Refactor)
**Constraint gap:** Tooling claims exceed current implementation.

Findings:
- goto-definition is stubbed; returns `None`.
  - File: `crates/atlas-lsp/src/server.rs`
- Symbol ranges use default Range due to missing span conversions.
  - File: `crates/atlas-lsp/src/navigation.rs`
- Cross-file indexing is TODO.
  - File: `crates/atlas-lsp/src/index.rs`
- Refactor extract ignores captures and return type inference.
  - File: `crates/atlas-lsp/src/refactor/extract.rs`

What Atlas should be doing instead:
- Implement span→range conversion and use real ranges everywhere.
- Build workspace-wide symbol index with position-based lookups.
- Capture analysis + return type inference for refactors.

---

## F. Error Discipline (Not Compiler-Grade)
**Constraint gap:** Production paths rely on panic/unwrap, undermining stability.

Findings:
- High density of `unwrap/expect/panic` in non-test runtime/cli/build paths.
  - Reference: `docs/codex-findings/code-quality.md`
- Serialization panics on unsupported constants rather than returning errors.
  - File: `crates/atlas-runtime/src/bytecode/serialize.rs`

What Atlas should be doing instead:
- Replace panics with structured errors and error codes.
- Add CI budgets for `panic/unwrap` in non-test code.

---

## G. Unsafe Blocks Without Invariants
**Constraint gap:** Unsafe code exists but lacks explicit safety contracts.

Findings:
- Unsafe usage concentrated in JIT/VM/FFI without invariants.
  - Reference: `docs/codex-findings/code-quality.md`

What Atlas should be doing instead:
- Add safety comments outlining invariants and preconditions above each unsafe cluster.

---

## H. Documentation Drift (AI-First Trust Failure)
**Constraint gap:** Docs/spec contradict runtime behavior.

Findings:
- Memory model docs still describe Arc<Mutex> aliasing, contradicting CoW model.
  - Files: `docs/specification/language-semantics.md`, `docs/interpreter-status.md`, `docs/embedding-guide.md`
- Spec error codes do not match runtime codes.
  - File: `docs/specification/types.md`
- JIT docs reference old memory model and incorrect defaults.
  - File: `crates/atlas-jit/JIT_STATUS.md`

What Atlas should be doing instead:
- Treat spec as authoritative and enforce sync with runtime.
- Add CI drift checks between runtime error codes and docs.

---

## I. REPL and Package Pipeline Gaps
**Constraint gap:** Docs imply readiness where functionality is incomplete.

Findings:
- REPL stdout capture is TODO.
  - File: `crates/atlas-runtime/src/repl.rs`
- Package install/publish pipeline has multiple TODOs (download, tests, tarball).
  - Files: `crates/atlas-cli/src/commands/install.rs`, `crates/atlas-cli/src/commands/publish.rs`

What Atlas should be doing instead:
- Either implement these paths or mark them clearly as unsupported.

---

## J. CI & Quality Gates (Strong but Incomplete)
**Positive:** CI is strong with fmt, clippy, nextest, fuzz, coverage.
  - File: `.github/workflows/ci.yml`

**Missing quality gates:**
- No enforcement on panic/unwrap budgets in production code.
- No spec/runtime drift checks.
- No policy that blocks TODOs in release-critical files.

---

## Top 10 Constraint Gaps (Prioritized)
1) Sandbox enforcement missing (time/memory/network) + FFI bypass.
2) Bytecode serialization absent.
3) JIT not wired + ABI incorrect.
4) Compiler/VM parity broken on side-effecting expressions.
5) LSP navigation/indexing stubbed.
6) Panic/unwrap density in production paths.
7) Unsafe blocks lack safety invariants.
8) Docs/spec drift on memory model and error codes.
9) REPL stdout capture TODO.
10) Package registry/install/publish workflows incomplete.

---

## Recommended Remediation Order
1) Security enforcement + FFI policy integration.
2) Stable bytecode serialization + round-trip tests.
3) Compiler/VM parity fixes + opcode cleanup.
4) Wire or remove JIT; fix ABI and cache accounting.
5) LSP core navigation + cross-file indexing.
6) Replace panic/unwrap with structured errors + CI budgets.
7) Add safety invariants for unsafe blocks.
8) Fix docs/spec drift + add CI drift checks.
9) REPL stdout capture or doc downgrade.
10) Finish or disable placeholder package workflows.

---

## Next Step Options
- Full line-by-line audit of every crate with explicit line references and patch plan.
- Immediate remediation plan with patches for the top 3 blockers (security, serialization, parity).

