# pt Workflow Guide

> Detailed reference for the gates defined in CLAUDE.md.
> The gates themselves live in CLAUDE.md — this file explains the *how* and *why*.

---

## Session Lifecycle (Full Detail)

### Session Start
```bash
pt go opus          # Full sitrep: session ID, mode, P0s, CI status, block progress, handoff
pt in-progress      # What's already claimed — avoid duplicate effort
```
If handoff shows `~/.project-tracker/handoffs/atlas-handoff.md` → **read it before anything else**.
If `Work: BLOCKED` → fix P0s first. No new features.
Active plans: `pt plans` — shows open PL-XXX outcomes from brainstorm sessions.
Quick mid-session orientation (no session overhead): `pt context`

### Picking Work
```bash
pt next             # Groups by root cause, shows chains, delete-first/triage-first flags
pt issues [P0|P1|component]   # Raw list if you want to browse
pt issue H-XXX      # Full detail on a specific issue
```
**Always use `pt next` before picking.** Blind P0→P1 sorting misses chains where fixing H-001 closes H-002+H-003. `pt next` surfaces these.

### Issue Lifecycle
```bash
pt claim H-XXX                                        # Lock it — signals in-progress to other agents
# ... implement, verify ...
pt fix H-XXX "Root cause (specific)" "Fix (specific)" # Close immediately after verification
git commit -m "fix(...): description"
pt note S-XXX "fixed H-XXX: <one-line root cause + fix>"  # ALWAYS — keeps session data alive
# NOW move to next issue
```
**Close immediately, never batch at session end.** Batched closures lose root cause specificity.

Mid-issue corrections:
```bash
pt update H-XXX priority P0         # Escalate
pt update H-XXX title "new title"   # Correct title
pt abandon H-XXX "reason"           # Release claim if blocked
pt reopen H-XXX                     # If regression found
pt link H-001 blocks H-002          # Document dependency
```

### Decision Gate
```bash
pt decisions <component>            # Before any architecture/design work
pt decisions all                    # Full dump — D-001 through D-030+
pt decision D-XXX                   # Full detail on one decision
```
**Component map:**
| What you're changing | Component |
|---------------------|-----------|
| Parser, grammar, syntax | `parser` |
| Type inference, type checking | `typechecker` |
| Interpreter eval loop | `interpreter` |
| VM bytecode, opcodes | `vm` |
| Stdlib functions, builtins | `stdlib` |
| Runtime core, values, memory | `runtime` |
| LSP server | `lsp` |
| Test structure, CI, tooling | `infra` |

Amending a decision:
```bash
pt update-decision D-XXX rule "corrected rule text"
pt update-decision D-XXX rationale "updated rationale"
pt supersede D-XXX D-YYY            # Mark D-XXX superseded by D-YYY
```

### Block Tracking
```bash
pt blocks                           # All blocks + status
pt block B<N>                       # Detail + acceptance criteria
pt phase-start B<N>-P<XX>           # Mark phase in_progress (optional but good hygiene)
pt phase-done B<N>-P<XX> "outcome"  # After EVERY phase commit — auto-updates block count AND phases_completed
# phases_completed + git_commits tracked automatically — no extra steps needed
pt block B<N>                       # After final phase — verify ALL AC met + see full phase list
pt complete-block B<N> "summary"    # Mark block complete
pt phase-skip B<N>-P<XX> "reason"   # If a phase is intentionally skipped
```
Skipping `pt phase-done` = next agent re-derives block state from scratch. Don't.

**When scaffolding a new block**, always create the block first, then phases:
```bash
pt block-add B<N> "Block Title" "Acceptance criteria"    # CREATE BLOCK FIRST
pt phase-add B<N> "Phase title" "optional description"   # repeat for each phase
pt phases B<N>                                            # verify the list
```

**Block + Phase management commands:**
```bash
pt block-add B11 "Title" ["AC"]          # Create block (required before phase-add on new blocks)
pt block-delete B11                      # Delete block + all its phases
pt block-update B11 name "New Title"     # Update: name|ac|blockers|notes|status
pt phase-delete B8-P01                   # Delete a phase (recounts block totals)
pt phase-update B8-P01 title "New"       # Update: title|description|status
```

### Handoff File (MANDATORY before pt done)
Overwrite `~/.project-tracker/handoffs/atlas-handoff.md` — REPLACE entirely, don't append:
```markdown
# Atlas Handoff — <session-id>

**Updated:** <ISO timestamp> | **Commit:** <git sha> | **Agent:** <model>

## What Was Done This Session
<one sentence per issue/phase — specific: IDs, files, decisions. No bullet dumps.>

## Current State
<P0/P1/P2 counts, CI status, active block, any in-progress claims>

## In-Flight Work
<NONE — or: what was started but not finished, exact state, next concrete step>

## Next Action
<specific enough for cold-start: issue ID + what to do + file/function path>

## Open Questions (Needs Architect Input)
<NONE — or: decisions that need the architect, not implementation choices>

## Critical Context (Don't Lose This)
<anything a future agent would discover the hard way: patterns, pitfalls, non-obvious state>
```
`git add ~/.project-tracker/handoffs/atlas-handoff.md` — include in final commit or own `chore: update handoff` commit.

### Session Close
```bash
pt done S-XXX success \
  "Fixed H-001 (cause → fix). Implemented Phase-04 (what was wired)." \
  "Next: Phase-05 — Value::Future in runtime, file: crates/atlas-runtime/src/eval.rs"
```
Format: one clause per issue/phase. Root cause + fix. Next: specific enough for cold-start.

---

## CI Workflow
```bash
pt ci-status        # Last CI run — shows failed tests by name, not just pass/fail
pt run-ci           # Trigger full nightly suite on-demand
pt mark-ci-pass "reason"  # After resolving CI failures — never leave stale FAIL in sitrep
```
CI failures = P0 blocker. Fix before new work.

---

## Doc Drift (after every commit touching source)
Stop hook shows **DOC DRIFT ALERT** → fire `atlas-doc-patch` agent (Haiku, scoped, ~1-2 min).
Stop hook shows **DOC EDITS UNCOMMITTED** → atlas-doc-patch already ran. `git add docs/ && git commit`.
Neither blocks you — handle at natural stopping point. Never leave either alert across sessions.

---

## Brainstorm Outcomes (atlas-brainstorm skill)
```bash
pt add-decision "Title" <component> "Rule: ..." "Rationale: ..."   # If decision reached
pt add "title" P0|P1|P2 "context"                                   # If work identified
pt add "Open question: X" P2 "what's unclear, what info needed"    # If still open
pt plan add "Title" "Approach: ..." "H-XXX" "D-XXX"                # If concrete plan decided
```
No brainstorm ends with "let's think about this more" as the only output. That's lost.

---

## Maintenance
```bash
pt health           # DB health check
pt gc               # Garbage collect stale records
pt gc --aggressive  # Full GC
```

---

## Handoff File Location
`~/.project-tracker/handoffs/atlas-handoff.md` — NOT `.atlas-handoff.md` in project root.
Centralized for GUI monitoring. Always this path.
