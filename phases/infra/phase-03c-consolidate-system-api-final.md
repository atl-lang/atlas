# Phase Infra-03c: Test Consolidation — System, API, Final Audit

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Infra-03a and Infra-03b complete.

```bash
cargo nextest run -p atlas-runtime 2>&1 | tail -5
ls crates/atlas-runtime/tests/*.rs | wc -l  # should be ~29 after 03a+03b
```

---

## Objective
Final consolidation: merge 18 source files into 5 domain files, rename regression_suite.rs, and produce the finished 17-file test structure. Includes final verification pass and measurement.

## Source → Target Mapping

| Target | Sources | Notes |
|--------|---------|-------|
| `datetime_regex.rs` | `datetime_core_tests.rs`, `datetime_advanced_tests.rs`, `regex_core_tests.rs`, `regex_operations_tests.rs` | Small domains, bundle as stdlib extensions |
| `system.rs` | `path_tests.rs`, `fs_tests.rs`, `process_tests.rs`, `gzip_tests.rs`, `tar_tests.rs`, `zip_tests.rs` | OS-level interactions |
| `api.rs` | `api_tests.rs`, `api_conversion_tests.rs`, `api_native_functions_tests.rs`, `api_sandboxing_tests.rs`, `reflection_tests.rs`, `json_value_tests.rs`, `runtime_api.rs` | Public embedding API surface |
| `repl.rs` | `repl_state_tests.rs`, `repl_types_tests.rs` | REPL state management |
| `regression.rs` | `regression_suite.rs` | Rename only — no merge |

---

## CRITICAL EXECUTION APPROACH

**DO NOT use the `Read` tool to read source files in full.**

Use Grep first to scout structure, then use the python merge pattern:

```bash
# Count tests before starting (record this number)
cargo nextest list -p atlas-runtime | wc -l
```

---

## Implementation Steps

### Step 1: Create `tests/datetime_regex.rs`

```bash
grep -h "^use \|^extern crate " \
  crates/atlas-runtime/tests/datetime_core_tests.rs \
  crates/atlas-runtime/tests/datetime_advanced_tests.rs \
  crates/atlas-runtime/tests/regex_core_tests.rs \
  crates/atlas-runtime/tests/regex_operations_tests.rs | sort -u

grep -h "^fn \|^pub fn " \
  crates/atlas-runtime/tests/datetime_core_tests.rs \
  crates/atlas-runtime/tests/datetime_advanced_tests.rs \
  crates/atlas-runtime/tests/regex_core_tests.rs \
  crates/atlas-runtime/tests/regex_operations_tests.rs | sort | uniq -d
```

Write merged file with two clear sections (datetime | regex). Verify:
```bash
cargo nextest run -p atlas-runtime --test datetime_regex
```

### Step 2: Create `tests/system.rs`

Note: this is 6 source files (path, fs, process, gzip, tar, zip).

```bash
grep -h "^use \|^extern crate " \
  crates/atlas-runtime/tests/path_tests.rs \
  crates/atlas-runtime/tests/fs_tests.rs \
  crates/atlas-runtime/tests/process_tests.rs \
  crates/atlas-runtime/tests/gzip_tests.rs \
  crates/atlas-runtime/tests/tar_tests.rs \
  crates/atlas-runtime/tests/zip_tests.rs | sort -u

grep -h "^fn \|^pub fn " \
  crates/atlas-runtime/tests/path_tests.rs \
  crates/atlas-runtime/tests/fs_tests.rs \
  crates/atlas-runtime/tests/process_tests.rs \
  crates/atlas-runtime/tests/gzip_tests.rs \
  crates/atlas-runtime/tests/tar_tests.rs \
  crates/atlas-runtime/tests/zip_tests.rs | sort | uniq -d
```

Write merged file. Verify:
```bash
cargo nextest run -p atlas-runtime --test system
```

### Step 3: Create `tests/api.rs`

Note: includes `json_value_tests.rs` and `runtime_api.rs` which were not listed in original phase-03 but exist in the test dir.

```bash
grep -h "^use \|^extern crate " \
  crates/atlas-runtime/tests/api_tests.rs \
  crates/atlas-runtime/tests/api_conversion_tests.rs \
  crates/atlas-runtime/tests/api_native_functions_tests.rs \
  crates/atlas-runtime/tests/api_sandboxing_tests.rs \
  crates/atlas-runtime/tests/reflection_tests.rs \
  crates/atlas-runtime/tests/json_value_tests.rs \
  crates/atlas-runtime/tests/runtime_api.rs | sort -u

grep -h "^fn \|^pub fn " \
  crates/atlas-runtime/tests/api_tests.rs \
  crates/atlas-runtime/tests/api_conversion_tests.rs \
  crates/atlas-runtime/tests/api_native_functions_tests.rs \
  crates/atlas-runtime/tests/api_sandboxing_tests.rs \
  crates/atlas-runtime/tests/reflection_tests.rs \
  crates/atlas-runtime/tests/json_value_tests.rs \
  crates/atlas-runtime/tests/runtime_api.rs | sort | uniq -d
```

Write merged file. Verify:
```bash
cargo nextest run -p atlas-runtime --test api
```

### Step 4: Create `tests/repl.rs`

```bash
grep -h "^use \|^fn \|^pub fn " \
  crates/atlas-runtime/tests/repl_state_tests.rs \
  crates/atlas-runtime/tests/repl_types_tests.rs | sort -u
```

Write merged file. Verify:
```bash
cargo nextest run -p atlas-runtime --test repl
```

### Step 5: Rename regression file
```bash
git mv crates/atlas-runtime/tests/regression_suite.rs \
       crates/atlas-runtime/tests/regression.rs
```

Verify:
```bash
cargo nextest run -p atlas-runtime --test regression
```

### Step 6: Delete old files
```bash
git rm crates/atlas-runtime/tests/datetime_core_tests.rs \
       crates/atlas-runtime/tests/datetime_advanced_tests.rs \
       crates/atlas-runtime/tests/regex_core_tests.rs \
       crates/atlas-runtime/tests/regex_operations_tests.rs \
       crates/atlas-runtime/tests/path_tests.rs \
       crates/atlas-runtime/tests/fs_tests.rs \
       crates/atlas-runtime/tests/process_tests.rs \
       crates/atlas-runtime/tests/gzip_tests.rs \
       crates/atlas-runtime/tests/tar_tests.rs \
       crates/atlas-runtime/tests/zip_tests.rs \
       crates/atlas-runtime/tests/api_tests.rs \
       crates/atlas-runtime/tests/api_conversion_tests.rs \
       crates/atlas-runtime/tests/api_native_functions_tests.rs \
       crates/atlas-runtime/tests/api_sandboxing_tests.rs \
       crates/atlas-runtime/tests/reflection_tests.rs \
       crates/atlas-runtime/tests/json_value_tests.rs \
       crates/atlas-runtime/tests/runtime_api.rs \
       crates/atlas-runtime/tests/repl_state_tests.rs \
       crates/atlas-runtime/tests/repl_types_tests.rs
```

### Step 7: Final structure audit
```bash
ls crates/atlas-runtime/tests/*.rs
```

Expected 20 files (17 domain + regression + stdlib + typesystem — verify exact count):
`api.rs`, `async_runtime.rs`, `bytecode.rs`, `collections.rs`, `datetime_regex.rs`,
`debugger.rs`, `diagnostics.rs`, `ffi.rs`, `frontend_integration.rs`, `frontend_syntax.rs`,
`http.rs`, `interpreter.rs`, `modules.rs`, `regression.rs`, `repl.rs`,
`security.rs`, `stdlib.rs`, `system.rs`, `typesystem.rs`, `vm.rs`

### Step 8: Test count integrity check
```bash
# Compare to number recorded in Step 1
cargo nextest list -p atlas-runtime | wc -l
```

Count must match pre-consolidation count.

### Step 9: Final suite + timing
```bash
time cargo nextest run -p atlas-runtime
cargo clippy -p atlas-runtime -- -D warnings
```

Wall time must be under 20 seconds.

### Step 10: Binary count measurement
```bash
cargo build --tests -p atlas-runtime 2>/dev/null
ls target/debug/deps/atlas_runtime-*.d 2>/dev/null | wc -l
```

Document reduction vs baseline.

## Tests
No new tests. All existing tests must pass with identical count.

## Acceptance
- All 5 new files compile and tests pass
- `regression.rs` present (renamed from `regression_suite.rs`)
- All 19 old source files deleted via `git rm`
- Test count before == test count after
- Full suite green, wall time < 20s
- Zero clippy warnings
- Exactly 20 `*.rs` files in `tests/` (verify with `ls`)
- Commit: `refactor(tests): Complete test infrastructure consolidation (125 → ~20 binaries)`
- Update STATUS.md:
  - Mark Infra-03a, 03b, 03c complete
  - Infra: 5/9 complete
  - Next Phase: `phases/infra/phase-04-ignore-audit.md`
  - Last Updated: current date
