# Phase Infra-03i: Flatten modules.rs

## Blocker
**REQUIRED:** Infra-03d complete.

---

## Objective
Remove the `mod binding { }`, `mod execution { }`, `mod execution_vm { }`,
`mod resolution { }` wrappers from `modules.rs` (1,806 lines).

**Note on parity:** `execution` (interpreter) and `execution_vm` (VM) are parity tests.
They have a duplicate `test_import_nonexistent_export` — resolve by renaming the copy in
`execution_vm` to `test_import_nonexistent_export_vm` to make the parity explicit.

## Current structure
```
mod binding      { use...; fn...; #[test] fn test_... }
mod execution    { use...; fn...; #[test] fn test_... }  // interpreter
mod execution_vm { use...; fn...; #[test] fn test_... }  // VM parity
mod resolution   { use...; fn...; #[test] fn test_... }
```

## Target structure
```rust
use ...

fn create_module(...) { ... }  // shared helper — ONE copy

// --- Module binding ---
#[test] fn test_module_bind_...

// --- Module execution (interpreter) ---
#[test] fn test_...

// --- Module execution (VM) ---
#[test] fn test_vm_...
#[test] fn test_import_nonexistent_export_vm() { ... }  // renamed parity test

// --- Module resolution ---
#[test] fn test_resolve_...
```

## Implementation

### Step 1: Scout
```bash
grep -n "^mod \|^    fn \|^    use " crates/atlas-runtime/tests/modules.rs | head -60
```

### Step 2: Find duplicate helpers between execution and execution_vm
```bash
grep -n "^    fn " crates/atlas-runtime/tests/modules.rs | grep -v "test_" | sort
```
Likely `create_module` and `execute_with_*` helpers — deduplicate to one copy.

### Step 3: Handle the duplicate test
```bash
grep -n "test_import_nonexistent_export" crates/atlas-runtime/tests/modules.rs
```
Rename the second occurrence (execution_vm's copy) to `test_import_nonexistent_export_vm`.

### Step 4: Flatten with python
Same flatten pattern. Note: VM-specific tests already have `test_vm_` prefix so section
separation is natural.

### Step 5: Verify
```bash
cargo nextest run -p atlas-runtime --test modules
```
Count must be 60 (or 61 if renamed test was previously hidden — verify).

## Acceptance
- No `mod` blocks remain
- `test_import_nonexistent_export_vm` exists (renamed parity test)
- Shared helpers appear once
- All tests pass
- Commit: `refactor(tests): Infra-03i — flatten modules.rs`
- Update STATUS.md: mark 03i complete, Next Phase → 03j
