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
2. Run `atlas-track done <session-id> success "summary" "next steps"`
3. Memory checked (GATE 7)
4. **Commit only** — local-first workflow
5. Deliver summary

**Required in summary:**
- Status: "PHASE COMPLETE - COMMITTED"
- Final Stats (bullets)
- **Memory:** Updated X / No updates needed (MANDATORY)
- Progress (run `atlas-track blocks`)
- Next phase

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
