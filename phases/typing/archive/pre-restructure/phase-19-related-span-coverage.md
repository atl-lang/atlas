# Phase 19 - Related Span Coverage

## Objective
Ensure key diagnostics include related spans consistently.

## Inputs
- `docs/specification/diagnostic-system.md`

## Deliverables
- Tests for related spans in:
  - Type mismatch
  - Unknown symbol
  - Redeclaration

## Steps
- Add fixtures that exercise related spans.
- Validate JSON output includes `related` entries.

## Exit Criteria
- Related span tests pass.
