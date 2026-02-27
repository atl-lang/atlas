# Phase 03: API + Debugger Test Files Split

**Block:** test-split (Maintenance)
**Track:** 1 — direct push to main with [skip ci] after local verification
**Depends on:** Phase 02 complete

## Goal

Split `api.rs` (92KB) and `debugger.rs` (80KB) into subdirectory routers.

## Split Strategy

### `tests/api.rs` → `tests/api/`

`api.rs` tests the public `Atlas` struct API. Read the file and split by API surface:

| New file | Content |
|----------|---------|
| `api/eval.rs` | `Atlas::eval()` basic execution tests |
| `api/errors.rs` | Error propagation, diagnostic collection |
| `api/config.rs` | `Atlas::new_with_security()`, config variants |
| `api/typecheck.rs` | `Atlas::typecheck()`, `TypecheckDump` tests |
| `api/repl_integration.rs` | REPL-mode eval, multi-statement sessions |
| `api/version.rs` | Version assertions, schema version tests |

Adjust based on actual content. Target ≤ 15KB per file.

### `tests/debugger.rs` → `tests/debugger/`

`debugger.rs` tests the breakpoint/stepping/source-mapping system:

| New file | Content |
|----------|---------|
| `debugger/breakpoints.rs` | Breakpoint set/clear/hit tests |
| `debugger/stepping.rs` | Step-over, step-into, step-out |
| `debugger/source_map.rs` | Source location mapping |
| `debugger/watch.rs` | Variable watch, value inspection |
| `debugger/integration.rs` | Full debug session flows |

Adjust based on actual content. Target ≤ 15KB per file.

## Router Pattern

Same as Phase 02 — existing `.rs` becomes thin router with helpers + `#[path]` declarations.

## Update CLAUDE.md

Add `api/` and `debugger/` subdirectory entries to the test table in
`crates/atlas-runtime/src/CLAUDE.md`. Remove them from any "single-file" references.

## Verification

```bash
du -sh crates/atlas-runtime/tests/api/*.rs       # all < 20KB
du -sh crates/atlas-runtime/tests/debugger/*.rs  # all < 20KB
cargo nextest run -p atlas-runtime --test api
cargo nextest run -p atlas-runtime --test debugger
```

## Acceptance Criteria

- [ ] `api.rs` is a thin router (≤ 5KB)
- [ ] `debugger.rs` is a thin router (≤ 5KB)
- [ ] All split files ≤ 20KB
- [ ] Both test binaries pass with identical test count to pre-split
- [ ] CLAUDE.md table updated
