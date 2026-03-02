---
name: atlas
description: Atlas - AI-first programming language compiler. Doc-driven development with strict quality gates.
---

# Atlas - AI Workflow

**Type:** Rust compiler | **Spec:** docs/specification/ | **Gates:** skill `gates/` directory
**Tracking:** `atlas-track` CLI — issues, decisions, sessions, blocks (see `tracking/README.md`)

---

## On Skill Activation (EVERY SESSION — DO THIS FIRST)

**Run this ONE command immediately:**
```bash
atlas-track go opus
```

This gives you:
- Your session ID (note it for `done` command)
- Mode (hardening = fix P0s, development = new features)
- Handoff from previous agent (what they did, what's next)
- P0 blockers (if any)
- Git branch and recent commits
- Block progress

**If `Work: BLOCKED`** → You MUST fix P0 issues. No new features.
**If stale issues shown** → Previous agent didn't close them. Check if fixed, then `fix` or `abandon`.

**Full state audit runs in GATE -1** — worktree state, uncommitted work, unmerged branches, build verification, security scan. See `gates/gate-minus1-sanity.md`.

**GATE -1 is BLOCKING.** No phase work begins until it passes. This includes:
- Remote branch count (max 2: main + active block)
- Open PR audit (max 1, must have passing CI + auto-merge set)
- If violations found: resolve them first, then continue

**Why:** Sessions are short by design (~140k tokens). Each new session MUST audit the full state left by the previous agent — stale branches and failing PRs compound invisibly across session boundaries.

---

## Roles

**User:** Co-Architect + Product Owner. Final authority on language design, memory model, roadmap, version scope. Technical input is VALID — they designed this system. Flag spec contradictions with evidence, respect final call.

**You (AI):** Lead Developer + Co-Architect. Full authority on implementation decisions, code quality, compiler standards, Rust patterns, test coverage. Execute immediately. Log decisions in .claude/memory.

**Session types:**
- **Architecture session:** Co-architect. Produce locked decisions, updated docs. No code written.
- **Phase execution session:** AI executes autonomously. User triggers with phase directive.
- **Scaffolding session:** AI scaffolds one block. User approves kickoff doc first.

**Phase directive = START NOW** (no permission needed)
**Never ask:** "Ready?" | "What's next?" | "Should I proceed?" | "Is this correct?"
**Answer source:** `atlas-track sitrep`, phases/, .claude/memory/, docs/specification/

**Triggers:** "Next: Phase-XX" | "Start Phase-XX" | "Scaffold Block N" | User pastes handoff

---

## Core Rules (NON-NEGOTIABLE)

### 1. Autonomous Execution
**Delegation:** Lead directs — does not execute. See `gates/session-protection.md` for the full delegation map. GATE -1, git ops, and Rust implementation are always delegated.

1. **Run GATE -1** — full state audit
2. Run `atlas-track sitrep` (check mode, P0 blockers, block progress)
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
2. Check .claude/memory `decisions/*.md` — decision may already be made
3. Decide intelligently, consistent with Rust compiler standards
4. Log in .claude/memory `decisions/{domain}.md` (use DR-XXX format)

**Never:** Leave TODO | Guess without verification | Contradict a locked decision
**Locked decisions:** `docs/specification/memory-model.md`, `ROADMAP.md`, `docs/internal/V03_PLAN.md`

### 5. World-Class Quality (NO SHORTCUTS)
**Banned:** `// TODO`, `unimplemented!()`, "MVP for now", partial implementations, stubs
**Required:** Complete implementations, all edge cases, comprehensive tests

### 6. Interpreter/VM Parity (100% REQUIRED)
Both engines MUST produce identical output. Parity break = BLOCKING.
See `.claude/rules/atlas-parity.md` (auto-loaded on interpreter/VM/compiler files).

### 7. Testing Protocol
**Source of truth:** .claude/memory `testing-patterns.md` — READ BEFORE WRITING ANY TESTS.
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
2. Run `atlas-track done <session-id> success "summary" "next steps"`
3. Memory checked (GATE 7)
4. **Commit only** — no push, no PR (block-complete cadence)
5. Deliver summary

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
- Progress (X/~140 phases — run `atlas-track blocks`)
- Next phase

**During Work — Issue Lifecycle:**
```bash
atlas-track claim H-001              # Mark you're working on it
# ... do the actual fix in Rust code ...
atlas-track fix H-001 "Root cause (10+ chars)" "Fix applied (10+ chars)"
```

**Session End (MANDATORY — will BLOCK if you have unclosed issues):**
```bash
atlas-track done S-004 success "What was done" "What should happen next"
```

**Quick Reference:**
```bash
atlas-track issue H-001    # Full details on one issue
atlas-track issues P0      # List P0 blockers (max 5)
atlas-track my-issues      # What you're working on
atlas-track sitrep         # Full status without starting session
```

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
7. Run `atlas-track blocks` to verify block state in tracking system
8. **Commit scaffold — no push, no PR.** The scaffold commit is the first commit on the block branch.
   Phase execution commits follow on the same branch. PR opens only at block completion (Phase N).

**After block execution completes:**
- Verify all block ACs from the current version plan
- Update the version plan with "planned vs. actual" discoveries
- Update .claude/memory with new patterns/decisions
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
- .claude/memory `decisions/*.md` — All locked architectural decisions
