# Phase 15 - Warning Tests

## Objective
Add tests to validate warning behavior and codes.

## Inputs
- `docs/specification/diagnostic-system.md`
- `docs/warnings-test-plan.md`

## Deliverables
- Golden tests for `AT2001` and `AT2002` warnings.
- Tests for warning ordering with errors.
- Test that errors (e.g., `AT1012`) suppress or precede warnings as specified.
- Tests for `_`-prefixed identifiers not producing warnings.

## Steps
- Add `.atl` fixtures and expected diagnostics.
- Ensure warnings do not block execution.
- Add fixture with illegal prelude shadowing + unused variable and verify ordering.

## Exit Criteria
- Warning tests pass with correct output.
