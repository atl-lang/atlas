# Phase 07 - Prelude Binding

## Objective
Ensure prelude built-ins are always in scope.

## Inputs
- `docs/RUNTIME.md`

## Deliverables
- Prelude symbols injected into the global scope.
- Tests verifying prelude availability without imports.
- Diagnostics for illegal global shadowing (`AT1012`).

## Steps
- Add prelude symbols during binding.
- Prevent prelude overrides in global scope.

## Exit Criteria
- Prelude tests pass.
