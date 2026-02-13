# Phase 11 - Bytecode Versioning

## Objective
Introduce versioning for `.atb` bytecode files.

## Inputs
- `docs/versioning.md`

## Deliverables
- Bytecode header includes version field.
- Loader validates version and errors on mismatch.

## Steps
- Add version field to header.
- Add version checks in loader.

## Exit Criteria
- Version mismatch produces a clear diagnostic.
