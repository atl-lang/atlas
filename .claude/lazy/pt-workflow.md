# pt Workflow Guide

> Detailed reference for the gates defined in CLAUDE.md.
> The gates themselves live in CLAUDE.md — this file explains the *how* and *why*.

---

## Session Lifecycle (Full Detail)

### Session Start
```bash
pt go          # Full sitrep: session ID, mode, Next Action, P0s, CI status, block progress (model auto-detected)
pt in-progress      # What's already claimed — avoid duplicate effort
```
`pt go` shows `── Next Action (from last session) ──` automatically — read it first, that's your starting point.
If `Work: BLOCKED` → fix P0s first. No new features.
Active plans: `pt plans` — shows open PL-XXX outcomes from brainstorm sessions.
**Mid-session status check:** use `pt context` — read-only, no session side effects. Never call `pt go` mid-session.

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

### Session Close
No handoff file. Context is stored in the DB and surfaced automatically in `pt go`.

```bash
pt done S-XXX success \
  "Fixed H-001 (cause → fix). Implemented Phase-04 (what was wired)." \
  "claim H-002 — fix Y in crates/atlas-runtime/src/eval.rs, grep for fn eval_expr"
```

- **Arg 3 (summary):** backward-looking — what was done this session. One clause per issue/phase, root cause + fix.
- **Arg 4 (next):** forward-looking — what the next agent should do first. Specific enough to act on without reading anything else: issue ID + action + file/function if known.

The next agent sees this as `── Next Action (from last session) ──` in `pt go`, before P0 blockers. No extra steps needed.

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

