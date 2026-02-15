# AI Optimization Guide
## Making status-manager Perfect for LLM Agents

**Core Principle:** This tool is 100% for AI agents. Every byte counts.

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

## Caching for Speed

### Cache Strategy

**Cache these expensive operations:**
1. Phase file parsing (78 files)
2. Decision log indexing (50+ files)
3. Doc hierarchy building (100+ files)

**Don't cache:**
1. Current progress (changes frequently)
2. Validation results (must be fresh)
3. Git operations (must be real-time)

### Cache Structure

```
~/.cache/atlas-status-manager/
├── phase-index.json        # All phase metadata
├── decision-index.json     # All decision log metadata
├── doc-index.json          # Doc hierarchy
└── .last-update            # Cache timestamp
```

### Cache File Format

**phase-index.json:**
```json
{
  "version": 1,
  "updated": 1708012800,
  "phases": {
    "phases/stdlib/phase-07a.md": {
      "name": "phase-07a",
      "cat": "stdlib",
      "desc": "Hash + HashMap",
      "files": ["hash.rs", "hashmap.rs"],
      "dep": ["phase-06c"],
      "blk": [],
      "tests": [35, 0],
      "accept": ["35+ tests", "100% parity"]
    }
  }
}
```

**decision-index.json:**
```json
{
  "version": 1,
  "updated": 1708012800,
  "decisions": {
    "DR-001": {
      "id": "DR-001",
      "title": "Value representation",
      "comp": "runtime",
      "date": "2026-01-15",
      "status": 0,
      "path": "docs/decision-logs/runtime/DR-001.md",
      "keywords": ["value", "representation", "memory"]
    }
  }
}
```

### Cache Invalidation

**Invalidate when:**
- Any tracked file modified (check mtime)
- Git HEAD changes (check `.git/HEAD`)
- Manual clear (`cache clear`)

**Validation:**
```bash
# Check if cache is fresh
latest_mtime=$(find status/ phases/ docs/decision-logs/ -type f -name "*.md" -exec stat -f %m {} \; | sort -n | tail -1)
cache_time=$(cat ~/.cache/atlas-status-manager/.last-update)

if [ $latest_mtime -gt $cache_time ]; then
  # Rebuild cache
fi
```

---

## Response Time Targets

### Without Cache (Cold)
- `phase complete`: <1 sec (writes are fast)
- `context current`: <2 sec (parses phase file + searches)
- `decision list`: <1 sec (scans directory)
- `decision read`: <0.5 sec (reads single file)
- `summary`: <1 sec (reads STATUS.md + trackers)

### With Cache (Warm)
- `phase complete`: <1 sec (same, writes don't use cache)
- `context current`: <0.05 sec (reads cache)
- `decision list`: <0.01 sec (reads index)
- `decision read`: <0.01 sec (cached metadata + file read)
- `summary`: <0.5 sec (STATUS.md only)

**Target: All reads <100ms with cache**

---

## Composability

### Piping Commands

**All commands output JSON to stdout:**
```bash
# Get current phase, then get its dependencies
status-manager context current | jq -r '.phase.dep[]'

# List all active categories
status-manager summary | jq -r '.cats[] | select(.status==1) | .name'

# Find all decisions from 2026
status-manager decision list | jq -r '.decisions[] | select(.date | startswith("2026")) | .id'
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
if status-manager phase complete "..."; then
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

## Configuration

### Default Config (AI-optimized)

```toml
[output]
format = "json"         # Always JSON (not "human")
compact = true          # Minified JSON (no pretty-print)
color = false           # No ANSI colors
emoji = false           # No emoji
abbreviate = true       # Use short field names

[cache]
enabled = true
ttl_hours = 24
max_size_mb = 100

[ai]
optimize = true         # Enable all AI optimizations
omit_empty = true       # Skip null/empty fields
use_arrays = true       # Use array notation for tuples
use_enums = true        # Use numeric enums
```

### Override for Humans

```bash
# Human-readable output
status-manager summary --human

# Pretty-printed JSON
status-manager summary --pretty

# Verbose errors
status-manager phase complete "..." --verbose
```

---

## Best Practices for AI Agents

### 1. Always Use JSON Mode (Default)

```bash
# Good (JSON output, parseable)
result=$(status-manager context current)
phase=$(echo $result | jq -r '.phase.path')

# Bad (human mode, hard to parse)
status-manager context current --human
```

### 2. Check Exit Code

```bash
if ! status-manager phase complete "..."; then
  echo "Failed to complete phase"
  exit 1
fi
```

### 3. Parse JSON with jq

```bash
# Get next phase
next=$(status-manager context current | jq -r '.next')

# Get all dependencies
deps=$(status-manager context current | jq -r '.phase.dep[]')

# Get test target
target=$(status-manager context current | jq -r '.phase.tests[0]')
```

### 4. Use --dry-run for Preview

```bash
# Preview changes without modifying files
status-manager phase complete "..." --dry-run

# Returns same JSON, but mod=[] and no commit
```

### 5. Cache Awareness

```bash
# Clear cache before critical operations
status-manager cache clear

# Get fresh data
status-manager context current --no-cache
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
