# Atlas

## Philosophy
- **AI-first.** "What's best for AI?" is the decision tiebreaker.
- **No MVP.** Complete implementations only. Do it right once.
- **100% AI developed.** This project is built entirely by AI.

## Build Rule (NON-NEGOTIABLE)

```bash
cargo build --release -p atlas-cli   # The ONLY valid build command
```

**NEVER:** `cargo install`, `cargo build` (debug), any copy/symlink step after build.
**PATH:** `$HOME/dev/projects/atlas/target/release` is in PATH. Build → immediately live.

## Atlas Identity (D-045)

**"Atlas is TypeScript's module system and type annotations wrapped around Rust's runtime model."**

1. TypeScript has an answer → use it
2. TypeScript has no answer (systems-level) → design Atlas-native, never copy Rust/Go
3. Runtime model (CoW, ownership, Result/Option) → Rust-inspired, keep it

See MEMORY.md for locked decisions (D-026, D-039) and active syntax issues.

## Source of Truth
- **Code is law.** The codebase is the only source of truth.
- **Docs may be wrong.** If docs contradict code, docs are wrong.
- **Test against reality.** Run `atlas check` and `atlas run` to verify.
- **Spec:** `docs/language/` and `docs/stdlib/`

## Guardian Protocol
- **Verify before agreeing.** User doubts? Check facts first.
- **Protect Atlas from everyone.** User confusion, AI shortcuts, bad ideas—all threats.
- **User is architect, not infallible.** Explain why something is wrong.
- **Pushback on scope creep.** P0 blockers exist? Fix those first.

## Git Process

**Branch mandate (hook-enforced):**
- `.rs` or `Cargo.toml` dep changes → MUST be on a branch, never main
- `fix/H-XXX` | `block/B-XX-name` | `feat/name`
- Docs/config/`.claude/**` → may commit to main
- **Check branch before writing Rust:** `git branch --show-current`

**Branch cleanup:** `pt done` auto-deletes merged branches, blocks on unmerged.

## Testing — Two-Tier System

**Tier 1: Pre-commit (automatic, < 15s)**
- `cargo fmt --check` + `cargo clippy`
- NO nextest

**Tier 2: Nightly CI (2am or `pt run-ci`)**
- Full corpus, test suite, parity sweep
- Results: `tracking/ci-status.json`
- CI failures = P0 blocker

**During development:**
```bash
cargo check -p atlas-runtime   # verify compile (~0.5s)
cargo fmt
git commit                      # fmt+clippy auto-run
```

**BANNED:** All manual nextest except TDD (bugfix skill, exact test name only).

## Doc Drift Protocol

After commits touching source files:
1. `doc-patch-trigger.sh` writes `.doc-patch-pending.json`
2. Stop hook shows DOC DRIFT ALERT
3. Fire `atlas-doc-patch` agent (Haiku, ~1-2 min)

Never leave unresolved across sessions.

## pt Workflow (invoke `atlas` skill for full protocol)

```bash
pt go                    # Session start — sitrep, handoff, P0s, CI
pt in-progress           # Check in-flight work
pt next                  # Smart triage
pt claim H-XXX           # Before starting issue
pt fix H-XXX "cause" "fix" "scope-audit"   # Close issue (auto-notes session)
pt phase-done B<N>-P<XX> "outcome"         # After phase (auto-notes session)
pt done S-XXX success "summary" "next"     # Session end (auto-cleans branches)
```

**Full workflow:** Invoke `atlas` skill → loads complete protocol, gates, sub-skills.

## Auto-Loaded Rules

These load automatically based on file patterns:
- `atlas-parity.md` — Interpreter/VM/typechecker parity (BLOCKING)
- `atlas-fullstack.md` — Full-stack feature completeness (BLOCKING)
- `atlas-diagnostics.md` — Error quality D-043 (BLOCKING)
- `atlas-testing.md` — Test organization
- `atlas-ast.md`, `atlas-typechecker.md`, `atlas-vm.md`, `atlas-syntax.md`
- `atlas-language-ref.md` — Syntax quick reference
- `atlas-context-guard.md` — Context window management
