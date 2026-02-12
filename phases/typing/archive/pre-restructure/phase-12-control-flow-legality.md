# Phase 12 - Control Flow Legality

## Objective
Enforce legality of `break`, `continue`, and `return` positions.

## Inputs
- `Atlas-SPEC.md`
- `docs/diagnostics.md`

## Deliverables
- Compile-time errors for `break/continue` outside loops.
- Compile-time errors for `return` outside functions.

## Steps
- Track loop and function context during binding/typechecking.
- Emit diagnostics with related span where possible.

## Exit Criteria
- Control-flow legality tests pass.
