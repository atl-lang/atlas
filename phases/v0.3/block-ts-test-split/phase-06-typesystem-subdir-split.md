# Phase 06: typesystem Subdirectory Further Splits

**Block:** test-split (Maintenance)
**Track:** 1 — direct push to main with [skip ci] after local verification
**Depends on:** Phase 05 complete

## Goal

The `tests/typesystem/` subdirectory already exists but four files within it exceed the
40KB hard cap. Two more are in the warning zone and should be addressed here too.

## Files

| File | Size | Status |
|------|------|--------|
| `typesystem/integration.rs` | 56KB | Hard violation |
| `typesystem/inference.rs` | 52KB | Hard violation |
| `typesystem/flow.rs` | 36KB | Warning zone |
| `typesystem/generics.rs` | 36KB | Warning zone |
| `typesystem/constraints.rs` | 24KB | Warning zone |
| `typesystem/bindings.rs` | 24KB | Warning zone |

## Split Strategy

### `typesystem/integration.rs` (56KB)

Full-pipeline type system tests. Split by feature intersection:

`typesystem/integration/inference_generics.rs` — inference meets generics
`typesystem/integration/flow_analysis.rs` — flow-sensitive type narrowing integration
`typesystem/integration/ownership.rs` — ownership type integration
`typesystem/integration/cross_module.rs` — cross-module type resolution
`typesystem/integration/regression.rs` — integration regression cases

### `typesystem/inference.rs` (52KB)

`typesystem/inference/locals.rs` — local variable type inference
`typesystem/inference/returns.rs` — return type inference (AT3050/51/52)
`typesystem/inference/expressions.rs` — expression-level inference
`typesystem/inference/let_polymorphism.rs` — let-polymorphism cases
`typesystem/inference/bidirectional.rs` — bidirectional checking

### `typesystem/flow.rs` (36KB)

`typesystem/flow/narrowing.rs` — type narrowing in branches
`typesystem/flow/null_safety.rs` — null/void flow
`typesystem/flow/assignment.rs` — assignment flow
`typesystem/flow/loops.rs` — loop flow analysis

### `typesystem/generics.rs` (36KB)

`typesystem/generics/monomorphization.rs` — generic instantiation
`typesystem/generics/constraints.rs` — generic constraints/bounds
`typesystem/generics/inference.rs` — generic type argument inference
`typesystem/generics/errors.rs` — generic error cases

### `typesystem/constraints.rs` and `typesystem/bindings.rs`

These are 24KB each — in the warning zone. If reading them reveals they can be split cleanly,
do it. If the content is already tightly cohesive, leave as-is and note it. Do not force a
split that creates artificial boundaries.

## Router Updates

`tests/typesystem.rs` already uses `#[path]` declarations. For files that get a nested split,
add intermediate router files (`tests/typesystem/integration.rs` → fans to
`tests/typesystem/integration/*.rs`).

## Verification

```bash
du -sh crates/atlas-runtime/tests/typesystem/**/*.rs   # all < 20KB
cargo nextest run -p atlas-runtime --test typesystem
```

## Acceptance Criteria

- [ ] `typesystem/integration.rs` split — output files ≤ 20KB
- [ ] `typesystem/inference.rs` split — output files ≤ 20KB
- [ ] `typesystem/flow.rs` split (if content warrants) — output files ≤ 20KB
- [ ] `typesystem/generics.rs` split (if content warrants) — output files ≤ 20KB
- [ ] `cargo nextest run -p atlas-runtime --test typesystem` — all tests pass, count unchanged
- [ ] `tests/typesystem.rs` router updated
