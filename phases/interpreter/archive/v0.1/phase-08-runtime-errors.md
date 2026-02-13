# Phase 08 - Runtime Errors

## Objective
Implement consistent runtime error propagation with stack traces.

## Inputs
- `Atlas-SPEC.md` (for correct Atlas test syntax)
- `docs/specification/diagnostic-system.md`
- `docs/RUNTIME.md`

## Deliverables
- Runtime error mapping to diagnostic codes.
- Stack trace formatting aligned with spec.

## Steps
- Standardize error creation helpers.
- Attach span and call stack info.
- Add error tests (divide by zero, bounds, invalid index, null use).

## Exit Criteria
- Runtime error tests match expected outputs.
