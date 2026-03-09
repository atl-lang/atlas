---
name: atlas
description: Atlas compiler — core AI workflow. Architecture, brainstorming, issue fixes, general development. Always activates for Atlas project work.
---

# Atlas — Core AI Workflow

**Type:** Rust compiler | **Spec:** docs/language/ + docs/stdlib/ | **Gates:** skill `gates/` directory
**Tracking:** `pt` CLI — issues, decisions, sessions, blocks (see `tracking/README.md`)

---

## Atlas Vision (NEVER VIOLATE)

**AI-First:** If it's hard for AI to generate, it's wrong. Atlas exists to make AI code generation effortless.

**No Versions, No Deferrals:** There is no "out of scope", no "future version". If you're given a task, DO IT NOW. The only scope is: does the spec support it?

**Every component is in scope:** LSP, JIT, package manager, runtime, VM — ALL OF IT. No excuses.

---

## On Skill Activation (EVERY SESSION — DO THIS FIRST)

```bash
pt go   # returns sitrep, handoff, P0s, stale issues (model detected automatically)
pt in-progress              # Check what's already claimed — avoid duplicate work
```

**If sitrep shows `📄 ~/.project-tracker/handoffs/atlas-handoff.md` → READ IT NOW. Not optional.**
```bash
# Read the handoff file — it has in-flight work, next action, critical context
# that cannot fit in the 300-char DB summary. It is written by every agent at close.
```

This gives you: session ID, mode, handoff, P0 blockers, git state, block progress, active plans.

**If `Work: BLOCKED`** → Fix P0 issues first. No new features.
**If stale issues shown** → Check if fixed, then `fix` or `abandon`.
**Active plans:** `pt plans` — shows open PL-XXX plans (brainstorm outcomes).
**Quick orientation mid-session:** `pt context` (no session overhead).

**After orienting — invoke the right skill BEFORE doing anything else:**
| Situation | Action |
|-----------|--------|
| Task is a bug fix / issue fix | Invoke `atlas-bugfix` skill via Skill tool |
| Task is a new block / feature phase | Invoke `atlas-blocks` skill via Skill tool |
| Task is "what should we build?" / B10 direction / design question | Invoke `atlas-brainstorm` skill via Skill tool |
| Task is adding/fixing tests | Invoke `atlas-test` skill via Skill tool |
| Task is battle testing / validation | Invoke `atlas-battle` skill via Skill tool |

**Do not answer the user before invoking the matching skill.** The skill loads the protocol that governs how you execute the task. Answering first = skipping the protocol.

---

## Roles

**User:** Co-Architect + Product Owner. Final authority on language design. Technical input is VALID.
**You (AI):** Lead Developer + Co-Architect. Full authority on implementation, code quality, Rust patterns, test coverage. Execute immediately. Log decisions via `pt add-decision`.

**Never ask:** "Ready?" | "What's next?" | "Should I proceed?"
**Answer source:** `pt sitrep`, docs/language/, docs/stdlib/, auto-memory

---

## Core Rules (NON-NEGOTIABLE)

### Spec Compliance (100%)
Spec defines it → implement EXACTLY. No shortcuts, no partial implementations.

### Intelligent Decisions (When Spec Silent)
1. Grep codebase — verify actual patterns before deciding
2. Check `pt decisions all` — shows ALL decisions (no cap). Decision may already be made.
3. Decide intelligently, log: `pt add-decision "Title" component "Rule" "Rationale"`
   - To amend an existing decision: `pt update-decision D-XXX rule "new text"`
4. **If enforceable by regex** → Add to `~/.claude/hooks/atlas/decision-patterns.json`

### World-Class Quality
**Banned:** `// TODO`, `unimplemented!()`, "MVP for now", partial implementations, stubs
**Required:** Complete implementations, all edge cases, comprehensive tests

### Interpreter/VM Parity (100%)
Both engines MUST produce identical output. See `.claude/rules/atlas-parity.md`.

### Testing — Two-Tier System

**Tier 1: Pre-commit (automatic, < 15s)** — fmt + clippy only, NO nextest

**Tier 2: Nightly CI (2am or `pt run-ci`)** — full suite, results in `tracking/ci-status.json`

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
- ❌ "The next agent will need to look out for X" → `pt add` it. NOW.
- ❌ "We should probably Y" → Do it now, or `pt add "Y" P2 "why"`.
- ❌ Anything said to the user that isn't architecture = gone forever after the session ends.

**Proactive filing:** Discover a bug, workaround, inconsistency, or gap mid-task? File it before moving on. 30 seconds now saves hours of re-discovery later. Include: battle test reference if applicable, workaround used, fix risk.

**Before touching any component — run the decision gate (mandatory, not aspirational):**
```bash
pt decisions <component>   # parser|typechecker|vm|interpreter|stdlib|runtime|lsp|infra
# Returns 3-8 lines. Takes 2 seconds. Skipping it risks violating a standing decision.
```
Map your task to a component, run it, read it. If a decision covers your change — follow it.
If your change contradicts a decision — stop and discuss with the architect before proceeding.
If no decision exists for your design choice — make the call, then log it:
```bash
pt add-decision "Title" <component> "Rule: what was decided" "Rationale: why"
```

**Block tracking — mandatory after every phase commit:**
```bash
pt phase-done B<N>-P<XX> "outcome"              # Every phase, no exceptions
pt complete-block B<N> "what shipped, bugs filed"  # Final phase only (after pt block B<N> AC check)
```

---

## Issue Lifecycle — CLOSE IMMEDIATELY, NOT AT END

**Rule:** Fix → verify → close issue → commit → THEN move to the next issue. Never batch closures at session end.

```bash
pt claim H-001                                       # Before starting
# ... implement, verify ...
pt fix H-001 "Root cause (specific: what was wrong)" "Fix (specific: what changed)"
git commit -m "fix(...): description"
# NOW move to next issue
```

**Session close** — required at end of every session, even if interrupted:

```bash
pt done S-XXX success \
  "Fixed H-001 (root cause → fix). Implemented Phase-04 (async parser wiring)." \
  "claim H-002 — fix Y in crates/atlas-runtime/src/eval.rs, grep for fn eval_expr"
```

- **Arg 3 (summary):** backward-looking — what was done. One clause per issue/phase, root cause + fix. No bullet dumps.
- **Arg 4 (next):** forward-looking — what the next agent does first. Issue ID + action + file/function. Specific enough to act on cold.

The next agent sees this as `── Next Action (from last session) ──` in `pt go`. No file to write, no extra steps.

## Work Selection
P0 blockers > P1 bugs > P2 features > cleanup

---

## Universal Bans
- Ad-hoc agents that write source files or run tests (Explore/Plan agents OK for research)
- Writing code touching AST/Type/Value without checking `.claude/rules/atlas-ast.md` first
- Assumptions without codebase verification (grep → verify → write)
- Stub implementations, partial work, skipped edge cases

---

## Workflow Skills (MANDATORY — use Skill tool, not just awareness)

| Skill | Trigger | What it adds |
|-------|---------|-------------|
| `atlas-brainstorm` | Design questions, "what to build", B-selection, tradeoff evaluation | Context-first exploration, plan capture |
| `atlas-blocks` | "Scaffold Block N", "Next Phase", feature implementation | Full gate sequence, scaffolding, phase handoff |
| `atlas-bugfix` | Bug fixes, issue fixes, TDD work | TDD protocol, focused quality gates |
| `atlas-battle` | Battle testing, validation, regression testing | Battle test suite, parity sweep, full GATE 6 |
| `atlas-test` | Writing tests, coverage gaps, test failures | Test domain rules, parity protocol |

**These are not reference docs — they are execution protocols. Use the Skill tool to invoke them.**
The core skill handles only what no other skill covers: refactoring, doc updates, infra, ad-hoc tasks.

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
- `pt decisions all` — All decisions, no cap (D-001 through D-029+)
- `pt issues [P0|component]` — Open issues with titles and status
- `pt ci-status` — Last CI run + failed test list
- `pt blocks` — Block progress with names
