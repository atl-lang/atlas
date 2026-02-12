# Phase 08 - Prelude Tests

## Objective
Add tests to validate prelude availability and shadowing rules.

## Inputs
- `docs/prelude.md`
- `docs/prelude-test-plan.md`

## Deliverables
- Golden tests for prelude availability.
- Tests for allowed shadowing in nested scopes.
- Tests for disallowed shadowing in global scope.

## Steps
- Add `.atl` fixtures and expected diagnostics.
- Verify test outputs are deterministic.

## Exit Criteria
- Prelude tests pass with correct diagnostics.
