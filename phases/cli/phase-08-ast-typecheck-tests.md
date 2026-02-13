# Phase 08 - AST & Typecheck Dump Tests

## Objective
Add tests to validate AST and typecheck dump JSON outputs.

## Inputs
- `Atlas-SPEC.md` (for correct Atlas syntax in .atl fixtures)
- `docs/ast-dump.md`
- `docs/typecheck-dump.md`
- `archive/test-plans/ast-typecheck-tests.md` (archived test plan)

## Deliverables
- Golden tests for `atlas ast --json`.
- Golden tests for `atlas typecheck --json`.

## Steps
- Add `.atl` fixtures and expected `.json` outputs.
- Validate deterministic field ordering.

## Exit Criteria
- Dump tests pass across machines.
