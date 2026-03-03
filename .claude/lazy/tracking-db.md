# Atlas Tracking Database

**Location:** `tracking/atlas.db` (SQLite)
**CLI:** `atlas-track` — see `tracking/README.md` for commands

---

## Quick Queries

```bash
# List open issues by priority
sqlite3 tracking/atlas.db "SELECT id, title, priority FROM issues WHERE status = 'open' ORDER BY priority, id"

# Get issue details
sqlite3 tracking/atlas.db "SELECT * FROM issues WHERE id = 'H-001'"

# List sessions
sqlite3 tracking/atlas.db "SELECT id, agent, started_at, outcome FROM sessions ORDER BY started_at DESC LIMIT 10"

# Check current mode
sqlite3 tracking/atlas.db "SELECT mode, block_work_allowed FROM state WHERE id = 1"
```

---

## Tables

### issues
| Column | Type | Notes |
|--------|------|-------|
| id | TEXT PK | H-001, P0-001 (H=hardening) |
| title | TEXT | Short description |
| status | TEXT | open, in_progress, blocked, resolved, wontfix, archived |
| priority | TEXT | P0=blocker, P1=critical, P2=important, P3=nice-to-have |
| severity | TEXT | critical, high, medium, low |
| component | TEXT | parser, vm, interpreter, lsp, cli, runtime, stdlib, jit, docs, infra |
| version | TEXT | v0.2, v0.3 |
| source | TEXT | battle-test, ci, audit, user, agent, fuzz |
| problem | TEXT | Detailed description |
| fix_required | TEXT | What needs to happen |
| fix_applied | TEXT | What was done (after fix) |
| root_cause | TEXT | Why it happened |
| files | TEXT | Comma-separated paths |
| tags | TEXT | Comma-separated: async, parity, security |

### sessions
| Column | Type | Notes |
|--------|------|-------|
| id | TEXT PK | S-001, S-002 |
| agent | TEXT | opus, sonnet, haiku, human |
| started_at | TEXT | datetime |
| ended_at | TEXT | datetime |
| mode | TEXT | development, hardening, release, research |
| outcome | TEXT | success, partial, blocked, failed, abandoned |
| summary | TEXT | What was accomplished |
| next_steps | TEXT | Handoff to next agent |
| issues_opened | TEXT | Comma-separated IDs |
| issues_closed | TEXT | Comma-separated IDs |

### state
| Column | Type | Notes |
|--------|------|-------|
| id | INTEGER PK | Always 1 (singleton) |
| mode | TEXT | development, hardening, release |
| block_work_allowed | INTEGER | 0=fix P0s first, 1=can do blocks |
| current_block | INTEGER | Active block number |
| last_session_id | TEXT | FK to sessions |

### blocks
| Column | Type | Notes |
|--------|------|-------|
| version | TEXT | v0.3 |
| block_num | INTEGER | 1, 2, 3... |
| name | TEXT | Block name |
| status | TEXT | pending, scaffolded, in_progress, complete, blocked |
| phases_total | INTEGER | Total phases |
| phases_done | INTEGER | Completed phases |

### decisions
| Column | Type | Notes |
|--------|------|-------|
| id | TEXT PK | D-001, D-002 |
| title | TEXT | Decision title |
| status | TEXT | active, superseded, deprecated, proposed |
| component | TEXT | Domain |
| rule | TEXT | The actual decision |
| rationale | TEXT | Why |

---

## Insert Examples

```sql
-- New issue
INSERT INTO issues (id, title, status, priority, severity, component, version, source, problem, fix_required, files, tags)
VALUES ('H-018', 'Title here', 'open', 'P1', 'medium', 'vm', 'v0.2', 'audit', 'Problem description', 'Fix description', 'path/to/file.rs', 'tag1,tag2');

-- Update issue status
UPDATE issues SET status = 'resolved', fix_applied = 'What was done' WHERE id = 'H-001';

-- Close session
UPDATE sessions SET ended_at = datetime('now'), outcome = 'success', summary = 'What was done', next_steps = 'What is next' WHERE id = 'S-008';
```

---

## CLI Commands (preferred)

```bash
atlas-track go opus          # Start session, get sitrep
atlas-track sitrep           # Status without starting session
atlas-track issues P0        # List P0 issues
atlas-track issue H-001      # Show issue details
atlas-track claim H-001      # Mark working on issue
atlas-track fix H-001 "root cause" "fix applied"  # Close issue
atlas-track done S-008 success "summary" "next steps"  # End session
atlas-track blocks           # Show block progress
```
