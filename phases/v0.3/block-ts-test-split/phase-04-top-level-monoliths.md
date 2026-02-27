# Phase 04: Top-Level Monolith Splits (Group B)

**Block:** test-split (Maintenance)
**Track:** 1 — direct push to main with [skip ci] after local verification
**Depends on:** Phase 03 complete

## Goal

Split the remaining top-level test file violations. Ten files, all currently standalone with
no subdirectory. Each becomes a thin router + subdirectory.

## Files

| File | Size | New subdirectory |
|------|------|-----------------|
| `diagnostics.rs` | 68KB | `diagnostics/` |
| `async_runtime.rs` | 68KB | `async_runtime/` |
| `ffi.rs` | 64KB | `ffi/` |
| `collections.rs` | 64KB | `collections/` |
| `modules.rs` | 56KB | `modules/` |
| `datetime_regex.rs` | 52KB | `datetime_regex/` |
| `security.rs` | 48KB | `security/` |
| `closures.rs` | 48KB | `closures/` |
| `pattern_matching.rs` | 40KB | `pattern_matching/` |
| `regression.rs` | 40KB | `regression/` |

## Split Strategy Per Domain

Read each file before splitting. Suggested domain groupings:

**`diagnostics.rs`** — split by diagnostic category:
`diagnostics/type_errors.rs`, `diagnostics/parse_errors.rs`, `diagnostics/runtime_errors.rs`,
`diagnostics/warnings.rs`, `diagnostics/spans.rs`

**`async_runtime.rs`** — split by async primitive:
`async_runtime/futures.rs`, `async_runtime/channels.rs`, `async_runtime/tasks.rs`,
`async_runtime/integration.rs`

**`ffi.rs`** — split by FFI surface:
`ffi/bindings.rs`, `ffi/types.rs`, `ffi/safety.rs`, `ffi/integration.rs`

**`collections.rs`** — split by collection type:
`collections/hash_map.rs`, `collections/set.rs`, `collections/queue.rs`,
`collections/cow.rs`, `collections/integration.rs`

**`modules.rs`** — split by module feature:
`modules/import.rs`, `modules/export.rs`, `modules/resolution.rs`, `modules/integration.rs`

**`datetime_regex.rs`** — split by stdlib domain:
`datetime_regex/datetime.rs`, `datetime_regex/regex.rs`

**`security.rs`** — split by permission domain:
`security/permissions.rs`, `security/sandbox.rs`, `security/fs_access.rs`,
`security/integration.rs`

**`closures.rs`** — split by closure feature:
`closures/capture.rs`, `closures/anon_fn.rs`, `closures/hof.rs`, `closures/ownership.rs`,
`closures/integration.rs`

**`pattern_matching.rs`** — split by pattern type:
`pattern_matching/literals.rs`, `pattern_matching/destructure.rs`, `pattern_matching/guards.rs`,
`pattern_matching/integration.rs`

**`regression.rs`** — split by bug era or feature area:
`regression/parser.rs`, `regression/typechecker.rs`, `regression/runtime.rs`,
`regression/stdlib.rs`

All suggested groupings are based on typical content for these domains. Read each file
first and adjust groupings so every output file lands ≤ 15KB.

## Approach

Do not attempt all 10 files in one pass. Work sequentially:
1. Read file → identify section boundaries → design split
2. Write router + subdir files
3. Verify with `cargo nextest run -p atlas-runtime --test <domain>`
4. Move to next file

## Update CLAUDE.md

Add all 10 new subdirectories to the test table. Update the testing rule file to reflect
that none of these are single-file domains anymore.

## Verification

```bash
# After all 10 splits:
find crates/atlas-runtime/tests -name "*.rs" -size +40k -not -path "*/target/*" | xargs du -sh
# Output should be empty (no hard violations remaining at top level)

cargo nextest run -p atlas-runtime  # full suite green
```

## Acceptance Criteria

- [ ] All 10 files converted to thin routers (≤ 5KB each)
- [ ] All split files ≤ 20KB
- [ ] No hard violations (> 40KB) remain in `tests/` top level
- [ ] Full test suite passes with identical test count
- [ ] CLAUDE.md and atlas-testing.md tables updated
