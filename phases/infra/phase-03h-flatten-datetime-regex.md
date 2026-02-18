# Phase Infra-03h: Flatten datetime_regex.rs

## Blocker
**REQUIRED:** Infra-03d complete.

---

## Objective
Remove the `mod datetime_core { }`, `mod datetime_advanced { }`, `mod regex_core { }`,
`mod regex_ops { }` wrappers from `datetime_regex.rs` (1,593 lines).

## Current structure
```
mod datetime_core     { use...; #[test] fn test_... }
mod datetime_advanced { use...; #[test] fn test_... }
mod regex_core        { use...; #[test] fn test_... }
mod regex_ops         { use...; #[test] fn test_... }
```

## Target structure
```rust
use ...

// --- DateTime core ---
#[test] fn test_datetime_...

// --- DateTime advanced ---
#[test] fn test_datetime_advanced_...

// --- Regex core ---
#[test] fn test_regex_...

// --- Regex operations ---
#[test] fn test_regex_ops_...
```

## Implementation

### Step 1: Scout
```bash
grep -n "^mod \|^    fn \|^    use \|^    #\[test\]" \
  crates/atlas-runtime/tests/datetime_regex.rs | head -60
```

### Step 2: Check for shared helpers
```bash
grep -n "^    fn " crates/atlas-runtime/tests/datetime_regex.rs | grep -v "test_"
```
These domains likely have no shared helper overlap — confirm with grep before proceeding.

### Step 3: Flatten with python
Same flatten pattern as 03e/03g: extract inner mod content, merge use statements,
remove 4-space indent, write flat file with section comments.

### Step 4: Verify
```bash
cargo nextest run -p atlas-runtime --test datetime_regex
```
Count must match: 141 passed, 0 skipped.

## Acceptance
- No `mod` blocks remain
- 4 clear section comments matching the 4 domains
- 141 pass, 0 skip
- Commit: `refactor(tests): Infra-03h — flatten datetime_regex.rs`
- Update STATUS.md: mark 03h complete, Next Phase → 03i
