# Phase 09 - Runtime API Scaffold

## Objective
Create the minimal runtime API surface for future embedding, without implementing full behavior.

## Inputs
- `docs/api/runtime-api.md`

## Deliverables
- `Atlas` runtime struct with `new`, `eval`, `eval_file` signatures.
- API lives in `atlas-runtime` and is re-exported.

## Steps
- Define `Atlas` type and stub methods.
- Return `Diagnostic` errors for unimplemented methods.

## Exit Criteria
- Runtime API compiles and is documented.
