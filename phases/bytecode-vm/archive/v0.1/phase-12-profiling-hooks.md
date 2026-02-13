# Phase 12 - VM Profiling Hooks

## Objective
Add optional profiling hooks for future performance work.

## Inputs
- `docs/RUNTIME.md`

## Deliverables
- Hooks for instruction counting.
- Feature flag to enable/disable profiling.

## Steps
- Add counters and hook callbacks.
- Gate behind feature flag.

## Exit Criteria
- Profiling can be toggled without affecting semantics.
