# Phase 10 - Function Return Analysis

## Objective
Ensure functions with non-void returns always return a value on all paths.

## Inputs
- `Atlas-SPEC.md`
- `docs/specification/diagnostic-system.md`

## Deliverables
- Control-flow-aware return analysis.
- Tests for missing return in nested branches.

## Steps
- Implement return checking for `if/else` and loops.
- Add negative tests with `AT0004`.

## Exit Criteria
- All return analysis tests pass with correct diagnostics.
