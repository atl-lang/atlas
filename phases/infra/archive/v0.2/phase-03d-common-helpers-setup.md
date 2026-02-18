# Phase Infra-03d: Common Helpers Setup

## Blocker
**REQUIRED:** Infra-03c complete (20-file structure in place).

```bash
ls crates/atlas-runtime/tests/*.rs | wc -l  # must be 20
```

---

## Objective
Expand `tests/common/mod.rs` with the shared helpers that currently live duplicated inside mod-wrapped files. This is the prerequisite for phases 03e–03m which flatten those files.

## Duplicate helpers to extract (confirmed by grep)

These appear duplicated across 2+ of the 7 problem files:
- `eval_ok(code: &str) -> String`
- `extract_bool(value: &Value) -> bool`
- `extract_number(value: &Value) -> f64`
- `str_value(s: &str) -> Value`
- `str_array_value(paths: &[&str]) -> Value`
- `security() -> SecurityContext`
- `create_test_dir(dir: &Path, name: &str) -> PathBuf`
- `create_test_file(dir: &Path, name: &str, content: &str)`

**Do NOT extract:** `compile`, `loc`, `span` — debugger-specific, stay in `debugger.rs`.

## Implementation

### Step 1: Read current common/mod.rs
```bash
# Check what already exists (200 lines, manageable)
wc -l crates/atlas-runtime/tests/common/mod.rs
```
Read the file. Then grep each problem file for the actual signatures of the helpers above:
```bash
grep -n "^    fn eval_ok\|^    fn extract_bool\|^    fn extract_number\|^    fn str_value\|^    fn str_array_value\|^    fn security\|^    fn create_test_dir\|^    fn create_test_file" \
  crates/atlas-runtime/tests/security.rs \
  crates/atlas-runtime/tests/api.rs \
  crates/atlas-runtime/tests/system.rs
```
Read only the matched line + ~10 lines after to get each signature and body. Do NOT read the full files.

### Step 2: Add helpers to common/mod.rs
Append the canonical versions of each helper. Use the most complete/general implementation found. Mark `#[allow(dead_code)]` — the module already has this at the top.

The `security()` helper should return `SecurityContext::allow_all()` (matching CLI/DR-001).

### Step 3: Verify common/mod.rs compiles
```bash
cargo check -p atlas-runtime 2>&1 | grep -E "error|warning" | head -20
```

## Tests
No test changes. This phase only adds to common/mod.rs.

## Acceptance
- `common/mod.rs` contains all 8 helpers listed above
- `cargo check -p atlas-runtime` clean
- Commit: `refactor(tests): Infra-03d — expand common test helpers`
- Update STATUS.md: mark 03d complete, Next Phase → 03e
