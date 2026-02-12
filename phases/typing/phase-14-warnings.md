# Phase 14 - Warnings

## Objective
Implement warnings for unused variables and unreachable code.

## Inputs
- `docs/warnings.md`
- `docs/diagnostics.md`

## Deliverables
- Warning diagnostics for unused variables (`AT2001`).
- Warning diagnostics for unreachable code (`AT2002`).
- Suppression of unused warnings for `_`-prefixed identifiers.

## Steps
- Track variable usage in binder/typechecker.
- Detect unreachable code after `return`.

## Exit Criteria
- Warning tests pass and emit correct codes.
