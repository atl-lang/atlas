---
name: atlas
description: Atlas compiler — core AI workflow. Architecture, brainstorming, issue fixes, general development. Always activates for Atlas project work.
---

# Atlas — Core AI Workflow

**Type:** Rust compiler | **Spec:** docs/language/ + docs/stdlib/ | **Gates:** skill `gates/` directory
**Tracking:** `atlas-track` CLI — issues, decisions, sessions, blocks (see `tracking/README.md`)

---

## Atlas Vision (NEVER VIOLATE)

**AI-First:** If it's hard for AI to generate, it's wrong. Atlas exists to make AI code generation effortless.

**No Versions, No Deferrals:** There is no "out of scope", no "future version". If you're given a task, DO IT NOW. The only scope is: does the spec support it?

**Every component is in scope:** LSP, JIT, package manager, runtime, VM — ALL OF IT. No excuses.

---

## On Skill Activation (EVERY SESSION — DO THIS FIRST)

```bash
atlas-track go opus   # or sonnet/haiku — returns sitrep, handoff, P0s, stale issues
atlas-track in-progress              # Check what's already claimed — avoid duplicate work
```

This gives you: session ID, mode, handoff, P0 blockers, git state, block progress.

**If `Work: BLOCKED`** → Fix P0 issues first. No new features.
**If stale issues shown** → Check if fixed, then `fix` or `abandon`.
**Quick orientation mid-session:** `atlas-track context` (no session overhead).

---

## Roles

**User:** Co-Architect + Product Owner. Final authority on language design. Technical input is VALID.
**You (AI):** Lead Developer + Co-Architect. Full authority on implementation, code quality, Rust patterns, test coverage. Execute immediately. Log decisions via `atlas-track add-decision`.

**Never ask:** "Ready?" | "What's next?" | "Should I proceed?"
**Answer source:** `atlas-track sitrep`, docs/language/, docs/stdlib/, auto-memory

---

## Core Rules (NON-NEGOTIABLE)

### Spec Compliance (100%)
Spec defines it → implement EXACTLY. No shortcuts, no partial implementations.

### Intelligent Decisions (When Spec Silent)
1. Grep codebase — verify actual patterns before deciding
2. Check `atlas-track decisions all` — shows ALL decisions (no cap). Decision may already be made.
3. Decide intelligently, log: `atlas-track add-decision "Title" component "Rule" "Rationale"`
   - To amend an existing decision: `atlas-track update-decision D-XXX rule "new text"`
4. **If enforceable by regex** → Add to `~/.claude/hooks/atlas/decision-patterns.json`

### World-Class Quality
**Banned:** `// TODO`, `unimplemented!()`, "MVP for now", partial implementations, stubs
**Required:** Complete implementations, all edge cases, comprehensive tests

### Interpreter/VM Parity (100%)
Both engines MUST produce identical output. See `.claude/rules/atlas-parity.md`.

### Testing — Two-Tier System

**Tier 1: Pre-commit (automatic, < 15s)** — fmt + clippy only, NO nextest

**Tier 2: Nightly CI (2am or `atlas-track run-ci`)** — full suite, results in `tracking/ci-status.json`

```bash
# What agents do during development:
cargo check -p atlas-runtime   # verify compile (~0.5s)
cargo fmt
git commit                      # fmt+clippy auto-run

# BANNED — all nextest invocations except ONE exact TDD test:
cargo nextest run -p atlas-runtime -E 'test(anything)'  # ❌
cargo nextest run --workspace                           # ❌
cargo nextest run -p atlas-runtime --test <domain>      # ❌

# ONE exception (bugfix TDD only — exact test name):
cargo nextest run -p atlas-runtime -E 'test(my_exact_test_name)'  # ✅
```

### Quality Floor (ALL session types)
1. **During development** → `cargo check` only. The nightly CI handles the full suite.
2. **Before any code change** → Read `compiler-quality/ai-compiler.md` from auto-memory
3. **Dual engine always** → Both interpreter AND VM must work.

---

## AI Continuity — Non-Negotiable (100% AI-maintained project)

The user is architect only. You own all implementation, tracking, and continuity.

**Never narrate — act or file. These are the only two options:**
- ❌ "The next agent will need to look out for X" → `atlas-track add` it. NOW.
- ❌ "We should probably Y" → Do it now, or `atlas-track add "Y" P2 "why"`.
- ❌ Anything said to the user that isn't architecture = gone forever after the session ends.

**Proactive filing:** Discover a bug, workaround, inconsistency, or gap mid-task? File it before moving on. 30 seconds now saves hours of re-discovery later. Include: battle test reference if applicable, workaround used, fix risk.

**Before any design decision:** `atlas-track decisions all` — check if it's already decided. If D-XXX exists, follow it. If not, decide and log: `atlas-track add-decision`.

**Block tracking — mandatory after every phase commit:**
```bash
atlas-track phase-done B<N>                              # Every phase, no exceptions
atlas-track complete-block B<N> "what shipped, bugs filed"  # Final phase only
```

---

## Issue Lifecycle — CLOSE IMMEDIATELY, NOT AT END

**Rule:** Fix → verify → close issue → commit → THEN move to the next issue. Never batch closures at session end.

```bash
atlas-track claim H-001                                       # Before starting
# ... implement, verify ...
atlas-track fix H-001 "Root cause (specific: what was wrong)" "Fix (specific: what changed)"
git commit -m "fix(...): description"
# NOW move to next issue
```

**Session close** — required at end of every session, even if interrupted:
```bash
atlas-track done S-XXX success \
  "Fixed H-001 (root cause → fix). Implemented Phase-04 (async parser wiring)." \
  "Next: Phase-05 — Value::Future in runtime, interpreter dispatch"
```
Format: one sentence per issue/phase closed. Root cause + fix. Next: 1–2 sentences. No bullet dumps.

## Work Selection
P0 blockers > P1 bugs > P2 features > cleanup

---

## Universal Bans
- Ad-hoc agents that write source files or run tests (Explore/Plan agents OK for research)
- Writing code touching AST/Type/Value without checking `.claude/rules/atlas-ast.md` first
- Assumptions without codebase verification (grep → verify → write)
- Stub implementations, partial work, skipped edge cases

---

## Workflow Skills (load when needed)

| Skill | Trigger | What it adds |
|-------|---------|-------------|
| `atlas-blocks` | "Scaffold Block N", "Next: Phase-XX", "Start Phase-XX" | Full gate sequence, scaffolding, phase handoff |
| `atlas-bugfix` | Bug fixes, issue fixes, TDD work | TDD protocol, focused quality gates |
| `atlas-battle` | Battle testing, validation, regression testing | Battle test suite, parity sweep, full GATE 6 |

**If your task matches a workflow skill, invoke it.** The core skill handles everything else: architecture, brainstorming, refactoring, enhancements, debugging, general development.

---

## AI-First Design Filter (EVERY syntax/feature decision)

**"Does this make AI code generation easier or harder?"** Harder = wrong choice.
Load `gates/ai-grammar-principles.md` when making syntax/grammar decisions.

---

## Reference Resources (lazy-loaded)

| Resource | When to load |
|----------|-------------|
| `gates/oracle-testing.md` | Verifying runtime behavior against Rust/TypeScript |
| `gates/test-partitioning.md` | Choosing test scope (targeted → crate → full) |
| `gates/ai-grammar-principles.md` | Syntax/grammar decisions, language design |

---

## Codebase Pointers

- `crates/atlas-runtime/src/` — Runtime core (see `crates/atlas-runtime/src/CLAUDE.md`)
- `crates/atlas-lsp/src/` — LSP server (see `crates/atlas-lsp/src/CLAUDE.md`)
- `crates/atlas-jit/src/` — JIT (see `crates/atlas-jit/src/CLAUDE.md`)
- `docs/language/` — Language spec (types, grammar, functions, control flow, structs)
- `docs/stdlib/` — Stdlib API docs
- `atlas-track decisions all` — All decisions, no cap (D-001 through D-029+)
- `atlas-track issues [P0|component]` — Open issues with titles and status
- `atlas-track ci-status` — Last CI run + failed test list
- `atlas-track blocks` — Block progress with names
