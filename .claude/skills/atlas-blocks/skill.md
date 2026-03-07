---
name: atlas-blocks
description: Atlas block/phase execution. Scaffolding, gate sequence, phase handoff. Use when building new features via the phase system — "Scaffold Block", "Next Phase", "Start Phase".
---

# Atlas — Block & Phase Execution

**Prerequisite:** Core `atlas` skill must be active. If not, activate it first.

---

## Gate Sequence

1. **Run GATE -1** — full state audit (see `gates/gate-minus1-sanity.md`)
2. Run `atlas-track sitrep` (check mode, P0 blockers, block progress)
2a. **Run the decision gate — before writing a single line of code:**
    ```bash
    atlas-track decisions <component>   # e.g. typechecker, parser, vm, stdlib, runtime
    ```
    3-8 lines back. If a decision covers your approach — follow it.
    If your plan contradicts one — stop, surface to architect.
    New design choice not covered — decide, then: `atlas-track add-decision`
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
5. Scaffold all phase files
6. Run `atlas-track blocks` to verify
7. **Commit scaffold — no push, no PR**

---

## Phase Handoff

**CRITICAL:** Only hand off when ALL tests pass AND commit is made.

**Protocol:**
1. All gates passed (build, tests, clippy, fmt, coderabbit, parity, battle tests)
2. Close any issues fixed this phase: `atlas-track fix H-XXX "cause" "fix"` — do this NOW, not later
3. **Update block progress — NON-NEGOTIABLE, do this before session close:**
   ```bash
   atlas-track phase-done B<N>
   # If this was the FINAL phase — check AC:
   atlas-track block B<N>          # Verify all AC are met
   atlas-track complete-block B<N> "What was implemented. Any bugs filed (H-XXX)."
   ```
   Skipping this = next AI sees wrong block state and wastes a session re-deriving it.
4. **File any discovered bugs/issues NOW** — do NOT narrate them to the user. They evaporate.
   ```bash
   atlas-track add "Bug: X causes Y" P0|P1 "battle test file, workaround, fix risk"
   atlas-track link H-NEW related H-EXISTING  # if related to existing issue
   ```
5. Memory checked (GATE 7)
6. **Commit only** — local-first workflow
7. Close session with informative summary (for AI continuity):
```bash
atlas-track done <session-id> success \
  "Phase-XX complete: [what was wired up]. Fixed H-XXX (cause → fix). Filed H-YYY (bug discovered)." \
  "Next: Phase-YY — [one sentence: what needs doing and why]"
```

**Summary rules:**
- What: name the phase + what it wired/implemented (not "did stuff")
- Issues: one clause per issue closed AND per issue filed this session
- Next: exact phase name + one sentence on scope — enough for a cold-start agent to orient
- No bullet dumps, no status theater ("PHASE COMPLETE - COMMITTED" adds zero signal)

**Anti-patterns — never do these:**
- "The next agent will need to look out for X" → File H-XXX with context. NOW.
- "We should probably Y" → Either do Y now or `atlas-track add "Y" P2 "why"`.
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
