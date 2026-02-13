# Phase 13 - Related Spans

## Objective
Emit secondary/related spans for key diagnostics to aid AI tools.

## Inputs
- `docs/specification/diagnostic-system.md`

## Deliverables
- Related spans for:
  - Type mismatch (expected vs actual declaration)
  - Unknown symbol (suggested declaration sites, if any)
  - Redeclaration (original declaration)

## Steps
- Extend diagnostic builder to include related spans.
- Add tests for related spans in JSON output.

## Exit Criteria
- Related spans appear in diagnostics where applicable.
