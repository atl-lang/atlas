# Phase 01: Router Cleanup + Orphaned Directory Removal

**Block:** test-split (Maintenance)
**Track:** 1 — direct push to main with [skip ci] after local verification
**Depends on:** Nothing — standalone cleanup

## Goal

Fix leftover debris from the previous test refactor: one duplicate test in a router file,
dead section comments across three routers, and two orphaned directories that declare modules
whose child files never existed and that no test binary includes.

## Problems to Fix

### 1. Duplicate test in `tests/interpreter.rs`

The router contains a live `#[rstest]` block (`test_json_as_string` and siblings) that also
exists in `tests/interpreter/member.rs`. The router is supposed to be helpers + `#[path]`
declarations only. Remove the test block from the router.

Lines to remove: the block beginning at `// JSON as_string() Tests` down through
the last `fn test_json_as_bool_error()` case. Keep the `run_interpreter` helper above it —
it is used by the submodules.

### 2. Dead section comments in routers

Three routers have `// From old_file_name.rs` section headers referencing file names that
no longer exist. These are noise that future agents might try to interpret as structure.

| File | Dead comment to remove |
|------|----------------------|
| `tests/vm.rs` | `// From vm_integration_tests.rs` |
| `tests/typesystem.rs` | `// From advanced_inference_tests.rs` |
| `tests/interpreter.rs` | `// From interpreter_member_tests.rs` |

### 3. Orphaned directories

`tests/integration/interpreter/mod.rs` references `mod arithmetic`, `mod arrays`,
`mod control_flow`, `mod functions`, `mod logical`, `mod strings` — none of those files exist.
`tests/unit/mod.rs` references `pub mod frontend` which is also empty stubs.
Neither directory is included by any root test binary.

**Action:** Delete both directories entirely.
```
tests/integration/   → delete
tests/unit/          → delete
```

## Verification

```bash
# Confirm no test references these dirs
grep -r "integration\|unit" crates/atlas-runtime/tests/*.rs | grep "mod \|path ="
# Should show no references to tests/integration or tests/unit

# Run full test suite — must be green
cargo nextest run -p atlas-runtime
```

## Acceptance Criteria

- [ ] `tests/interpreter.rs` has no `#[test]` or `#[rstest]` attributes (helpers only + `#[path]` mods)
- [ ] Dead `// From *.rs` comments removed from all three routers
- [ ] `tests/integration/` directory deleted
- [ ] `tests/unit/` directory deleted
- [ ] `cargo nextest run -p atlas-runtime` — all tests pass, count unchanged
