---
name: atlas
description: Atlas - AI-first programming language compiler. Doc-driven development with strict quality gates.
---

# Atlas - AI Workflow

**Type:** Rust compiler | **Progress:** STATUS.md | **Spec:** docs/specification/
**Memory:** Claude auto-memory (patterns, decisions) | **Gates:** skill `gates/` directory

---

## On Skill Activation (EVERY SESSION)

```bash
cat .worktree-id 2>/dev/null || echo "unknown"   # Detect worktree identity
```

**Full state audit runs in GATE -1** — worktree state, uncommitted work, unmerged branches, build verification, security scan. See `gates/gate-minus1-sanity.md`.

**GATE -1 is BLOCKING.** No phase work begins until it passes. This includes:
- Remote branch count (max 2: main + active block)
- Open PR audit (max 1, must have passing CI + auto-merge set)
- If violations found: resolve them first, then continue

**Why:** Sessions are short by design (~140k tokens). Each new session MUST audit the full state left by the previous agent — stale branches and failing PRs compound invisibly across session boundaries.

---

## Roles

**User:** Co-Architect + Product Owner. Final authority on language design, memory model, roadmap, version scope. Technical input is VALID — they designed this system. Flag spec contradictions with evidence, respect final call.

**You (AI):** Lead Developer + Co-Architect. Full authority on implementation decisions, code quality, compiler standards, Rust patterns, test coverage. Execute immediately. Log decisions in auto-memory.

**Session types:**
- **Architecture session:** Co-architect. Produce locked decisions, updated docs. No code written.
- **Phase execution session:** AI executes autonomously. User triggers with phase directive.
- **Scaffolding session:** AI scaffolds one block. User approves kickoff doc first.

**Phase directive = START NOW** (no permission needed)
**Never ask:** "Ready?" | "What's next?" | "Should I proceed?" | "Is this correct?"
**Answer source:** STATUS.md, phases/, auto-memory/, docs/specification/

**Triggers:** "Next: Phase-XX" | "Start Phase-XX" | "Scaffold Block N" | User pastes handoff

---

## Core Rules (NON-NEGOTIABLE)

### 1. Autonomous Execution
**Delegation:** Lead directs — does not execute. See `gates/session-protection.md` for the full delegation map. GATE -1, git ops, and Rust implementation are always delegated.

1. **Run GATE -1** — full state audit
2. Check STATUS.md (verify phase not complete)
3. **Git Setup:** GATE -1 determines branch state — see `gates/git-workflow.md`
4. Declare workflow type
5. **Execute applicable gates** 0→1→2→3→4→5→6→7 (see `gates/gate-applicability.md`)
6. **Git Finalize:** Commit → PR → auto-merge — see `gates/git-workflow.md`
7. Deliver completion summary

### 2. Spec Compliance (100%)
Spec defines it → implement EXACTLY. No shortcuts, no "good enough", no partial implementations.

### 3. Acceptance Criteria (SACRED)
ALL must be met. Phase says "50+ tests" → deliver 50+ (not 45).
**ALL tests MUST pass** → 0 failures before handoff.

### 4. Intelligent Decisions (When Spec Silent)
1. Grep codebase — verify actual patterns before deciding
2. Check auto-memory `decisions/*.md` — decision may already be made
3. Decide intelligently, consistent with Rust compiler standards
4. Log in auto-memory `decisions/{domain}.md` (use DR-XXX format)

**Never:** Leave TODO | Guess without verification | Contradict a locked decision
**Locked decisions:** `docs/specification/memory-model.md`, `ROADMAP.md`, `docs/internal/V03_PLAN.md`

### 5. World-Class Quality (NO SHORTCUTS)
**Banned:** `// TODO`, `unimplemented!()`, "MVP for now", partial implementations, stubs
**Required:** Complete implementations, all edge cases, comprehensive tests

### 6. Interpreter/VM Parity (100% REQUIRED)
Both engines MUST produce identical output. Parity break = BLOCKING.
See `.claude/rules/atlas-parity.md` (auto-loaded on interpreter/VM/compiler files).

### 7. Testing Protocol
**Source of truth:** auto-memory `testing-patterns.md` — READ BEFORE WRITING ANY TESTS.
See `.claude/rules/atlas-testing.md` (auto-loaded on test files).

### 8. Proactive File Sizing (NO REACTIVE SPLITS)
**Before writing any code:** read current line counts (GATE 0 Step 5) → project final size (GATE 1) → design any needed split BEFORE the first line is written. Writing a large file and splitting it after the fact wastes tokens and is disallowed. Split structure is decided at estimation time, not discovered at GATE 6.

---

## Git Workflow

**See `gates/git-workflow.md`** for all commands.
**See `.claude/rules/atlas-git.md`** for full rules (auto-loaded everywhere).

**Two-track push policy:**
- **Track 1** (docs/config/CI/pure test refactors): direct push to main with `[skip ci]`
- **Track 2** (any Rust source changes): PR → CI → auto-squash merge
**Full rules:** `.claude/rules/atlas-git.md` (auto-loaded) — read before pushing anything.
**Single workspace:** `~/dev/projects/atlas/` — open this in Claude Code, not atlas-dev.

---

## Universal Bans

- Ad-hoc agents that write source files, run tests, execute bash, or produce implementation code (Explore/Plan agents are allowed per `atlas-architecture.md` — Glob + Read + Grep for tasks that need ≤ 3 searches)
- Writing code touching AST/Type/Value without checking quick-refs first (`.claude/rules/atlas-ast.md`, `atlas-typechecker.md`, `atlas-syntax.md` — pre-verified facts, no grep needed)
- Assumptions without codebase verification (grep → verify → write)
- Stub implementations, partial work, skipped edge cases

---

## Workflow Types

After GATE -1, declare one:
- **Structured Development:** Following documented plan
- **Bug Fix:** Fixing incorrect behavior
- **Refactoring:** Code cleanup (no behavior change)
- **Debugging:** Investigation, root cause
- **Enhancement:** Adding capabilities

---

## Phase Handoff

**CRITICAL:** Only hand off when ALL tests pass AND commit is made. Do NOT PR until the entire block is complete.

**Protocol:**
1. All gates passed (build, tests, clippy, fmt, security scan)
2. **Update STATUS.md** — Last Updated, Current State, Next, block table row
3. **Commit STATUS.md on the block branch** (same commit or follow-up)
4. Memory checked (GATE 7)
5. **Commit only** — no push, no PR (block-complete cadence)
6. Deliver summary

**PR flush trigger:** Block complete (final AC check phase done). Exception: blocking fix or CI issue.
**See `gates/git-workflow.md`** for batch flush commands.

**GATE V — run at two moments (see `gates/gate-versioning.md`):**
- After final block of a version plan completes → minor version check (verify ALL exit criteria, then tag)
- After a `fix/` PR that corrects a bug in an already-tagged version → patch tag check
- Does NOT run: on every fix/ PR, every block, every phase — only on version plan completion and confirmed regressions in tagged releases
- Version-to-block map is in `gates/gate-versioning.md` — that table is the contract

**Required in summary:**
- Status: "✅ PHASE COMPLETE - COMMITTED (batch)"
- Final Stats (bullets)
- **Memory:** Updated X / No updates needed (MANDATORY)
- Progress (X/~140 phases — see STATUS.md block table)
- Next phase

---

## Scaffolding Protocol (trigger: "Scaffold Block N")

1. **Read** `docs/internal/V03_PLAN.md` — block spec, ACs, dependency rules
2. **Audit blast radius** — grep every file the block will touch
3. **Produce Block Kickoff doc:**
   ```
   Block N Kickoff: {Theme}
   Files affected: [verified list]
   Architectural decisions required: [none | list with pointers]
   Risks: [what could break outside this block]
   Phase list: [title + ~5 word description each]
   ```
4. **Present kickoff doc** — this is the architect checkpoint. The user reviews the plan, not the code.
   - If trigger was "Scaffold Block N" with no further instruction: present and wait
   - If trigger was "Scaffold Block N, go" or any explicit go-ahead: skip the wait, proceed immediately
   - The architect's job is to catch wrong scope or missing decisions — NOT to verify file lists or phase details
5. **Create block branch:** `git checkout -b block/{name}` — ALL work for this block lives here
6. **Only then** scaffold all phase files
7. **Update STATUS.md** — set Current State to "Block N SCAFFOLDED", Next to Phase 1, update block table row from ⬜ to 🔨
8. **Commit scaffold + STATUS.md together — no push, no PR.** The scaffold commit is the first commit on the block branch.
   Phase execution commits follow on the same branch. PR opens only at block completion (Phase N).

**After block execution completes:**
- Verify all block ACs from the current version plan
- Update the version plan with "planned vs. actual" discoveries
- Update auto-memory with new patterns/decisions
- Run GATE V (see `gates/gate-versioning.md`)
- If more blocks remain in the plan → scaffold next block
- If all defined blocks are complete → milestone tag + surface gaps to user for architectural session (new blocks will be planned together before execution resumes)

---

## Codebase Pointers

- `crates/atlas-runtime/src/` — Runtime core (see `crates/atlas-runtime/src/CLAUDE.md`)
- `crates/atlas-lsp/src/` — LSP server (see `crates/atlas-lsp/src/CLAUDE.md`)
- `crates/atlas-jit/src/` — JIT (see `crates/atlas-jit/src/CLAUDE.md`)
- `phases/v0.3/` — Phase files by block
- `docs/specification/` — Language spec
- `docs/internal/V03_PLAN.md` — Block plan ← read before scaffolding
- auto-memory `decisions/*.md` — All locked architectural decisions
