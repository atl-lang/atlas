# Phase 9 - Typecheck Stability

## Objective
Stabilize typecheck dump format and guarantee diagnostic stability.

## Inputs
- `docs/specification/json-formats.md`
- `docs/specification/diagnostic-system.md`

## Deliverables

### Implementation
- `typecheck_version` field in JSON output
- Version field enforcement in typecheck dumps
- Diagnostic ordering guarantee (stable across runs)

### Tests
- Version field presence tests
- Version mismatch tests for future-proofing
- Diagnostic ordering stability tests (same input = same output order)
- Typecheck dump format stability tests

## Steps
1. Add version field to typecheck JSON output
2. Add tests asserting version value
3. Verify diagnostic ordering is stable and deterministic
4. Add typecheck dump stability tests

## Exit Criteria
- [ ] Typecheck dumps include version field
- [ ] Version tests pass
- [ ] Diagnostic ordering is guaranteed stable
- [ ] Typecheck dump format is stable and versioned
