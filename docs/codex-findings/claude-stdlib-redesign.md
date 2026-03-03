# Atlas Stdlib Redesign Analysis

**Date:** 2026-03-03
**Status:** Deferred (post-grammar cleanup)
**Priority:** v0.3 Phase 2 (after grammar rewrite)

---

## Current State

Atlas stdlib consists of ~300 global functions dumped into the namespace:

```atlas
// Current Atlas (procedural, 1990s PHP-style)
let data = parseJSON(str);
let output = toJSON(data);
let trimmed = trim(str);
let len = length(arr);
```

## What Modern Languages Do

### Go
```go
import "encoding/json"
json.Unmarshal([]byte(str), &data)
json.Marshal(data)
```

### Rust
```rust
use serde_json;
let data: Value = serde_json::from_str(&str)?;
let output = serde_json::to_string(&data)?;
```

### Python
```python
import json
data = json.loads(str)
output = json.dumps(data)
```

### TypeScript
```typescript
const data = JSON.parse(str);
const output = JSON.stringify(data);
```

**Common patterns:**
- Namespaced (module.function or Type.method)
- Discoverable (IDE autocomplete on module/type)
- Method-chainable where appropriate
- No global namespace pollution

## Problems with Current Approach

1. **300+ global functions** - namespace pollution, no discoverability
2. **No module organization** - `parseJSON`, `readFile`, `sin`, `cos` all in same global scope
3. **Not method-chainable** - `trim(toUpperCase(str))` instead of `str.toUpperCase().trim()`
4. **Inconsistent naming** - `parseJSON` vs `toString` vs `length`
5. **AI confusion** - no namespace hints about what functions exist or relate to each other

## Proposed Redesign

### Option A: Module-Based (Go/Python style)
```atlas
import json from "std/json";
import fs from "std/fs";

let data = json.parse(str);
let content = fs.readFile("config.json");
```

### Option B: Method-Based (Rust/modern JS style)
```atlas
let data = str.parseJSON()?;
let content = "config.json".readFile()?;
let doubled = [1, 2, 3].map((x) => x * 2);
```

### Option C: Hybrid (Recommended)
```atlas
// Modules for external operations
import fs from "std/fs";
import http from "std/http";

// Methods for type operations
let data = str.parseJSON()?;
let upper = str.toUpperCase();
let len = arr.length();

// Module functions for complex operations
let content = fs.read("config.json")?;
let response = http.get("https://api.example.com")?;
```

## Blast Radius

Redesigning stdlib affects:
- Every test file (~6,700 tests)
- Every example
- All documentation
- Symbol table / builtin registry
- Method dispatch system
- Module resolution system

**Estimated effort:** 1-2 weeks minimum

## Decision

**Deferred until grammar cleanup complete.**

Grammar issues (ambiguous `{}`, redundant syntax, `Type::Unknown`) are blocking AI code generation NOW. Stdlib naming is ugly but functional - `parseJSON()` generates correctly even if it's not discoverable.

Fix grammar first. Plan stdlib redesign as v0.3 Phase 2 or dedicated block.

---

## References

- Grammar audit: `docs/codex-findings/atlas-language-issues-advanced.md`
- V02 lessons: `docs/internal/V02_LESSONS_LEARNED.md`
- Current stdlib: `crates/atlas-runtime/src/stdlib/`
