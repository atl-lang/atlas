# Phase Infra-03g: Flatten security.rs

## Blocker
**REQUIRED:** Infra-03d complete (common/ has shared helpers).

---

## Objective
Remove the `mod permissions { }`, `mod runtime { }`, `mod audit { }` wrappers from
`security.rs` (1,527 lines). Produce a flat file with a single use block and logical
section comments.

## Current structure
```
mod permissions { use...; fn...; #[test] fn test_... }  // security_tests.rs
mod runtime     { use...; fn...; #[test] fn test_... }  // runtime_security_tests.rs
mod audit       { use...; fn...; #[test] fn test_... }  // audit_logging_tests.rs
```

## Target structure
```rust
use ...  // merged, deduplicated

// --- Shared helpers ---
fn security() -> SecurityContext { ... }  // OR: use common::security;

// --- Permission model ---
#[test] fn test_...

// --- Runtime enforcement ---
#[test] fn test_...

// --- Audit logging ---
#[test] fn test_...
```

## Implementation

### Step 1: Scout mod boundaries and inner helpers
```bash
grep -n "^mod \|^    fn \|^    use " crates/atlas-runtime/tests/security.rs | head -50
```

### Step 2: Check which common/ helpers are used
```bash
grep -n "fn security\|fn eval_ok" crates/atlas-runtime/tests/security.rs
```
If `security()` exists in all 3 mods → remove 2 copies, add `use common::security;` after flattening.

### Step 3: Flatten with python
Read file in python, extract inner content from each mod (remove 4-space indent), merge
use statements, remove duplicates, write flat output.

### Step 4: Handle `test_import_nonexistent_export` collision
If this duplicate test appears here: it doesn't (confirmed earlier). No action needed.

### Step 5: Verify
```bash
cargo nextest run -p atlas-runtime --test security
```
Count must match: 94 passed, 1 skipped.

## Acceptance
- No `mod` blocks remain
- Single use block at top, no duplicates
- `security()` helper appears once (or imported from common/)
- 94 pass, 1 skip
- Commit: `refactor(tests): Infra-03g — flatten security.rs`
- Update STATUS.md: mark 03g complete, Next Phase → 03h
