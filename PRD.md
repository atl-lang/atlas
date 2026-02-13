# Atlas PRD (Product Requirements Document)

## Summary
Atlas is a strict, typed, REPL-first programming language with a bytecode VM and a single cross-platform binary. It is designed to be readable, predictable, and AI-friendly. Atlas borrows proven ideas from TypeScript, Go, Rust, and Python while remaining cohesive and small.

## Vision
Create a language that feels natural for humans and AI agents, combining strict typing with a fast iteration loop (REPL) and a clear compilation path. Atlas must remain lightweight, deterministic, and easy to embed in tooling or applications later.

## Non-Negotiable Principles
- Strict typing: no implicit any, no implicit nullable.
- Clear diagnostics: precise error locations, helpful messages, and JSON diagnostics.
- Cohesion over feature sprawl: only add features when they're truly needed and well-designed.
- Single binary: no runtime dependencies.
- Cross-platform: macOS, Windows, Linux.
- Small surface area: keep syntax and stdlib focused and thoughtful.

## Primary Users
- Developers who want a strict, readable scripting language.
- AI agents that need consistent syntax and high-quality diagnostics.

## Goals
- REPL with type checking and safe evaluation.
- Bytecode compiler + VM for speed.
- Clear and deterministic CLI workflow.
- Strong error handling (human + JSON diagnostics).

## Features Under Careful Research
These features are important but require deep exploration before implementation:
- **Async/await** - Needs research on AI-friendly async patterns
- **JIT/native codegen** - Performance benefits vs complexity tradeoffs
- **Advanced types** (unions, intersections) - How far to push the type system?
- **Concurrency primitives** - Which model best serves AI-first principles?
- **Module system** - Avoiding pitfalls of existing approaches

**We add features when ready, not on a schedule.**

## Functional Requirements
- Parse and type-check `.atl` files.
- Evaluate scripts and REPL inputs.
- Emit bytecode for `.atl` files.
- Standard library includes: `print`, `len`, `str`.
- Errors include file, line, column, length, code, and hints.
- JSON diagnostic output for tooling/AI.

## Success Criteria
- Atlas can run programs with correct typing and clear diagnostics
- REPL handles errors gracefully without crashing
- Bytecode VM produces identical results to interpreter (parity verified)
- Type system catches bugs that would be runtime errors in dynamic languages
- Error messages are precise and actionable for both humans and AI agents

**Quality is measured by correctness and usability, not feature count or deadlines.**

## Design Constraints
- Language implemented in Rust.
- Runtime structured so it can be exposed as a library later.
- Minimal dependencies.

## Tooling Choices (v0.1)
- CLI: `clap`
- Errors: `thiserror`
- Diagnostics JSON: `serde`, `serde_json`
- REPL line editor: `rustyline` (default) or `reedline` if richer UX is required.

## Quality Bar
- Every phase must include tests.
- No ambiguous syntax.
- Diagnostics must be actionable and consistent.
- Engineering standards must align with `docs/engineering.md`.

## Deliverables
- `Atlas-SPEC.md` finalized for v0.1.
- `docs/engineering.md` and `tests/README.md`.
- `docs/diagnostics.md` and `docs/testing.md`.
- `docs/runtime.md`, `docs/modules.md`, `docs/ir.md`.
- `docs/value-model.md`.
- `docs/versioning.md` and `docs/style.md`.
- `docs/ai-principles.md`.
- `docs/debug-info.md`.
- `docs/ast-dump.md`, `docs/typecheck-dump.md`, `docs/runtime-api.md`.
- `docs/runtime-api-evolution.md`.
- `docs/prelude.md` and `docs/warnings.md`.
- `docs/bytecode-format.md`.
- `docs/top-level-execution.md`.
- `docs/operator-rules.md`.
- `docs/string-semantics.md`.
- `docs/array-aliasing.md`.
- `docs/diagnostic-normalization.md`.
- `docs/json-dump-stability.md`.
- `docs/decision-log.md`.
- `docs/coverage-matrix.md`.
- `docs/phase-gates.md`.
- `docs/numeric-edge-cases.md`.
- `docs/keyword-policy.md`.
- `docs/diagnostic-ordering.md`.
- `docs/diagnostic-normalization.md` and `docs/json-dump-stability.md`.
- `docs/e2e-parity.md`.
- `docs/repl-state.md`.
- `docs/cli-e2e.md`.
- Rust project scaffolding.
- Interpreter and VM.
- CLI tools.

### Archived Deliverables
The following test plan documents were created during v0.1 development and later archived to `archive/test-plans/` after their tests were integrated into phase files:
- `archive/test-plans/runtime-api-test-plan.md`
- `archive/test-plans/ast-typecheck-tests.md`
- `archive/test-plans/prelude-test-plan.md`
- `archive/test-plans/warnings-test-plan.md`
- `archive/test-plans/modules-test-plan.md` (modules are a post-v1.0 feature)

## Risks
- Scope creep from feature requests.
- Inconsistent diagnostics without a defined schema.
- Over-engineering early without a stable spec.

## Mitigations
- Phased development with clear scope for each phase
- Spec-first workflow with tests defining behavior
- Quality gates before moving to new features
- No feature is added until previous work is solid
- Honest assessment of progress, not wishful thinking
