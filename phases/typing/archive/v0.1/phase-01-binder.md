# Phase 01 - Binder

## Objective
Implement symbol binding and scope resolution.

## Inputs
- `Atlas-SPEC.md`

## Deliverables
- Scope stack for blocks/functions.
- Resolve identifiers to symbols.
- Detect unknown symbols and redeclaration.
- Enforce function hoisting at top-level.
- Enforce no forward reference for variables.

## Steps
- Implement scope stack with push/pop.
- Bind identifiers to symbol entries.
- Emit diagnostics for unknown and redeclared symbols.
- Build a pre-pass for top-level function declarations.

## Exit Criteria
- Binder tests pass for scope and shadowing.
