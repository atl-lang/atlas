# Phase 07 - Lexer Edge Cases

## Objective
Harden lexer behavior for edge cases and ambiguous inputs.

## Inputs
- `Atlas-SPEC.md`
- `docs/specification/diagnostic-system.md`

## Deliverables
- Tests for unterminated strings, invalid escapes, and unexpected characters.
- Diagnostics with precise spans for lexing errors.
- Map lexer errors to `AT1001` (invalid token), `AT1002` (unterminated string), `AT1003` (invalid escape).

## Steps
- Enumerate invalid inputs.
- Add golden tests under `tests/errors/`.
- Ensure lexer reports the first error cleanly.

## Exit Criteria
- Lexer edge-case tests pass with stable diagnostics.
