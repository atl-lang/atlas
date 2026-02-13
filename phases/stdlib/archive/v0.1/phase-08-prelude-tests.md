# Phase 08 - Prelude Tests

## Objective
Add tests to validate prelude availability and shadowing rules.

## Inputs
- `Atlas-SPEC.md` (for correct Atlas syntax in test code)
- `docs/RUNTIME.md`
- `archive/test-plans/prelude-test-plan.md` (archived test plan)

## Deliverables
- Golden tests for prelude availability.
- Tests for allowed shadowing in nested scopes.
- Tests for disallowed shadowing in global scope.

## Steps
- Add `.atl` fixtures and expected diagnostics.
- Verify test outputs are deterministic.

## Exit Criteria
- Prelude tests pass with correct diagnostics.
