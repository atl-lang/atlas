# Phase 04 - Parser Errors

## Objective
Define and implement consistent parser error reporting.

## Inputs
- `docs/diagnostics.md`

## Deliverables
- Error messages aligned with `docs/diagnostics.md`.
- Span calculation for common syntax failures.
- Map parser errors to `AT1000` (syntax error).

## Steps
- Enumerate expected parse errors.
- Map errors to codes and messages.
- Add golden tests for parse failures.
- Add tests that function declarations inside blocks are rejected.
- Add tests that `import` is rejected in v0.1.

## Exit Criteria
- Parser error tests produce stable diagnostics.
