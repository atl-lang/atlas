# Phase 04 - Diagnostic Normalization

## Objective
Normalize diagnostics for stable golden tests.

## Inputs
- `docs/specification/diagnostic-system.md`

## Deliverables
- Normalizer that strips non-deterministic data (absolute paths, timestamps).
- JSON diagnostics used as golden source.

## Steps
- Implement normalization helpers.
- Apply normalization in test harness.

## Exit Criteria
- Golden tests pass across different machines.
