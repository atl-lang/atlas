# Atlas

## Identity (D-045 + D-060)
**"TypeScript's module system and type annotations wrapped around Rust's runtime model."**
- TypeScript has an answer → use it exactly
- TypeScript has no answer → design Atlas-native, minimal tokens, never copy Rust/Go surface
- Runtime model (CoW, ownership, Result/Option) → Rust-inspired, invisible to everyday code
- Full design spec: `docs/AI-DESIGN-PRINCIPLES.md`

## Build Rule (NON-NEGOTIABLE)
```bash
cargo build --release -p atlas-cli
```
NEVER: `cargo install`, debug builds, copy/symlink. PATH: `$HOME/dev/projects/atlas/target/release`

## Session Start (every agent, every session)
```bash
pt go                  # sitrep: session ID, handoff, P0s, CI, block
pt decisions CORE      # 17 critical decisions — read before any work
pt in-progress         # what's claimed — no duplicates
```
`pt decision D-XXX` for full detail. NOT `pt issue D-XXX`. See `.claude/lazy/pt-workflow.md`.

## Branch Rule (hook-enforced)
`.rs` or `Cargo.toml` dep changes → branch required. Never commit to main.
`fix/H-XXX` | `block/B-XX-name` | `feat/name`
Docs/config/`.claude/**` → main OK.

## Testing
**Pre-commit (auto):** `cargo fmt --check` + `cargo clippy` — no nextest
**Nightly CI:** full suite — `pt ci-status` for results. Failures = P0 blocker.
**Dev loop:** `cargo check -p atlas-runtime` → `cargo fmt` → `git commit`
**Banned:** all nextest except one exact-name TDD test (bugfix only)

## Doc Drift
Commit touches source → Stop hook shows DOC DRIFT ALERT → fire `atlas-doc-patch` agent (Haiku).
Never leave unresolved across sessions.

## Source of Truth
Code is law. Docs may be wrong. Spec: `docs/language/` + `docs/stdlib/`

## Auto-Loaded Rules
- `atlas-parity.md` — compiler output must match spec (BLOCKING)
- `atlas-fullstack.md` — full-stack feature completeness (BLOCKING)
- `atlas-diagnostics.md` — error quality D-043 (BLOCKING)
- `atlas-testing.md`, `atlas-ast.md`, `atlas-vm.md`, `atlas-syntax.md`
- `atlas-language-ref.md` — syntax quick reference
- `atlas-context-guard.md` — context window management
