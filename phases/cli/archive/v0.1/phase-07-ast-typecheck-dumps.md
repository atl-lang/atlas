# Phase 07 - AST and Typecheck Dumps

## Objective
Implement AI-friendly JSON dumps for AST and typechecker output.

## Inputs
- `docs/specification/json-formats.md`
- `docs/specification/json-formats.md`

## Deliverables
- `atlas ast file.atl --json` outputs AST JSON.
- `atlas typecheck file.atl --json` outputs symbol/type JSON.

## Steps
- Implement AST serializer with stable ordering.
- Implement typecheck dump serializer.
- Add golden tests for JSON output.

## Exit Criteria
- Dumps are deterministic and match schemas.
