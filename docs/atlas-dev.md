# Atlas Dev CLI - AI Reference

**AI-only, token-optimized tracking. Database is truth.**

---

## Command Index

**Phase Management:**
- `phase count [-s STATUS] [-c CAT]` - Count phases (20 bytes)
- `phase list [-s STATUS] [-c CAT] [--limit N]` - List phases (minimal, default limit 10)
- `phase info <path>` - Full phase details
- `phase current` - Last completed phase
- `phase next [-c CAT]` - Next pending phase
- `phase complete <path> -d "desc" --tests N [-c]` - Complete phase, optional commit

**Context (PRIMARY - Use This First):**
- `context current` - **START HERE** - Next phase + deps + decisions + progress (single command)
- `context phase <path>` - Full context for specific phase

**Decisions:**
- `decision count [-c COMP] [-s STATUS]` - Count decisions (20 bytes)
- `decision list [-c COMP] [-s STATUS] [--limit N]` - List decisions (minimal, limit 10)
- `decision read <id>` - Full decision details
- `decision search "keyword"` - Full-text search
- `decision create -c COMP -t "title" --decision "..." --rationale "..."` - Log decision

**Features:**
- `feature count [-s STATUS]` - Count features
- `feature list [-s STATUS] [--limit N]` - List features (minimal, limit 10)
- `feature read <name>` - Full feature details
- `feature sync <name>` - Sync from codebase

**Analytics:**
- `summary` - Project dashboard (categories, progress, recent)
- `blockers` - Blocked phases
- `stats` - Velocity/estimates
- `timeline` - Completion history
- `test-coverage` - Test stats by category

**Validation:**
- `validate` - DB integrity
- `validate parity` - Parity checks
- `validate tests` - Test coverage
- `validate all` - All checks

---

## Surgical Query Pattern (ALWAYS USE)

**Never list everything. Always: count → list → read**

```bash
# 1. COUNT: How many? (20 bytes)
atlas-dev phase count -s pending
# {"ok":true,"cnt":21}

# 2. LIST: Which ones? (minimal fields, limit 10, ~400 bytes)
atlas-dev phase list -s pending --limit 5
# {"ok":true,"phases":[{"path":"...","cat":"...","sts":"..."}],"cnt":5}

# 3. READ: Full details (only when needed)
atlas-dev phase info phases/stdlib/phase-07b.md
# Full phase object
```

**Token Savings:**
- OLD (list all): 6,026 bytes
- NEW (count): 21 bytes = **99.6% savings**
- NEW (list 10): 1,128 bytes = **81% savings**

---

## Key Workflows

### Start Work (ALWAYS START HERE)
```bash
# Single command - everything you need
atlas-dev context current
# Returns: next phase, dependencies, blockers, decisions, progress
```

### Complete Phase
```bash
atlas-dev phase complete <path> \
  -d "Brief description" \
  --tests N \
  --commit
```

### Check Dependencies
```bash
atlas-dev context current           # Deps + blockers in context
atlas-dev blockers                  # See all blocked phases
atlas-dev phase info <path>         # Specific phase status
```

### Find Decisions
```bash
atlas-dev decision count -c stdlib  # How many?
atlas-dev decision search "hash"    # Full-text search
atlas-dev decision read DR-001      # Full details
```

### Check Progress
```bash
atlas-dev summary                   # Dashboard
atlas-dev stats                     # Velocity
atlas-dev phase count -s completed  # Total done
```

---

## Piping Workflows

**Auto-detects JSON stdin (no --stdin flag):**

```bash
# List → Read
atlas-dev phase list -s pending | atlas-dev phase info

# Search → Read
atlas-dev decision search "hash" | atlas-dev decision read

# List → Validate
atlas-dev feature list | atlas-dev feature validate

# With jq filtering
atlas-dev phase list -c stdlib | jq -r '.phases[].path'
```

---

## Command Patterns

**Always prefer surgical commands:**

| Goal | ❌ Wasteful | ✅ Surgical |
|------|------------|-----------|
| Count phases | `phase list` (6KB) | `phase count` (21 bytes) |
| Check if complete | `phase list` + grep | `phase info <path>` |
| Find decisions | `decision list` (all) | `decision search "keyword"` |
| Get context | Read STATUS.md | `context current` |
| See blockers | `phase list` + parse | `blockers` |

---

## Compact JSON (All Outputs)

**Abbreviated fields (76% token reduction):**
- `ok` - Success boolean
- `cnt` - Count
- `lim` - Limit
- `sts` - Status
- `cat` - Category
- `comp` - Component
- `desc` - Description
- `pct` - Percentage
- `prog` - Progress

---

## Default Limits (Token Protection)

**All list commands default to 10 results:**
- `phase list` → 10 phases (not all 85!)
- `decision list` → 10 decisions
- `feature list` → 10 features

**Override:** `--limit N` (max 100)

---

## Critical Rules

1. **Start with `context current`** - Single command gives everything
2. **Use count before list** - Know quantity first (99.6% savings)
3. **Use list with limits** - Default 10, never fetch all
4. **Use read only when needed** - Full details on demand
5. **Pipe for composition** - Chain commands efficiently
6. **Never read STATUS.md** - Database is truth

---

## Common Operations

**Starting phase:**
```bash
atlas-dev context current
```

**Checking blockers:**
```bash
atlas-dev blockers
```

**Completing work:**
```bash
atlas-dev phase complete <path> -d "..." --tests N -c
```

**Finding related decisions:**
```bash
atlas-dev decision search "relevant-keyword"
```

**Progress check:**
```bash
atlas-dev summary
```

---

## Token Cost Reference

| Operation | Tokens | Notes |
|-----------|--------|-------|
| `context current` | ~800 | Full phase context (best value) |
| `summary` | ~900 | Full dashboard |
| `phase count` | ~21 | Just count |
| `phase list` (10) | ~1,128 | Minimal fields, limit 10 |
| `phase info` | ~600 | Full phase details |
| `blockers` | ~400 | Blocked phases only |
| `decision search` | ~300-600 | Depends on matches |

**Full docs:** `tools/atlas-dev/README.md` (human-focused, verbose, 1093 lines - use only when needed)
