# Phase 05: Integration Tests + Edge Cases

**Block:** 6 (Error Handling)
**Depends on:** Phases 01-04 complete

## What Gets Done

1. **Cross-feature integration tests:**
   - `?` in closures (capture + error propagation)
   - `?` in trait method implementations
   - `?` with ownership annotations (`borrow` param returning Result)
   - Nested `?` (function calling function with `?`)
   - `?` in match arm bodies
   - `?` in if-else branches
2. **Edge cases:**
   - `?` on already-unwrapped value (typechecker rejects)
   - Multiple `?` in single expression: `foo()? + bar()?`
   - `?` on function call result directly: `get_value()?`
3. **Parity tests** for all integration scenarios (both engines).

## Acceptance Criteria

- [ ] ≥15 integration tests covering cross-feature interactions
- [ ] Both engines produce identical output for all integration tests
- [ ] All Block 6 ACs from V03_PLAN.md verified:
  - `?` operator propagates Err to caller
  - `?` operator propagates None to caller
  - Type checker rejects `?` in non-Result/Option-returning functions
  - Both engines propagate errors identically
  - At least 20 stdlib functions updated to use Result<T, E>
