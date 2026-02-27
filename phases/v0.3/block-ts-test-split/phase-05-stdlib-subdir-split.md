# Phase 05: stdlib Subdirectory Further Splits

**Block:** test-split (Maintenance)
**Track:** 1 — direct push to main with [skip ci] after local verification
**Depends on:** Phase 04 complete

## Goal

The `tests/stdlib/` subdirectory already exists but four files within it are still over the
40KB hard cap. These need further splitting into nested subdirectories or additional sibling
files within `tests/stdlib/`.

## Files

| File | Size |
|------|------|
| `stdlib/real_world.rs` | 84KB |
| `stdlib/vm_stdlib.rs` | 80KB |
| `stdlib/integration.rs` | 52KB |
| `stdlib/functions.rs` | 40KB |

## Split Strategy

### `stdlib/real_world.rs` (84KB)

Real-world scenario tests covering multiple stdlib areas together. Split by scenario category:

`stdlib/real_world/data_processing.rs` — array/map/filter pipelines
`stdlib/real_world/string_manipulation.rs` — text processing scenarios
`stdlib/real_world/json_workflows.rs` — JSON parse/transform/serialize scenarios
`stdlib/real_world/math_programs.rs` — numeric computation scenarios
`stdlib/real_world/io_workflows.rs` — file I/O scenarios
`stdlib/real_world/integration.rs` — cross-domain real-world programs

### `stdlib/vm_stdlib.rs` (80KB)

VM-executed stdlib tests. Split by stdlib module:

`stdlib/vm_stdlib/strings.rs` — string function VM tests
`stdlib/vm_stdlib/collections.rs` — array/map VM tests
`stdlib/vm_stdlib/math.rs` — math function VM tests
`stdlib/vm_stdlib/io.rs` — I/O VM tests
`stdlib/vm_stdlib/types.rs` — type conversion VM tests
`stdlib/vm_stdlib/json.rs` — JSON VM tests
`stdlib/vm_stdlib/integration.rs` — cross-module VM tests

### `stdlib/integration.rs` (52KB)

`stdlib/integration/core.rs`, `stdlib/integration/cross_module.rs`,
`stdlib/integration/error_handling.rs`, `stdlib/integration/parity.rs`

### `stdlib/functions.rs` (40KB)

`stdlib/functions/higher_order.rs`, `stdlib/functions/closures.rs`,
`stdlib/functions/recursion.rs`, `stdlib/functions/builtins.rs`

## Router Updates

`tests/stdlib.rs` (the top-level router) already uses `#[path]` declarations. Update it to
point to the new nested files. For `real_world` and `vm_stdlib`, add a nested router file
`tests/stdlib/real_world.rs` and `tests/stdlib/vm_stdlib.rs` that fan out to the next level.

## Verification

```bash
du -sh crates/atlas-runtime/tests/stdlib/**/*.rs   # all < 20KB
cargo nextest run -p atlas-runtime --test stdlib
```

## Acceptance Criteria

- [ ] `stdlib/real_world.rs` split — all output files ≤ 20KB
- [ ] `stdlib/vm_stdlib.rs` split — all output files ≤ 20KB
- [ ] `stdlib/integration.rs` split — all output files ≤ 20KB
- [ ] `stdlib/functions.rs` split — all output files ≤ 20KB
- [ ] `cargo nextest run -p atlas-runtime --test stdlib` — all tests pass, count unchanged
- [ ] `tests/stdlib.rs` router updated
