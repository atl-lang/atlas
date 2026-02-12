# Phase 11 - Typecheck Dump Versioning

## Objective
Stabilize the JSON schema version for typecheck dumps.

## Inputs
- `docs/typecheck-dump.md`

## Deliverables
- `typecheck_version` field enforced in outputs.
- Version mismatch tests for future-proofing.

## Steps
- Add version field to typecheck JSON output.
- Add tests that assert version value.

## Exit Criteria
- Typecheck dumps include version field.
