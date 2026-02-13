# Phase 01 - Foundation

## Objective
Establish the project skeleton, build system, and testing harness in Rust. No language logic yet.

## Inputs
- `docs/guides/code-quality-standards.md`
- `docs/guides/code-quality-standards.md`

## Deliverables
- Rust workspace with crates: `atlas-runtime` (library) and `atlas-cli` (binary).
- CI-ready test runner (cargo test).
- Linting/formatting defaults (rustfmt).
- README with build/run instructions.
- Rust edition: 2021.
- Dependency policy: keep external deps minimal; no parser generators.
- Baseline dependencies selected and documented:
  - `clap` (CLI)
  - `thiserror` (error types)
  - `serde` + `serde_json` (JSON diagnostics)
  - `insta` optional (golden tests)

## Steps
- Create a Rust workspace with `crates/atlas-runtime` and `crates/atlas-cli`.
- Add a minimal `main` to `atlas-cli` with placeholder subcommands.
- Ensure `cargo fmt` works and `cargo test` runs.

## Exit Criteria
- `cargo test` runs and passes (even if only smoke tests).
- `atlas-cli` binary builds on macOS.
- Workspace layout exists:
  - `crates/atlas-runtime`
  - `crates/atlas-cli`
  - `tests/`
- `docs/guides/code-quality-standards.md` exists and is referenced in README.

## Tests
- A simple unit test in `atlas-runtime` that always passes.
- A CLI smoke test that validates the binary runs and exits 0.
