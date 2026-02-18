# Phase Infra-03l: Flatten api.rs

## Blocker
**REQUIRED:** Infra-03d complete (common/ has shared helpers).

---

## Objective
Remove the 7 mod wrappers from `api.rs` (2,955 lines):
`api_core`, `api_conversion`, `api_native`, `api_sandboxing`, `reflection`, `json_value`, `runtime_api`

This is the most complex flatten — 7 mods, 2,955 lines. Work carefully: scout first,
then flatten one concern at a time using a python script. Do NOT read the full file.

## Current structure
```
mod api_core       { use...; fn...; #[test] fn... }
mod api_conversion { use...; fn...; #[test] fn... }
mod api_native     { use...; fn...; #[test] fn... }
mod api_sandboxing { use...; fn...; #[test] fn... }
mod reflection     { use...; fn...; #[test] fn... }
mod json_value     { use...; fn...; #[test] fn... }
mod runtime_api    { use...; fn...; #[test] fn... }
```

## Target structure
```rust
use ...

// --- Shared helpers ---
fn eval_ok(...) { ... }    // ONE copy — or use common::eval_ok

// --- Core API ---
#[test] fn test_api_...

// --- Type conversions ---
#[test] fn test_conversion_...

// --- Native functions ---
#[test] fn test_native_...

// --- Sandboxing ---
#[test] fn test_sandbox_...

// --- Reflection ---
#[test] fn test_reflect_...

// --- JSON value handling ---
#[test] fn test_json_...

// --- Runtime API surface ---
#[test] fn test_runtime_...
```

## Implementation

### Step 1: Scout mod boundaries and helpers
```bash
grep -n "^mod \|^    fn \|^    pub fn " crates/atlas-runtime/tests/api.rs | \
  grep -v "test_" | head -40
```

### Step 2: Find duplicated helpers across mods
```bash
grep -h "^    fn \|^    pub fn " crates/atlas-runtime/tests/api.rs | \
  grep -v "test_" | sort | uniq -d
```

### Step 3: Check which common/ helpers apply
```bash
grep -n "fn eval_ok\|fn str_value\|fn extract_number\|fn extract_bool" \
  crates/atlas-runtime/tests/api.rs
```
For each found: remove the inner copy, add `use common::{...};` at top level after flattening.

### Step 4: Flatten with python
Same flatten script pattern. Merge all 7 mods' use statements, deduplicate helpers,
unindent content, write flat with 7 section comments.

### Step 5: Verify
```bash
cargo nextest run -p atlas-runtime --test api
```
Count must match: 287 passed, 3 skipped.

## Acceptance
- No `mod` blocks remain
- 7 section comments present
- Shared helpers appear once (or imported from common/)
- 287 pass, 3 skip
- Commit: `refactor(tests): Infra-03l — flatten api.rs`
- Update STATUS.md: mark 03l complete, Next Phase → 03m
