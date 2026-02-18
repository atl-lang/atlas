# Phase Infra-03m: Flatten system.rs

## Blocker
**REQUIRED:** Infra-03d complete (common/ has shared helpers).

---

## Objective
Remove the 6 mod wrappers from `system.rs` (3,821 lines):
`path`, `fs`, `process`, `gzip`, `tar`, `zip`

This is the largest flatten (3,821 lines, 6 mods). Scout carefully — do NOT read the full
file. Use python to flatten programmatically.

## Current structure
```
mod path    { use...; fn...; #[test] fn test_path_... }
mod fs      { use...; fn...; #[test] fn test_fs_... }
mod process { use...; fn...; #[test] fn test_process_... }
mod gzip    { use...; fn...; #[test] fn test_gzip_... }
mod tar     { use...; fn...; #[test] fn test_tar_... }
mod zip     { use...; fn...; #[test] fn test_zip_... }
```

## Target structure
```rust
use ...

// --- Shared filesystem helpers ---
fn create_test_dir(...) { ... }   // ONE copy — or use common::create_test_dir
fn create_test_file(...) { ... }  // ONE copy — or use common::create_test_file

// --- Path manipulation ---
#[test] fn test_path_...

// --- Filesystem operations ---
#[test] fn test_fs_...

// --- Process management ---
#[test] fn test_process_...

// --- Gzip compression ---
#[test] fn test_gzip_...

// --- Tar archives ---
#[test] fn test_tar_...

// --- Zip archives ---
#[test] fn test_zip_...
```

## Implementation

### Step 1: Scout mod boundaries and shared helpers
```bash
grep -n "^mod \|^    fn \|^    pub fn " crates/atlas-runtime/tests/system.rs | \
  grep -v "test_" | head -40
```

### Step 2: Find duplicate helpers across mods
```bash
grep -h "^    fn \|^    pub fn " crates/atlas-runtime/tests/system.rs | \
  grep -v "test_" | sort | uniq -d
```
Expect: `create_test_dir`, `create_test_file`, possibly TempDir-based helpers.

### Step 3: Check which helpers can use common/
```bash
grep -n "fn create_test_dir\|fn create_test_file\|fn str_array_value\|fn str_value" \
  crates/atlas-runtime/tests/system.rs
```
If signatures match common/ versions → remove inner copies, use `common::*`.
If signatures differ (e.g., different TempDir usage) → keep one local copy.

### Step 4: Flatten with python
```bash
# The 6 mods each have test_prefix naming already (test_path_, test_fs_, etc.)
# so flattening to top level will not cause name collisions.
python3 - <<'EOF'
# flatten_mods(path, output_path) using same proven pattern
EOF
```

### Step 5: Verify
```bash
cargo nextest run -p atlas-runtime --test system
```
Count must match: 210 passed, 0 skipped.

## Acceptance
- No `mod` blocks remain
- 6 section comments present
- Shared helpers appear once (or imported from common/)
- 210 pass, 0 skip
- Zero clippy warnings
- Commit: `refactor(tests): Infra-03m — flatten system.rs`
- Update STATUS.md: mark 03m complete, Next Phase → phase-04-ignore-audit.md
