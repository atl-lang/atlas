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
atlas-track status
atlas-track start-session opus  # Returns your session ID

# Check what needs work:
atlas-track issues P0

# Get details on one issue:
atlas-track issue H-001

# When you fix something:
atlas-track close-issue H-001 S-002 "Root cause" "Fix applied"

# Before handoff:
atlas-track end-session S-002 success "What was done" "Next steps"
```

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
STARTUP:
  atlas-track status              # 3-line status (mode, P0s, last session)

INDEX (IDs only, max 5):
  atlas-track issues              # Open issues
  atlas-track issues P0           # P0 only
  atlas-track issues runtime      # By component
  atlas-track decisions           # Active decisions
  atlas-track blocks              # Block progress
  atlas-track sessions            # Recent sessions

DETAIL (single item):
  atlas-track issue H-001         # Full issue details
  atlas-track decision D-001      # Full decision details
  atlas-track session S-001       # Full session details
  atlas-track block 7             # Full block details

MUTATIONS:
  atlas-track start-session opus              # Start session
  atlas-track end-session ID outcome "summary" "next"
  atlas-track close-issue ID session "cause" "fix"
  atlas-track open-issue "title" P1 high comp "problem" "fix"
  atlas-track unblock                         # Allow block work
  atlas-track block-work                      # Enter hardening mode
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
