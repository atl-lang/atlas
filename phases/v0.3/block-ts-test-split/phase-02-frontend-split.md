# Phase 02: Frontend Test Files Split

**Block:** test-split (Maintenance)
**Track:** 1 — direct push to main with [skip ci] after local verification
**Depends on:** Phase 01 complete

## Goal

Split `frontend_syntax.rs` (100KB) and `frontend_integration.rs` (100KB) into subdirectory
routers. Both are currently listed as "single-file domains" in CLAUDE.md — at 100KB each,
that designation is no longer valid.

## Split Strategy

### `tests/frontend_syntax.rs` → `tests/frontend_syntax/`

Read the file and group tests by what language feature they cover. Likely natural splits:

| New file | Content |
|----------|---------|
| `frontend_syntax/literals.rs` | Number, string, bool, null literals |
| `frontend_syntax/operators.rs` | Arithmetic, comparison, logical, bitwise operators |
| `frontend_syntax/control_flow.rs` | if/else, while, for, break, continue, return |
| `frontend_syntax/functions.rs` | Function declarations, calls, params, return types |
| `frontend_syntax/types.rs` | Type annotations, ownership annotations, type params |
| `frontend_syntax/patterns.rs` | Pattern matching syntax |
| `frontend_syntax/expressions.rs` | Complex expressions, closures, anon fns |
| `frontend_syntax/errors.rs` | Syntax error cases, malformed input |

Verify actual section boundaries by reading the file before splitting. Adjust groupings to
keep each output file under 20KB. The goal is ≤ 15KB per file.

### `tests/frontend_integration.rs` → `tests/frontend_integration/`

Similar approach — read the file and split by pipeline stage or feature area:

| New file | Content |
|----------|---------|
| `frontend_integration/pipeline.rs` | Full parse→typecheck→eval pipeline tests |
| `frontend_integration/declarations.rs` | Variable, function, type declarations |
| `frontend_integration/control_flow.rs` | Control flow integration |
| `frontend_integration/functions.rs` | Function call integration, HOFs |
| `frontend_integration/types.rs` | Type checking integration |
| `frontend_integration/modules.rs` | Module/import integration |
| `frontend_integration/errors.rs` | Error propagation through pipeline |

Adjust based on actual file content. Files ≤ 15KB each.

## Router Pattern

Each `.rs` root file becomes a thin router:

```rust
//! THIN ROUTER — DO NOT ADD TESTS HERE.
//! Add tests to the submodule files: tests/frontend_syntax/{literals,operators,...}.rs

// Shared helpers used by submodules via `use super::*`
// ... (move any helper functions here) ...

#[path = "frontend_syntax/literals.rs"]
mod literals;
// etc.
```

## Update CLAUDE.md

Update `crates/atlas-runtime/src/CLAUDE.md` test table — remove `frontend_syntax.rs` and
`frontend_integration.rs` from "Single-file domains" and add them to the subdirectory table.

Update `atlas-testing.md` domain table to match.

## Verification

```bash
du -sh crates/atlas-runtime/tests/frontend_syntax/*.rs   # all < 20KB
du -sh crates/atlas-runtime/tests/frontend_integration/*.rs  # all < 20KB
cargo nextest run -p atlas-runtime --test frontend_syntax
cargo nextest run -p atlas-runtime --test frontend_integration
```

## Acceptance Criteria

- [ ] `frontend_syntax.rs` is a thin router (helpers + `#[path]` mods only, ≤ 5KB)
- [ ] `frontend_integration.rs` is a thin router (≤ 5KB)
- [ ] All split files ≤ 20KB
- [ ] Both test binaries pass with identical test count to pre-split
- [ ] CLAUDE.md and atlas-testing.md tables updated
