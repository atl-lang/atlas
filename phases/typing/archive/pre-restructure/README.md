# Archived Test-Only Phases

**Archived on:** 2026-02-12

## Why These Were Archived

These phases were separated test-only phases that caused agents to accumulate tests without building implementation. They have been merged into their corresponding implementation phases.

## What Happened to Each Phase

| Old Phase | Status | Merged Into |
|-----------|--------|-------------|
| phase-05-type-rules-tests.md | Completed | phase-02-typechecker.md |
| phase-06-scope-shadowing-tests.md | Completed | phase-03-scopes-shadowing.md |
| phase-15-warning-tests.md | Archived | phase-06-warnings.md |
| phase-16-top-level-order-tests.md | Archived | phase-03-scopes-shadowing.md |
| phase-17-operator-rule-tests.md | Archived | phase-02-typechecker.md |
| phase-18-string-semantics-tests.md | Archived | phase-08-semantic-edge-cases.md |
| phase-19-related-span-coverage.md | Archived | phase-07-diagnostics.md |
| phase-20-diagnostic-normalization-tests.md | Archived | phase-07-diagnostics.md |
| phase-21-numeric-edge-tests.md | Archived | phase-08-semantic-edge-cases.md |
| phase-22-diagnostic-ordering-tests.md | Archived | phase-09-typecheck-stability.md |

## New Structure

The typing section now has 9 focused phases instead of 22:

1. phase-01-binder.md
2. phase-02-typechecker.md
3. phase-03-scopes-shadowing.md (renamed from phase-06)
4. phase-04-nullability.md (renamed from phase-07)
5. phase-05-function-returns.md (merged from phase-10 + phase-12)
6. phase-06-warnings.md (merged from phase-14 + phase-15)
7. phase-07-diagnostics.md (merged from phase-13 + phase-19 + phase-20)
8. phase-08-semantic-edge-cases.md (merged from phase-18 + phase-21)
9. phase-09-typecheck-stability.md (merged from phase-11 + phase-22)

Each phase now includes both implementation AND comprehensive tests.

## See Also

- `PHASE_RESTRUCTURE_PLAN.md` - Full restructure rationale and plan
- `BUILD-ORDER.md` - Updated canonical build order
- `STATUS.md` - Updated progress tracker
