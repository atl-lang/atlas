# Phase 15 - Debug Info Defaults

## Objective
Make debug info emission the default in v0.1.

## Inputs
- `docs/debug-info.md`

## Deliverables
- Bytecode serializer emits debug info by default.
- CLI flag to strip debug info (optional for future).

## Steps
- Wire debug info into `.atb` serializer.
- Add tests for debug info presence.

## Exit Criteria
- Debug info is present in all `.atb` files by default.
