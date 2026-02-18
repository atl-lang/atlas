# Phase Infra-03b: Test Consolidation — Debugger, Security, Modules

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Phase Infra-03a complete.

```bash
cargo nextest run -p atlas-runtime 2>&1 | tail -5
ls crates/atlas-runtime/tests/*.rs | wc -l  # should be ~39 after 03a
```

---

## Objective
Merge 10 source files into 3 domain files. These domains contain shared helper functions that must be deduplicated before merging.

## Source → Target Mapping

| Target | Sources | Notes |
|--------|---------|-------|
| `debugger.rs` | `debugger_execution_tests.rs`, `debugger_inspection_tests.rs`, `debugger_protocol_tests.rs` | Shared helpers — deduplicate |
| `security.rs` | `security_tests.rs`, `runtime_security_tests.rs`, `audit_logging_tests.rs` | Similar setup helpers |
| `modules.rs` | `module_binding_tests.rs`, `module_execution_tests.rs`, `module_execution_vm_tests.rs`, `module_resolution_tests.rs` | VM + interpreter variants |

---

## CRITICAL EXECUTION APPROACH

**DO NOT use the `Read` tool to read source files in full.**

Use Grep to scout structure, then python to merge:

```bash
# Scout for duplicate helpers before merging
grep -h "^fn \|^pub fn " \
  crates/atlas-runtime/tests/debugger_execution_tests.rs \
  crates/atlas-runtime/tests/debugger_inspection_tests.rs \
  crates/atlas-runtime/tests/debugger_protocol_tests.rs \
  | sort | uniq -d
```

```bash
# Scout imports
grep -h "^use " \
  crates/atlas-runtime/tests/debugger_execution_tests.rs \
  crates/atlas-runtime/tests/debugger_inspection_tests.rs \
  crates/atlas-runtime/tests/debugger_protocol_tests.rs \
  | sort -u
```

For each merge: collect unique imports, identify and keep one copy of duplicate helpers, concatenate test functions. The python merge script pattern from Infra-03a applies here too.

---

## Implementation Steps

### Step 1: Create `tests/debugger.rs`

```bash
# Find duplicate helpers
grep -h "^fn \|^pub fn \|^async fn " \
  crates/atlas-runtime/tests/debugger_execution_tests.rs \
  crates/atlas-runtime/tests/debugger_inspection_tests.rs \
  crates/atlas-runtime/tests/debugger_protocol_tests.rs \
  | sort | uniq -d

# Find all imports
grep -h "^use \|^extern crate " \
  crates/atlas-runtime/tests/debugger_execution_tests.rs \
  crates/atlas-runtime/tests/debugger_inspection_tests.rs \
  crates/atlas-runtime/tests/debugger_protocol_tests.rs \
  | sort -u
```

Write merged file (deduplicate shared helpers). Verify:
```bash
cargo nextest run -p atlas-runtime --test debugger
```

### Step 2: Create `tests/security.rs`

```bash
grep -h "^fn \|^pub fn \|^async fn " \
  crates/atlas-runtime/tests/security_tests.rs \
  crates/atlas-runtime/tests/runtime_security_tests.rs \
  crates/atlas-runtime/tests/audit_logging_tests.rs \
  | sort | uniq -d

grep -h "^use \|^extern crate " \
  crates/atlas-runtime/tests/security_tests.rs \
  crates/atlas-runtime/tests/runtime_security_tests.rs \
  crates/atlas-runtime/tests/audit_logging_tests.rs \
  | sort -u
```

Write merged file. Verify:
```bash
cargo nextest run -p atlas-runtime --test security
```

### Step 3: Create `tests/modules.rs`

The module files include both interpreter and VM variants — both must be kept:
```bash
grep -c "#\[test\]" \
  crates/atlas-runtime/tests/module_binding_tests.rs \
  crates/atlas-runtime/tests/module_execution_tests.rs \
  crates/atlas-runtime/tests/module_execution_vm_tests.rs \
  crates/atlas-runtime/tests/module_resolution_tests.rs

grep -h "^fn \|^pub fn \|^async fn " \
  crates/atlas-runtime/tests/module_binding_tests.rs \
  crates/atlas-runtime/tests/module_execution_tests.rs \
  crates/atlas-runtime/tests/module_execution_vm_tests.rs \
  crates/atlas-runtime/tests/module_resolution_tests.rs \
  | sort | uniq -d
```

Write merged file. Verify:
```bash
cargo nextest run -p atlas-runtime --test modules
```

### Step 4: Delete old files
```bash
git rm crates/atlas-runtime/tests/debugger_execution_tests.rs \
       crates/atlas-runtime/tests/debugger_inspection_tests.rs \
       crates/atlas-runtime/tests/debugger_protocol_tests.rs \
       crates/atlas-runtime/tests/security_tests.rs \
       crates/atlas-runtime/tests/runtime_security_tests.rs \
       crates/atlas-runtime/tests/audit_logging_tests.rs \
       crates/atlas-runtime/tests/module_binding_tests.rs \
       crates/atlas-runtime/tests/module_execution_tests.rs \
       crates/atlas-runtime/tests/module_execution_vm_tests.rs \
       crates/atlas-runtime/tests/module_resolution_tests.rs
```

### Step 5: Verify
```bash
cargo nextest run -p atlas-runtime --test debugger
cargo nextest run -p atlas-runtime --test security
cargo nextest run -p atlas-runtime --test modules
cargo clippy -p atlas-runtime -- -D warnings
```

## Tests
No new tests. All existing tests must pass with identical count.

## Acceptance
- `debugger.rs`, `security.rs`, `modules.rs` compile and tests pass
- All 10 old files deleted via `git rm`
- No duplicate helper functions in merged files
- Zero clippy warnings
- Commit: `refactor(tests): Infra-03b — merge debugger/security/modules test files`
- Update STATUS.md: mark Infra-03b complete, Next Phase → Infra-03c
