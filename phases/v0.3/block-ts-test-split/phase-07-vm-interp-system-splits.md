# Phase 07: VM + Interpreter + System Subdirectory Splits

**Block:** test-split (Maintenance)
**Track:** 1 — direct push to main with [skip ci] after local verification
**Depends on:** Phase 06 complete

## Goal

Three existing subdirectories (`tests/vm/`, `tests/interpreter/`, `tests/system/`) each contain
files over the 40KB hard cap. Also address warning-zone files in these subdirs.

## Files

| File | Size | Status |
|------|------|--------|
| `system/compression.rs` | 80KB | Hard violation |
| `interpreter/integration.rs` | 76KB | Hard violation |
| `interpreter/nested_functions.rs` | 32KB | Warning zone |
| `vm/for_in.rs` | 36KB | Warning zone |
| `vm/integration.rs` | 32KB | Warning zone |
| `vm/functions.rs` | 28KB | Warning zone |
| `vm/complex_programs.rs` | 28KB | Warning zone |
| `vm/regression.rs` | 24KB | Warning zone |

## Split Strategy

### `system/compression.rs` (80KB) — Hard violation

Tests for gzip, tar, and zip. These are already three distinct domains crammed in one file:

`system/compression/gzip.rs` — gzip compress/decompress tests
`system/compression/tar.rs` — tar archive tests
`system/compression/zip.rs` — zip archive tests
`system/compression/integration.rs` — cross-format scenarios

### `interpreter/integration.rs` (76KB) — Hard violation

Full interpreter integration scenarios. Split by language feature area:

`interpreter/integration/arithmetic.rs`
`interpreter/integration/control_flow.rs`
`interpreter/integration/functions.rs`
`interpreter/integration/closures.rs`
`interpreter/integration/collections.rs`
`interpreter/integration/stdlib.rs`
`interpreter/integration/types.rs`
`interpreter/integration/errors.rs`

### VM files (warning zone)

`vm/for_in.rs` (36KB), `vm/integration.rs` (32KB), `vm/functions.rs` (28KB),
`vm/complex_programs.rs` (28KB), `vm/regression.rs` (24KB):

Read each file. If natural section boundaries exist (e.g. `vm/for_in.rs` covers both
array iteration and map iteration), split. If the content is cohesive and under 30KB,
leave with a note — do not force splits that create noise. Priority is the two hard violations.

### `interpreter/nested_functions.rs` (32KB) — Warning zone

If content naturally segments (e.g. simple nesting vs closures vs recursion), split.
Otherwise leave with a note.

## Router Updates

`tests/system.rs` uses `#[path]` — update to point `system_compression` at the new router.
Add `tests/system/compression.rs` as an intermediate router.

`tests/interpreter.rs` uses `#[path]` — update `interp_integration` to point at the new router.
Add `tests/interpreter/integration.rs` as an intermediate router.

`tests/vm.rs` uses `#[path]` — update any split vm files similarly.

## Verification

```bash
du -sh crates/atlas-runtime/tests/system/*.rs crates/atlas-runtime/tests/system/**/*.rs 2>/dev/null | sort -rh | head -20
du -sh crates/atlas-runtime/tests/interpreter/*.rs crates/atlas-runtime/tests/interpreter/**/*.rs 2>/dev/null | sort -rh | head -20
du -sh crates/atlas-runtime/tests/vm/*.rs crates/atlas-runtime/tests/vm/**/*.rs 2>/dev/null | sort -rh | head -20

cargo nextest run -p atlas-runtime --test system
cargo nextest run -p atlas-runtime --test interpreter
cargo nextest run -p atlas-runtime --test vm
```

## Acceptance Criteria

- [ ] `system/compression.rs` split — all output files ≤ 20KB
- [ ] `interpreter/integration.rs` split — all output files ≤ 20KB
- [ ] Warning-zone files assessed — split where natural boundaries exist
- [ ] No hard violations (> 40KB) remain in `tests/vm/`, `tests/interpreter/`, `tests/system/`
- [ ] All three test binaries pass with identical test counts
- [ ] Router files updated
