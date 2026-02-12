# Phase 13 - Debugger Hooks

## Objective
Add hooks to support future debugging tools without changing semantics.

## Inputs
- `docs/runtime.md`

## Deliverables
- Instruction-level hook callbacks (enabled via feature flag).
- No impact on performance when disabled.

## Steps
- Define hook interface.
- Ensure hooks compile out when disabled.

## Exit Criteria
- Hook-enabled builds pass existing VM tests.
