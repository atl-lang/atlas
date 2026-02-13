# Phase 7 - Diagnostics & Related Spans

## Objective
Implement related spans, diagnostic normalization, and ordering guarantees.

## Inputs
- `Atlas-SPEC.md`
- `docs/specification/diagnostic-system.md`

## Deliverables

### Implementation
- Related span support for diagnostics
- Primary and related span rendering
- Diagnostic normalization (consistent ordering, deduplication)
- Deterministic diagnostic ordering guarantees

### Tests
- Related span coverage for type errors, binding errors, control flow errors
- Tests verifying related spans point to relevant code locations
- Diagnostic normalization tests (same error in different contexts produces same output)
- Diagnostic ordering tests (deterministic order across runs)
- Tests for multi-span diagnostics

## Steps
1. Implement related span tracking in diagnostic system
2. Add related span rendering to diagnostic output
3. Implement diagnostic normalization (sort, deduplicate)
4. Add comprehensive tests for related span accuracy
5. Verify diagnostic ordering is deterministic

## Exit Criteria
- [ ] Related spans implemented and rendering correctly
- [ ] Diagnostic normalization working (consistent output)
- [ ] All related span tests pass
- [ ] Diagnostic ordering is deterministic and tested
- [ ] Related spans present for all relevant error types
