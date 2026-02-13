# Phase 14 - Debug Info Mapping

## Objective
Implement debug info mapping from bytecode to source spans.

## Inputs
- `docs/specification/json-formats.md`

## Deliverables
- Span table and instruction span mapping.
- Serialization of debug info in `.atb` files.

## Steps
- Add span table to bytecode container.
- Wire VM error reporting to span table.

## Exit Criteria
- VM errors show accurate source spans.
