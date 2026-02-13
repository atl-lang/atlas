# Phase 6 - Warnings

## Objective
Implement and test warning diagnostics for unused variables and unreachable code.

## Inputs
- `docs/specification/diagnostic-system.md`
- `docs/specification/diagnostic-system.md`
- `archive/test-plans/warnings-test-plan.md` (archived test plan)

## Deliverables

### Implementation
- Warning diagnostics for unused variables (`AT2001`)
- Warning diagnostics for unreachable code (`AT2002`)
- Suppression of unused warnings for `_`-prefixed identifiers
- Variable usage tracking in binder/typechecker
- Unreachable code detection after `return`

### Tests
- Golden tests for `AT2001` and `AT2002` warnings
- Tests for warning ordering with errors
- Tests that errors (e.g., `AT1012`) suppress or precede warnings as specified
- Tests for `_`-prefixed identifiers not producing warnings
- Tests that warnings do not block execution
- Fixture with illegal prelude shadowing + unused variable to verify ordering

## Steps
1. Track variable usage in binder/typechecker
2. Detect unreachable code after `return` statements
3. Add `.atl` test fixtures and expected diagnostics
4. Ensure warnings do not block execution
5. Verify warning ordering relative to errors

## Exit Criteria
- [ ] Warning implementation complete for unused variables and unreachable code
- [ ] All warning tests pass with correct diagnostic codes
- [ ] Warning ordering verified relative to errors
- [ ] `_`-prefix suppression working correctly
