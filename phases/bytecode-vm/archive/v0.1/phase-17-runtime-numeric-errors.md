# Phase 17 - VM Numeric Error Propagation

## Objective
Ensure VM mirrors interpreter numeric error behavior.

## Inputs
- `docs/specification/language-semantics.md`
- `docs/RUNTIME.md`

## Deliverables
- VM emits `AT0005` and `AT0007` correctly.
- Error span mapping uses debug info.

## Steps
- Implement numeric checks in VM execution.
- Add tests to match interpreter behavior.

## Exit Criteria
- VM numeric error tests pass.
