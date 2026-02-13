# Phase 08 - AST Dump Versioning

## Objective
Stabilize the JSON schema version for AST dumps.

## Inputs
- `docs/specification/json-formats.md`

## Deliverables
- `ast_version` field enforced in outputs.
- Version mismatch tests for future-proofing.

## Steps
- Add version field to AST JSON output.
- Add tests that assert version value.

## Exit Criteria
- AST dumps include version field.
