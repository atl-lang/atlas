# Atlas Tracking — CLI Reference

**CLI:** `atlas-track <command>` — all agent workflow goes through this tool.
**DB:** `tracking/atlas.db` (SQLite). Use the CLI, not raw SQL.

---

## Orientation (no session needed)

```bash
atlas-track context        # Ultra-compact dashboard: issues/CI/block/last session
atlas-track in-progress    # What's being worked on right now — check BEFORE claiming
```

---

## Session Lifecycle

```bash
atlas-track go opus                              # START: init session + full sitrep
atlas-track done S-001 success "Summary" "Next" # END: required for handoff
atlas-track sitrep                               # Status only, no session created
```

**`go` returns:** session ID, mode, P0 blockers, stale issues, CI status, block progress, handoff from last agent.

---

## Issues

```bash
atlas-track issues [P0|P1|P2|component]          # List open+in_progress with titles (max 5)
atlas-track issue H-001                          # Full detail — no truncation
atlas-track add "Title" P0|P1|P2 "problem"       # Create, returns H-XXX
atlas-track claim H-001                          # Mark in_progress (do this first)
atlas-track fix H-001 "root cause" "fix applied" # Close (min 10 chars each field)
atlas-track fix-batch H-001,H-002 "c" "f"        # Close multiple, same cause/fix
atlas-track search "keyword"                     # Search title+problem — shows priority/component/status
atlas-track update H-001 field value             # Update: priority|component|title|problem
atlas-track abandon H-001 "reason"               # Release in_progress → open
atlas-track reopen H-001                         # Reopen resolved issue
atlas-track link H-001 blocks H-002              # Link: blocks|blocked-by|related
atlas-track links H-001                          # Show relationships
atlas-track history H-001                        # Change history (last 10)
atlas-track my-issues                            # Your work this session
atlas-track components                           # Valid components + counts
```

**Issue status flow:** `open` → `in_progress` → `resolved`
**Priorities:** P0=blocker, P1=critical, P2=important, P3=nice-to-have
**Components:** parser, vm, interpreter, lsp, cli, runtime, stdlib, jit, docs, infra, formatter, typechecker

---

## Decisions (source of truth — NOT MEMORY.md)

```bash
atlas-track decisions [component|all]            # List decisions (all = no cap, shows everything)
atlas-track decision D-001                       # Full detail — no truncation
atlas-track add-decision "Title" comp "Rule" "Rationale"   # Log new decision
atlas-track update-decision D-001 rule "new text"          # Amend rule or rationale
atlas-track update-decision D-001 rationale "new text"
atlas-track supersede D-001 D-002                # D-001 superseded by D-002
atlas-track deprecate D-001 "reason"             # Mark deprecated
```

**Key rule:** `decisions all` bypasses the 5-result cap — use it when you need the full picture.
**Never duplicate decisions in MEMORY.md.** Decisions live in the DB.

---

## Blocks

```bash
atlas-track blocks          # All blocks with name + progress
atlas-track block 8         # Block detail + acceptance criteria (also: block B8)
```

---

## What To Work On

```bash
atlas-track next            # Smart triage: groups by root cause, chains, delete-first flags
```

Always run `next` before picking up issues manually.

---

## CI

```bash
atlas-track ci-status       # Last CI run: status, failed checks, failed tests (first 20)
atlas-track run-ci          # Trigger full suite (~10-20 min)
```

CI failures = P0 blockers. Fix before new feature work.

---

## Maintenance

```bash
atlas-track health          # Quick DB health check
atlas-track gc              # Close stale sessions, release orphaned in_progress issues
atlas-track gc --aggressive # + archive old issues, vacuum DB
```

---

## Workflow Quick Reference

### Start of session
```bash
atlas-track go sonnet
atlas-track context         # Quick orientation if already in a session
atlas-track in-progress     # Check what's in flight before claiming work
```

### Bug fix
```bash
atlas-track claim H-001
# ... TDD cycle ...
atlas-track fix H-001 "root cause" "fix applied"
```

### Architecture decision
```bash
atlas-track decisions all   # Check all existing decisions first
atlas-track decision D-001  # Read specific decision in full
atlas-track add-decision "Title" component "Rule" "Rationale"
```

### End of session
```bash
atlas-track done S-001 success "What was done" "What comes next"
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
