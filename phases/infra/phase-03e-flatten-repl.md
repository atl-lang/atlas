# Phase Infra-03e: Flatten repl.rs

## Blocker
**REQUIRED:** Infra-03d complete (common/mod.rs has shared helpers).

---

## Objective
Remove the `mod state { }` and `mod types { }` wrappers from `repl.rs` (366 lines).
Produce a flat file with a single clean use block at the top.

## Current structure
```
mod state {   // repl_state_tests.rs content
    use ...
    fn ...
    #[test] fn ...
}

mod types {   // repl_types_tests.rs content
    use ...
    fn ...
    #[test] fn ...
}
```

## Target structure
```
use ...  // merged, deduplicated

// --- State tests ---
#[test] fn test_repl_...

// --- Type snapshot tests ---
#[test] fn test_...
```

## Implementation

### Step 1: Scout the file (no full read)
```bash
grep -n "^mod \|^    fn \|^    pub fn \|^    use " crates/atlas-runtime/tests/repl.rs | head -40
```

### Step 2: Flatten with python
```python
# Read file, extract content from each mod block, merge use statements, write flat.
# Strip mod wrappers, remove 4-space indentation, deduplicate use lines.
# Replace: // ===== filename.rs ===== banners with: // --- meaningful section ---
```

Run this pattern (see gates.md for canonical flatten script). The file is only 366 lines — low risk.

### Step 3: Replace common/ duplicates
If `eval_ok` or any helper from common/ appears in the file, remove it and add:
```rust
use common::*;  // or specific: use common::{eval_ok, ...};
```

### Step 4: Verify
```bash
cargo nextest run -p atlas-runtime --test repl
```

## Acceptance
- `repl.rs` has no `mod` blocks
- Single clean use block at top (no duplicates)
- All tests pass (same count as before)
- Commit: `refactor(tests): Infra-03e — flatten repl.rs`
- Update STATUS.md: mark 03e complete, Next Phase → 03f
