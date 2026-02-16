# AI Optimization Guide
## Making atlas-dev Perfect for LLM Agents

**Core Principle:** This tool is 100% for AI agents. Every byte counts.

---

## IMPORTANT: Pure SQLite Architecture

**One-time migration, then SQL forever:**
1. ✅ **Bootstrap once** - `atlas-dev migrate bootstrap` (parses STATUS.md → SQLite)
2. ✅ **Delete markdown** - Remove STATUS.md, trackers/*.md (no longer needed)
3. ✅ **SQL only** - All future updates via CLI → database
4. ❌ **No markdown indexing** - Database is indexed, no file parsing needed

**After migration:**
- All tracking data in `atlas-dev.db`
- Phase files (`phases/*.md`) are instructions only (never change)
- No STATUS.md, no trackers, no decision-logs markdown
- All operations via SQL (indexed, < 1ms)

**This guide assumes pure SQLite architecture (post-migration).**

---

## Token Efficiency

### Problem: Verbose Output Wastes Tokens

**Example - Human-friendly (BAD for AI):**
```
✅ Phase Successfully Completed!

Phase: phase-07b-hashset.md
Category: Standard Library (stdlib)

Progress Update:
  Category Progress: 10 out of 21 phases complete (48%)
  Overall Progress: 31 out of 78 phases complete (40%)

Files Modified:
  ✅ status/trackers/1-stdlib.md (updated phase status)
  ✅ STATUS.md (updated 5 fields)

Validation: PASSED ✓
  - Sync check: tracker count matches STATUS.md ✓
  - Percentage calculations: correct ✓

Next Phase: phases/stdlib/phase-07c-queue-stack.md
  Description: Queue (FIFO) + Stack (LIFO), ~690 lines, 36+ tests

Git Commit: a1b2c3d4e5f6
  Message: "Mark phase-07b-hashset.md complete (31/78)"
```

**Token count: ~180 tokens**

---

**AI-optimized (GOOD):**
```json
{"ok":true,"phase":"phase-07b-hashset.md","cat":"stdlib","progress":{"cat":[10,21,48],"total":[31,78,40]},"next":"phase-07c-queue-stack.md","mod":["status/trackers/1-stdlib.md","STATUS.md"],"commit":"a1b2c3d"}
```

**Token count: ~80 tokens**

**Savings: 56% fewer tokens**

---

## Compact JSON Schema

### Field Naming Convention

**Standard abbreviations:**

| Full Name | Abbrev | Type | Description |
|-----------|--------|------|-------------|
| success | ok | bool | Operation succeeded |
| error | err | string\|null | Error message |
| message | msg | string | Info message |
| category | cat | string | Phase category |
| percentage | pct | int | Percentage value |
| count | cnt | int | Count value |
| total | tot | int | Total count |
| completed | cmp | int | Completed count |
| modified | mod | []string | Modified files |
| dependencies | dep | []string | Dependencies |
| blockers | blk | []string | Blockers |
| target | tgt | int | Target value |
| current | cur | int | Current value |
| timestamp | ts | int | Unix timestamp |
| description | desc | string | Description |

### Array Notation for Tuples

**Common patterns:**

```json
// Progress: [completed, total, percentage]
"progress": [31, 78, 40]

// Count: [current, target, percentage]
"tests": [17, 36, 47]

// Range: [min, max]
"range": [1, 100]

// Coordinates: [x, y]
"pos": [10, 20]
```

**Benefits:**
- 50% fewer tokens than object notation
- Deterministic order (no key names)
- Faster to parse

### Omit Empty/Null Fields

**Bad:**
```json
{
  "ok": true,
  "err": null,
  "warn": null,
  "blockers": [],
  "deps": []
}
```

**Good:**
```json
{
  "ok": true
}
```

**Rule:** Only include fields with meaningful values.

---

## Command Output Schemas

### `phase complete`

```typescript
{
  ok: boolean,
  phase: string,              // Phase filename
  cat: string,                // Category name
  progress: {
    cat: [number, number, number],   // [completed, total, pct]
    total: [number, number, number]  // [completed, total, pct]
  },
  next: string,               // Next phase path
  mod: string[],              // Modified files
  commit?: string,            // Commit SHA (if --commit)
  ts: number                  // Unix timestamp
}
```

**Example:**
```json
{"ok":true,"phase":"phase-07b","cat":"stdlib","progress":{"cat":[10,21,48],"total":[31,78,40]},"next":"phase-07c","mod":["status/trackers/1-stdlib.md","STATUS.md"],"commit":"a1b2c3d","ts":1708012800}
```

---

### `context current`

```typescript
{
  ok: boolean,
  phase: {
    path: string,
    name: string,
    cat: string,
    desc: string,
    files: string[],          // Files to create/modify
    dep: string[],            // Dependencies (must be complete)
    blk: string[],            // Blockers
    tests: [number, number],  // [target, current]
    accept: string[]          // Acceptance criteria
  },
  progress: {
    cat: [number, number, number],
    total: [number, number, number]
  },
  decisions: Array<{
    id: string,
    title: string,
    path: string
  }>,
  docs: string[]              // Related doc paths
}
```

**Compact example:**
```json
{"ok":true,"phase":{"path":"phases/stdlib/phase-07c.md","name":"phase-07c","cat":"stdlib","desc":"Queue+Stack","files":["queue.rs","stack.rs"],"dep":["phase-07a","phase-07b"],"blk":[],"tests":[36,0],"accept":["36+ tests","FIFO/LIFO","100% parity"]},"progress":{"cat":[10,21,48],"total":[31,78,40]},"decisions":[{"id":"DR-003","title":"Hash design","path":"docs/decision-logs/stdlib/DR-003.md"}],"docs":["docs/api/stdlib.md"]}
```

---

### `decision list`

```typescript
{
  ok: boolean,
  decisions: Array<{
    id: string,              // DR-XXX
    title: string,
    comp: string,            // component (abbrev)
    date: string,            // YYYY-MM-DD
    status: 0|1|2,          // 0=Accepted, 1=Superseded, 2=Deprecated
    path: string
  }>,
  cnt: number,              // Total count
  by_comp: {[key: string]: number}  // Count by component
}
```

**Example:**
```json
{"ok":true,"decisions":[{"id":"DR-001","title":"Value repr","comp":"runtime","date":"2026-01-15","status":0,"path":"docs/decision-logs/runtime/DR-001.md"},{"id":"DR-002","title":"Security ctx","comp":"runtime","date":"2026-01-20","status":0,"path":"docs/decision-logs/runtime/DR-002.md"}],"cnt":2,"by_comp":{"runtime":2,"stdlib":4,"vm":1}}
```

---

### `decision read`

```typescript
{
  ok: boolean,
  decision: {
    id: string,
    title: string,
    date: string,
    status: 0|1|2,
    comp: string,
    context: string,        // Context section (full text)
    decision: string,       // Decision section
    rationale: string,      // Rationale section
    alternatives: string[], // List of alternatives considered
    benefits: string[],     // List of benefits
    tradeoffs: string[],    // List of tradeoffs
    costs: string[],        // List of costs
    impl_notes?: string,    // Implementation notes (optional)
    refs: string[],         // References
    supersedes?: string,    // DR-XXX
    superseded_by?: string  // DR-XXX
  }
}
```

---

### `summary`

```typescript
{
  ok: boolean,
  progress: [number, number, number],  // [completed, total, pct]
  last: string,                        // Last completed phase
  next: string,                        // Next phase
  cats: Array<{
    name: string,
    prog: [number, number, number],    // [completed, total, pct]
    status: 0|1|2|3                    // 0=pending, 1=active, 2=blocked, 3=complete
  }>
}
```

**Example:**
```json
{"ok":true,"progress":[31,78,40],"last":"phase-07b","next":"phase-07c","cats":[{"name":"foundation","prog":[21,21,100],"status":3},{"name":"stdlib","prog":[10,21,48],"status":1},{"name":"bytecode-vm","prog":[0,8,0],"status":0}]}
```

---

## Enum Mappings

### Status Codes

```
0 = pending
1 = active / in_progress
2 = blocked
3 = complete
```

### Decision Status

```
0 = Accepted
1 = Superseded
2 = Deprecated
```

### Component Abbreviations

```
foundation = found
stdlib = lib
bytecode-vm = vm
frontend = front
typing = type
interpreter = interp
cli = cli
lsp = lsp
polish = pol
```

---

## Performance: SQLite = Built-In Speed

**NO FILE CACHING NEEDED** - SQLite with indexes IS the cache.

### Why No Cache Files?

**Old approach (markdown):**
- ❌ Parse 78 phase files (slow)
- ❌ Index 50+ decision logs (slow)
- ❌ Build doc hierarchy (slow)
- ❌ Need cache files (phase-index.json, etc.)

**New approach (SQLite):**
- ✅ All data in indexed database
- ✅ Queries < 1ms (prepared statements)
- ✅ No file parsing needed
- ✅ No cache files needed

**Database = cache.** Indexed SQL is faster than file caching.

---

## Response Time Targets

**All queries use indexed SQL (< 1ms):**
- `phase complete`: < 100ms (write + transaction)
- `phase current`: < 1ms (indexed query)
- `phase next`: < 1ms (indexed query)
- `decision list`: < 5ms (indexed query)
- `summary`: < 10ms (aggregate queries)

**Target: All reads < 10ms** (no cache needed - DB is fast)

---

## Composability

### Piping Commands

**All commands output JSON to stdout:**
```bash
# Get current phase, then get its dependencies
atlas-dev context current | jq -r '.phase.dep[]'

# List all active categories
atlas-dev summary | jq -r '.cats[] | select(.status==1) | .name'

# Find all decisions from 2026
atlas-dev decision list | jq -r '.decisions[] | select(.date | startswith("2026")) | .id'
```

### Exit Codes

```
0 = Success
1 = Invalid arguments
2 = File not found
3 = Validation failed
4 = Git operation failed
5 = Cache error
6 = Permission denied
```

**AI can check exit code:**
```bash
if atlas-dev phase complete "..."; then
  echo "Success"
else
  echo "Failed with code $?"
fi
```

---

## Error Handling

### Error Response Format

```json
{
  "ok": false,
  "err": "Phase not found in tracker",
  "details": {
    "phase": "phase-07x-invalid.md",
    "tracker": "status/trackers/1-stdlib.md",
    "available": ["phase-07a", "phase-07b", "phase-07c"]
  },
  "suggestion": "Did you mean: phase-07c?"
}
```

**Rules:**
- Always include `ok: false`
- Provide actionable error message
- Include relevant details for debugging
- Suggest fixes when possible

---

## Hard-Coded Defaults (No Config Files)

**NO configuration needed.** Everything is hard-coded for AI-only:

```
Output (hard-coded):
  ✅ JSON always (compact, minified)
  ✅ Abbreviated field names (cat, pct, cnt, tot)
  ✅ Omit null/empty fields
  ✅ Arrays for tuples ([10,21,48])
  ✅ Numeric enums (0=pending, 1=active, 2=blocked, 3=complete)
  ✅ No colors, no emoji, no pretty-printing

Flags that DON'T exist:
  ❌ No --human flag (JSON is the ONLY output)
  ❌ No --pretty flag (always compact)
  ❌ No --verbose flag (structured errors only)
  ❌ No config files (everything hard-coded)

AI-only. No options. No configuration.
```

---

## Best Practices for AI Agents

### 1. All Output is JSON (Only Mode)

```bash
# Always JSON (only mode)
result=$(atlas-dev context current)
phase=$(echo $result | jq -r '.phase.path')

# Parse with jq
deps=$(atlas-dev context current | jq -r '.phase.dep[]')
```

### 2. Check Exit Code

```bash
if ! atlas-dev phase complete "..."; then
  echo "Failed to complete phase"
  exit 1
fi
```

### 3. Parse JSON with jq

```bash
# Get next phase
next=$(atlas-dev context current | jq -r '.next')

# Get all dependencies
deps=$(atlas-dev context current | jq -r '.phase.dep[]')

# Get test target
target=$(atlas-dev context current | jq -r '.phase.tests[0]')
```

### 4. Use --dry-run for Preview

```bash
# Preview changes without modifying files
atlas-dev phase complete "..." --dry-run

# Returns same JSON, but mod=[] and no commit
```




---

## Performance Benchmarks

### Target Performance (with cache)

| Command | Tokens In | Tokens Out | Time | Cache Hit |
|---------|-----------|------------|------|-----------|
| `phase complete` | 50 | 80 | <1s | N/A |
| `context current` | 20 | 150 | <0.05s | Yes |
| `decision list` | 20 | 100 | <0.01s | Yes |
| `decision read DR-001` | 25 | 200 | <0.01s | Partial |
| `summary` | 15 | 120 | <0.5s | Partial |

### Token Budget

**Conservative estimate:**
- Average command: 30 tokens input, 100 tokens output
- 10 commands per session: 1,300 tokens
- 100 sessions: 130,000 tokens

**At $3/1M input + $15/1M output:**
- Cost per session: $0.0019
- Cost per 100 sessions: $0.19

**Negligible compared to LLM reasoning tokens.**

---

## Summary

**Key Principles:**
1. ✅ JSON by default (always parseable)
2. ✅ Compact notation (arrays, abbrevs, omit empty)
3. ✅ Fast responses (<100ms with cache)
4. ✅ Composable (pipe-friendly)
5. ✅ Deterministic (same input = same output)
6. ✅ Stateless (no hidden state)

**Result:**
- 50%+ token savings vs verbose output
- 20x+ speed improvement with caching
- 100% reliability (structured data, no parsing errors)
- Perfect for AI agents (designed from scratch for LLMs)

**This is what world-class AI tooling looks like.**
