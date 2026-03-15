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
pt go                  # sitrep: session ID, handoff, P0s, CI, block, gotchas, todos
pt decisions CORE      # 17 critical decisions — read before any work
pt in-progress         # what's claimed — no duplicates
```
`pt decision D-XXX` for full detail. NOT `pt issue D-XXX`. See `.claude/lazy/pt-workflow.md`.

## Gotchas (read before touching anything)
`pt go` surfaces critical gotchas automatically. Full list: `pt gotchas all`
File a new trap: `pt gotcha add "title" component "detail" --severity critical|warning`
Confirm still relevant: `pt gotcha confirm G-XXX`

## Attempts (log dead ends immediately)
`pt tried H-XXX "approach" "what happened"` — prevents future agents repeating failures
`pt attempts H-XXX` — see what's already been tried before starting

## TODO Queue (persistent across sessions)
`pt todos` — cross-session action items (survives session close, unlike next_steps)
`pt todo add "title" ["detail"] [P1]`
`pt todo done T-XXX`

## Orientation (cold agent ramp-up)
```bash
pt narrative 5         # last 5 sessions — what was done, decisions made
pt hotspots            # most-changed files in last 30 days
pt active              # files linked to open/in-progress issues
```

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
