---
name: atlas-blocks
description: Atlas block/phase execution. Scaffolding, gate sequence, phase handoff. Use when building new features via the phase system — "Scaffold Block", "Next Phase", "Start Phase".
---

# Atlas — Block & Phase Execution

**Prerequisite:** Core `atlas` skill must be active. If not, activate it first.

---

## Gate Sequence

1. **Run GATE -1** — full state audit (see `gates/gate-minus1-sanity.md`)
2. Run `pt sitrep` (check mode, P0 blockers, block progress)
2a. **Run the decision gate — before writing a single line of code** (see CLAUDE.md pt Gates):
    `pt decisions <component>` — if it covers your approach, follow it; if contradicts, stop.
3. **Git Setup:** GATE -1 determines branch state — see `gates/git-workflow.md`
4. Declare workflow type: **Structured Development**
5. **Execute gates** 0→1→2→3→4→5→6→7 (see `gates/gate-applicability.md`)
6. **Git Finalize:** Commit locally — see `gates/git-workflow.md`
7. Deliver completion summary

**Gate files:** All in `.claude/skills/atlas/gates/`

---

## Delegation Map

Lead directs — does not execute. See `gates/session-protection.md`.
- GATE -1, git ops, Rust implementation → always delegated
- Haiku: mechanical tasks, file scanning, build checks
- Sonnet: multi-file Rust implementation
- Opus: architecture, decisions, orchestration

---

## Scaffolding Protocol (trigger: "Scaffold Block N")

1. **Audit blast radius** — grep every file the block will touch
2. **Produce Block Kickoff doc:**
   ```
   Block N Kickoff: {Theme}
   Files affected: [verified list]
   Architectural decisions required: [none | list]
   Risks: [what could break outside this block]
   Phase list: [title + ~5 word description each]
   ```
3. **Present kickoff doc** — architect checkpoint
   - "Scaffold Block N" alone: present and wait
   - "Scaffold Block N, go": proceed immediately
4. **Create block branch:** `git checkout -b block/{name}`
5. **Register block + phases in DB — MANDATORY before any phase work:**
   ```bash
   pt block-add B<N> "Block Title" "Acceptance criteria"    # CREATE BLOCK FIRST
   # Then scaffold ALL phases in ONE call — see below
   pt phases B<N>                                           # verify after
   ```

   > ⚠️ **NEVER use `pt phase-add` in a loop.** It only sets title + description.
   > `scaffold-phases` sets ALL 9 fields in a single transaction. Always use it.

   **Use `pt scaffold-phases` — one JSON call, all phases, all fields:**
   ```bash
   echo '[
     {
       "title": "Phase 1 title",
       "description": "What this phase delivers and why — scope boundary",
       "deps": "none",
       "files": "crates/atlas-runtime/src/parser/mod.rs, crates/atlas-runtime/src/token.rs",
       "do": "What to implement — specific, actionable, no ambiguity",
       "dont": "What NOT to do — banned commands, out-of-scope, parity traps",
       "verify": "cargo check -p atlas-runtime",
       "ac": "Specific deliverables that must be TRUE when phase is done",
       "refs": "D-XXX, H-XXX, docs/language/foo.md"
     },
     {
       "title": "Phase 2 title",
       "description": "...",
       "deps": "B<N>-P01",
       "files": "crates/atlas-runtime/src/typechecker/expr.rs",
       "do": "...",
       "dont": "...",
       "verify": "cargo check -p atlas-runtime",
       "ac": "...",
       "refs": "D-XXX"
     }
   ]' | pt scaffold-phases B<N>
   ```

   **All 9 fields — all required, no exceptions:**
   | Field | What it gives cold-start agents |
   |-------|----------------------------------|
   | `title` | Phase identity |
   | `description` | Scope boundary — what's in, what's out |
   | `deps` | Ordering — what must be done first |
   | `files` | Exactly where to work — no guessing |
   | `do` | Implementation checklist |
   | `dont` | Guardrails — prevents known failure modes |
   | `verify` | Definition of done — exact command |
   | `ac` | Acceptance criteria for Gate 6 |
   | `refs` | Decisions + issues + docs for full context |

   Empty fields = agent guesses = errors and re-work. Fill all 9.
   `block-add` must come before `scaffold-phases` — block row must exist first.
6. Scaffold all phase files
7. Run `pt blocks` to verify block + phase count
8. **Commit scaffold — no push, no PR**

---

## Phase Handoff

**CRITICAL:** Only hand off when ALL tests pass AND commit is made.

**Protocol:**
1. All gates passed (build, tests, clippy, fmt, coderabbit, parity, battle tests)
2. Close any issues fixed this phase: `pt fix H-XXX "cause" "fix"` — do this NOW, not later
3. **Update block progress — NON-NEGOTIABLE, do this before session close:**
   ```bash
   pt phase-done B<N>-P<XX> "outcome summary"
   # If this was the FINAL phase — check AC:
   pt block B<N>          # Verify all AC are met
   pt complete-block B<N> "What was implemented. Any bugs filed (H-XXX)."
   ```
   Skipping this = next AI sees wrong block state and wastes a session re-deriving it.
4. **File any discovered bugs/issues NOW** — do NOT narrate them to the user. They evaporate.
   ```bash
   pt add "Bug: X causes Y" P0|P1 "battle test file, workaround, fix risk"
   pt link H-NEW related H-EXISTING  # if related to existing issue
   ```
5. Memory checked (GATE 7)
6. **Commit only** — local-first workflow
7. Close session — context is stored in DB, surfaced in next agent's `pt go`:

```bash
pt done <session-id> success \
  "Phase-XX complete: [what was wired up]. Fixed H-XXX (cause → fix). Filed H-YYY (bug discovered)." \
  "claim next phase B<N>-P<YY> — [one sentence: what needs doing and key files]"
```

**Summary rules:**
- What: name the phase + what it wired/implemented (not "did stuff")
- Issues: one clause per issue closed AND per issue filed this session
- Next: exact phase name + one sentence on scope — enough for a cold-start agent to orient
- No bullet dumps, no status theater ("PHASE COMPLETE - COMMITTED" adds zero signal)

**Anti-patterns — never do these:**
- "The next agent will need to look out for X" → File H-XXX with context. NOW.
- "We should probably Y" → Either do Y now or `pt add "Y" P2 "why"`.
- Skipping `phase-done` because "it's obvious" → It isn't. The DB is the source of truth.

---

## GATE V — Versioning (see `gates/gate-versioning.md`)

Run at two moments only:
- After final block of a version plan → minor version check
- After a `fix/` that corrects a bug in a tagged version → patch tag check

---

## Proactive File Sizing

**Canonical:** `.claude/lazy/architecture.md` — file size limits and split protocol.
GATE 0 Step 5 → GATE 1 → design split BEFORE writing. Reactive splits waste tokens.

---

## Git Workflow

**Canonical:** `.claude/lazy/git.md` — local-first, no remote push during development.
**Single workspace:** `~/dev/projects/atlas/`

---

## Deeper Reference
- Parity practices: auto-memory `compiler-quality/parity.md`
- Battle test strategy: auto-memory `compiler-quality/battle-testing.md`
- AI compiler lessons: auto-memory `compiler-quality/ai-compiler.md`
- Oracle testing: `gates/oracle-testing.md` (verify new features against Rust/TypeScript)
- Test partitioning: `gates/test-partitioning.md` (progressive test scope per gate)
- AI grammar principles: `gates/ai-grammar-principles.md` (syntax decisions)
