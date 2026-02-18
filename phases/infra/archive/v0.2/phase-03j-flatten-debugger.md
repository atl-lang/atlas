# Phase Infra-03j: Flatten debugger.rs

## Blocker
**REQUIRED:** Infra-03d complete.

---

## Objective
Remove the `mod execution { }`, `mod inspection { }`, `mod protocol { }` wrappers
from `debugger.rs` (1,921 lines). Domain-specific helpers (`compile`, `loc`, `span`)
stay in this file — they are NOT moved to common/.

## Current structure
```
mod execution  { use...; fn compile; fn loc; fn security; #[test] fn... }
mod inspection { use...; fn compile; fn loc; fn security; #[test] fn... }
mod protocol   { use...; fn compile; fn loc; fn security; #[test] fn... }
```

## Target structure
```rust
use ...

// --- Shared debugger helpers ---
fn compile(source: &str) -> Bytecode { ... }   // ONE copy — debugger-specific
fn loc(line: u32) -> SourceLocation { ... }     // ONE copy — debugger-specific
fn span() -> Span { ... }                        // ONE copy
fn security() -> SecurityContext { ... }         // ONE copy (or use common::security)

// --- Execution control ---
#[test] fn test_breakpoint_...

// --- Inspection ---
#[test] fn test_inspect_...

// --- Protocol ---
#[test] fn test_step_...
```

## Implementation

### Step 1: Scout mod boundaries and duplicate helpers
```bash
grep -n "^mod \|^    fn compile\|^    fn loc\|^    fn security\|^    fn span" \
  crates/atlas-runtime/tests/debugger.rs
```

### Step 2: Check `security()` — use common/ or keep local?
```bash
grep -n "fn security" crates/atlas-runtime/tests/debugger.rs
```
If it returns `SecurityContext::allow_all()` — use `common::security` instead.
If it has custom configuration — keep it local.

### Step 3: Flatten with python
Extract inner content from all 3 mods, deduplicate the 3-4 shared helpers (keep first
occurrence), merge use statements, remove 4-space indent, write flat file.

### Step 4: Verify
```bash
cargo nextest run -p atlas-runtime --test debugger
```
Count must match: 168 passed, 0 skipped.

## Acceptance
- No `mod` blocks remain
- `compile`, `loc`, `span` appear exactly once
- 168 pass, 0 skip
- Zero clippy warnings
- Commit: `refactor(tests): Infra-03j — flatten debugger.rs`
- Update STATUS.md: mark 03j complete, Next Phase → 03k
