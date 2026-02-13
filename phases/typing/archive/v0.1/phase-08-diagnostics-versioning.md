# Phase 08 - Diagnostics Versioning

## Objective
Introduce explicit diagnostics versioning to protect golden tests.

## Inputs
- `docs/specification/diagnostic-system.md`

## Deliverables
- Diagnostic version identifier (e.g., `diag_version: 1`).
- JSON diagnostics include version field.

## Steps
- Add version field to diagnostic schema.
- Update JSON renderer and tests.

## Exit Criteria
- Version present in all JSON diagnostics.
