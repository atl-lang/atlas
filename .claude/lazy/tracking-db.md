# Atlas Tracking — CLI Reference

**CLI:** `pt <command>` — all agent workflow goes through this tool.
**DB:** `tracking/atlas.db` (SQLite). Use the CLI, not raw SQL.

---

## Orientation (no session needed)

```bash
pt context        # Ultra-compact dashboard: issues/CI/block/last session
pt in-progress    # What's being worked on right now — check BEFORE claiming
```

---

## Session Lifecycle

```bash
pt go opus                              # START: init session + full sitrep
pt done S-001 success "Summary" "Next" # END: required for handoff
pt sitrep                               # Status only, no session created
```

**`go` returns:** session ID, mode, P0 blockers, stale issues, CI status, block progress, handoff from last agent.

---

## Issues

```bash
pt issues [P0|P1|P2|component]          # List open+in_progress with titles (max 5)
pt issue H-001                          # Full detail — no truncation
pt add "Title" P0|P1|P2 "problem"       # Create, returns H-XXX
pt claim H-001                          # Mark in_progress (do this first)
pt fix H-001 "root cause" "fix applied" # Close (min 10 chars each field)
pt fix-batch H-001,H-002 "c" "f"        # Close multiple, same cause/fix
pt search "keyword"                     # Search title+problem — shows priority/component/status
pt update H-001 field value             # Update: priority|component|title|problem
pt abandon H-001 "reason"               # Release in_progress → open
pt reopen H-001                         # Reopen resolved issue
pt link H-001 blocks H-002              # Link: blocks|blocked-by|related
pt links H-001                          # Show relationships
pt history H-001                        # Change history (last 10)
pt my-issues                            # Your work this session
pt components                           # Valid components + counts
```

**Issue status flow:** `open` → `in_progress` → `resolved`
**Priorities:** P0=blocker, P1=critical, P2=important, P3=nice-to-have
**Components:** parser, vm, interpreter, lsp, cli, runtime, stdlib, jit, docs, infra, formatter, typechecker

---

## Decisions (source of truth — NOT MEMORY.md)

```bash
pt decisions [component|all]            # List decisions (all = no cap, shows everything)
pt decision D-001                       # Full detail — no truncation
pt add-decision "Title" comp "Rule" "Rationale"   # Log new decision
pt update-decision D-001 rule "new text"          # Amend rule or rationale
pt update-decision D-001 rationale "new text"
pt supersede D-001 D-002                # D-001 superseded by D-002
pt deprecate D-001 "reason"             # Mark deprecated
```

**Key rule:** `decisions all` bypasses the 5-result cap — use it when you need the full picture.
**Never duplicate decisions in MEMORY.md.** Decisions live in the DB.

---

## Blocks & Phases

```bash
# Blocks
pt blocks                                    # All blocks with name + progress
pt block B8                                  # Detail + AC + inline phase list
pt block-add B11 "Title" ["AC"]              # Create new block explicitly
pt block-delete B11                          # Delete block + all its phases
pt block-update B11 name "New Title"         # Update: name|ac|blockers|notes|status
pt complete-block B8 "summary"               # Mark block complete

# Phases
pt phases B8                                 # List all phases for a block
pt phase B8-P01                              # Phase detail
pt phase-add B8 "title" ["desc"]             # Add phase (auto-creates block if missing)
pt phase-start B8-P01                        # Mark in_progress
pt phase-done B8-P01 "outcome"               # Mark done (auto-updates block count)
pt phase-skip B8-P01 "reason"               # Skip a phase
pt phase-delete B8-P01                       # Delete a phase
pt phase-update B8-P01 title "New title"     # Update: title|description|status
```

**Scaffolding a new block — correct order:**
```bash
pt block-add B11 "Block Title" "AC description"   # Create block first
pt phase-add B11 "Phase 1 title" "desc"           # Then add phases
pt phase-add B11 "Phase 2 title" "desc"
pt phases B11                                      # Verify list
```
Never use `phase-add` alone to scaffold a new block — `block-add` first ensures a named block row exists.

---

## What To Work On

```bash
pt next            # Smart triage: groups by root cause, chains, delete-first flags
```

Always run `next` before picking up issues manually.

---

## CI

```bash
pt ci-status       # Last CI run: status, failed checks, failed tests (first 20)
pt run-ci          # Trigger full suite (~10-20 min)
```

CI failures = P0 blockers. Fix before new feature work.

---

## Maintenance

```bash
pt health          # Quick DB health check
pt gc              # Close stale sessions, release orphaned in_progress issues
pt gc --aggressive # + archive old issues, vacuum DB
```

---

## Workflow Quick Reference

### Start of session
```bash
pt go sonnet
pt context         # Quick orientation if already in a session
pt in-progress     # Check what's in flight before claiming work
```

### Bug fix
```bash
pt claim H-001
# ... TDD cycle ...
pt fix H-001 "root cause" "fix applied"
```

### Architecture decision
```bash
pt decisions all   # Check all existing decisions first
pt decision D-001  # Read specific decision in full
pt add-decision "Title" component "Rule" "Rationale"
```

### End of session
```bash
pt done S-001 success "What was done" "What comes next"
```

---

## Database Tables (quick reference)

### issues
| Column | Notes |
|--------|-------|
| id | H-001 format |
| status | open, in_progress, blocked, resolved, wontfix, archived |
| priority | P0=blocker, P1=critical, P2=important, P3=nice-to-have |
| component | parser, vm, interpreter, lsp, cli, runtime, stdlib, jit, docs, infra, formatter, typechecker |
| problem | Detailed description |
| fix_required | What needs to happen |
| root_cause | Why it happened (populated on close) |
| fix_applied | What was done (populated on close) |
| fixed_by | Session ID that closed it |

### decisions
| Column | Notes |
|--------|-------|
| id | D-001 format |
| status | active, superseded, deprecated |
| rule | The decision itself |
| rationale | Why |

### sessions
| Column | Notes |
|--------|-------|
| id | S-001 format |
| agent | opus, sonnet, haiku, human |
| outcome | success, partial, blocked, failed, abandoned |
| summary | What was accomplished |
| next_steps | Handoff to next agent |

### blocks
| Column | Notes |
|--------|-------|
| block_num | 1, 2, 3… (B1, B2 in shorthand) |
| name | Block name |
| status | pending, scaffolded, in_progress, complete, blocked |
| phases_done / phases_total | Progress |
| acceptance_criteria | JSON array of AC items |
