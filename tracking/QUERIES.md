# Atlas Tracking — Agent Protocol

**CLI:** `atlas-track` — the ONLY approved interface
**Direct SQL is BANNED** — no `sqlite3` commands, ever

## Token Budget Rules (MANDATORY)

1. **Index queries return max 5 rows** — IDs only, no full data
2. **Detail queries are single-item only** — one issue, one decision, one session
3. **Never dump multiple full records** — if you need 3 issues, run 3 detail queries
4. **Use compound commands** — `init`, `done`, `fix` do multiple ops in one call

## Session Startup (ONE COMMAND)

```bash
atlas-track init opus
```

Output:
```
Mode: hardening | Block work: BLOCKED | Current: B6
P0: H-001,H-002,H-003,H-004,H-011,H-012
Last: Previous session summary...
Session: S-004
---
H-002 P0 codegen
H-012 P0 interpreter
H-003 P0 jit
H-001 P0 runtime
H-011 P0 runtime
(+1 more, use filter)
```

**If block work is BLOCKED:** You MUST work on P0 issues. No new features.

## Index Queries (IDs only, max 5 rows)

```bash
atlas-track issues          # All open issues (IDs only)
atlas-track issues P0       # P0 blockers only
atlas-track issues runtime  # Issues for one component
atlas-track decisions       # Active decisions
atlas-track blocks          # Block progress
atlas-track sessions        # Recent sessions
```

## Detail Queries (single item, when you need specifics)

```bash
atlas-track issue H-001     # One issue's full details
atlas-track decision D-001  # One decision's full details
atlas-track session S-001   # One session's full details
atlas-track block 7         # One block's full details
```

## Session Management (use compound commands)

```bash
# Start session (use init instead — does status + start + P0 list)
atlas-track init opus

# End session (use done instead — does end + final status)
atlas-track done S-004 success "What was done" "What should happen next"
```

Outcome values: `success`, `partial`, `blocked`, `failed`, `abandoned`

## Issue Management (use compound commands)

```bash
# Close an issue you fixed (use fix — uses current session, shows remaining P0s)
atlas-track fix H-001 "Root cause" "Fix applied"

# Open a new issue you discovered
atlas-track open-issue "New bug title" P1 high runtime "What's wrong" "What needs to happen"
```

## Mode Control

```bash
# Allow new feature work (all P0s resolved)
atlas-track unblock

# Block new work (hardening mode)
atlas-track block-work
```

## Component Values

`parser`, `binder`, `typechecker`, `interpreter`, `vm`, `codegen`, `jit`, `runtime`, `stdlib`, `lsp`, `cli`, `infra`, `docs`

## Priority Values

- `P0` — Blocker, stops all new work
- `P1` — Critical, must fix this version
- `P2` — Important, should fix this version
- `P3` — Nice to have, can defer

## What Agents MUST Do

1. **Start:** `atlas-track init opus` — status + session + P0 list in one call
2. **Claim:** `atlas-track claim H-XXX` before working on an issue
3. **Fix:** `atlas-track fix H-XXX "root cause" "fix applied"` — meaningful descriptions required
4. **Check:** `atlas-track my-issues` — see your in-progress and closed issues
5. **End:** `atlas-track done S-XXX outcome "summary" "next"` — BLOCKED if unclosed issues
6. **Never:** Raw sqlite3, skip session tracking, leave issues in_progress

## Decision Management

```bash
# List active decisions
atlas-track decisions

# List by component
atlas-track decisions runtime

# List all (including superseded/deprecated)
atlas-track decisions all

# View one decision
atlas-track decision D-001

# Add new decision
atlas-track add-decision "Title" component "Rule text" "Rationale"

# Supersede (D-001 replaced by D-002)
atlas-track supersede D-001 D-002

# Deprecate (no replacement)
atlas-track deprecate D-001 "reason"
```

Decision statuses: `active`, `superseded`, `deprecated`, `proposed`
