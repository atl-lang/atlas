# Atlas Tracking System

**CLI:** `pt` — the ONLY approved interface
**Direct SQL is BANNED** — protects against token-burning unbounded queries

## Token Safety (Enforced)

| Query Type | Max Output | Example |
|------------|------------|---------|
| Status | 3 lines | `pt status` |
| Index | 5 rows, IDs only | `pt issues` |
| Detail | 1 item, ~6 lines | `pt issue H-001` |

**Never possible:**
- Dump all issues with full text
- List 20 decisions at once
- Get unbounded query results

## Quick Start

```bash
# Every session starts with:
pt go opus              # THE command: init + full sitrep

# Check what needs work:
pt issues P0

# Get details on one issue:
pt issue H-001

# File a new issue:
pt add "Title" P1 "Problem description"

# When you fix something:
pt claim H-001          # Mark in_progress
pt fix H-001 "Root cause (10+ chars)" "Fix applied (10+ chars)"

# Before handoff:
pt done S-002 success "What was done" "Next steps"
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
  pt go opus             # THE command: init + full sitrep
  pt sitrep              # Status only (no session start)

ISSUES - CRUD:
  pt add "Title" P0|P1|P2 "problem"  # Create issue
  pt update H-001 priority P0        # Update field (priority|component|title|problem)
  pt claim H-001         # Mark in_progress
  pt fix H-001 "cause" "fix"         # Close issue
  pt fix-batch H-001,H-002 "cause" "fix"  # Close multiple
  pt abandon H-001       # Release back to open
  pt reopen H-001        # Reopen closed issue

ISSUES - QUERY:
  pt search "keyword"    # Find issues by keyword (max 10)
  pt my-issues           # Your work this session
  pt components          # List valid components + counts

ISSUES - LINKING:
  pt link H-001 blocks H-002      # H-001 blocks H-002
  pt link H-001 blocked-by H-002  # H-001 is blocked by H-002
  pt link H-001 related H-002     # Related issues
  pt links H-001                  # Show all relationships

ISSUES - HISTORY:
  pt history H-001       # Show change history for issue

INDEX (IDs only, max 5):
  pt issues              # Open issues
  pt issues P0           # P0 only
  pt issues runtime      # By component
  pt decisions           # Active decisions
  pt blocks              # Block progress

DETAIL (single item):
  pt issue H-001         # Full issue details
  pt decision D-001      # Full decision details

END:
  pt done S-001 success "summary" "next"  # End session

DECISIONS:
  pt add-decision "Title" component "Rule" "Rationale"

MODE:
  pt unblock             # Allow block work
  pt block-work          # Enter hardening mode

MAINTENANCE:
  pt gc                  # Clean up stale sessions
  pt health              # Quick status check
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
