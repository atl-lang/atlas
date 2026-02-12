# Phase 05 - Type Rules Tests

## Objective
Expand type checker coverage with operator and array rules.

## Inputs
- `Atlas-SPEC.md`

## Deliverables
- Tests for operator typing (`+`, `-`, `*`, `/`, `%`, `==`, `!=`, `&&`, `||`).
- Tests for array literal typing and indexing.
- Tests for string concatenation (`string + string` only).
- Tests for array element assignment type rules.
- Tests that comparisons (`<`, `<=`, `>`, `>=`) only allow `number`.

## Steps
- Add positive and negative tests for each operator.
- Verify diagnostics for invalid operations.

## Exit Criteria
- All tests pass and match diagnostics policy.
