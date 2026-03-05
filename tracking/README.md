# Atlas Tracking System

**CLI:** `atlas-track` — the ONLY approved interface
**Direct SQL is BANNED** — protects against token-burning unbounded queries

## Token Safety (Enforced)

| Query Type | Max Output | Example |
|------------|------------|---------|
| Status | 3 lines | `atlas-track status` |
| Index | 5 rows, IDs only | `atlas-track issues` |
| Detail | 1 item, ~6 lines | `atlas-track issue H-001` |

**Never possible:**
- Dump all issues with full text
- List 20 decisions at once
- Get unbounded query results

## Quick Start

```bash
# Every session starts with:
atlas-track go opus              # THE command: init + full sitrep

# Check what needs work:
atlas-track issues P0

# Get details on one issue:
atlas-track issue H-001

# File a new issue:
atlas-track add "Title" P1 "Problem description"

# When you fix something:
atlas-track claim H-001          # Mark in_progress
atlas-track fix H-001 "Root cause (10+ chars)" "Fix applied (10+ chars)"

# Before handoff:
atlas-track done S-002 success "What was done" "Next steps"
```

## Bulk Operations

For bulk issue imports (e.g., after audits), use SQLite directly:
```bash
sqlite3 tracking/atlas.db << 'SQL'
INSERT INTO issues (id, title, status, priority, severity, component, version, source, problem)
VALUES
  ('H-071', 'Issue 1', 'open', 'P1', 'high', 'parser', '0.3.0', 'audit', 'Description'),
  ('H-072', 'Issue 2', 'open', 'P2', 'medium', 'runtime', '0.3.0', 'audit', 'Description');
SQL
```
Use CLI for single issues (better UX). Use SQL for 5+ issues (efficiency).

## Components (Compiler Domains)

`parser`, `binder`, `typechecker`, `interpreter`, `vm`, `codegen`, `jit`, `runtime`, `stdlib`, `lsp`, `cli`, `infra`, `docs`

## Priority Levels

| Priority | Meaning | Block Work? |
|----------|---------|-------------|
| P0 | Blocker | Yes — no new features until resolved |
| P1 | Critical | No — but must fix this version |
| P2 | Important | No — should fix this version |
| P3 | Nice-to-have | No — can defer |

## Modes

| Mode | Block Work | Description |
|------|------------|-------------|
| `hardening` | BLOCKED | Fix P0 issues first |
| `development` | Allowed | Normal feature work |
| `release` | BLOCKED | Prepare for version tag |

## CLI Commands

```
START:
  atlas-track go opus             # THE command: init + full sitrep
  atlas-track sitrep              # Status only (no session start)

ISSUES:
  atlas-track add "Title" P0|P1|P2 "problem"  # Create issue
  atlas-track claim H-001         # Mark in_progress
  atlas-track fix H-001 "cause" "fix"         # Close issue
  atlas-track abandon H-001       # Release back to open
  atlas-track reopen H-001        # Reopen closed issue
  atlas-track my-issues           # Your work this session

INDEX (IDs only, max 5):
  atlas-track issues              # Open issues
  atlas-track issues P0           # P0 only
  atlas-track issues runtime      # By component
  atlas-track decisions           # Active decisions
  atlas-track blocks              # Block progress

DETAIL (single item):
  atlas-track issue H-001         # Full issue details
  atlas-track decision D-001      # Full decision details

END:
  atlas-track done S-001 success "summary" "next"  # End session

DECISIONS:
  atlas-track add-decision "Title" component "Rule" "Rationale"

MODE:
  atlas-track unblock             # Allow block work
  atlas-track block-work          # Enter hardening mode

MAINTENANCE:
  atlas-track gc                  # Clean up stale sessions
  atlas-track health              # Quick status check
```

## Database Schema

The SQLite database (`atlas.db`) contains:
- `state` — current mode, block work allowed, version
- `issues` — bugs/tasks with component linkage
- `decisions` — architectural decisions (permanent record)
- `sessions` — agent continuity tracking
- `blocks` — phase progress per version
- `versions` — release lifecycle
- `archives` — completed version exports

**Agents interact via CLI only.** The schema is for reference, not direct access.
