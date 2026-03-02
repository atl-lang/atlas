# Phase 04: Stdlib Result Audit

**Block:** 6 (Error Handling)
**Depends on:** Phase 01 complete (Option ? must work)

## Current State (verified 2026-03-02)

- Many stdlib functions panic on invalid input (unwrap, expect) or return sentinel values.
- V03_PLAN AC: "At least 20 stdlib functions updated to use Result<T, E>".
- Existing Result-returning functions: `divide` (user-written), `result_map`, `result_and_then`, `result_or_else`.

## What Gets Done

1. **Audit all 25 stdlib modules** for functions that currently panic or return error strings.
2. **Convert ≥20 functions** to return `Result<T, E>` or `Option<T>` where appropriate.
3. **Priority targets:**
   - Parsing: `parseInt`, `parseFloat`, `toNumber` → `Result<number, string>`
   - Collections: `get` (array/map) → `Option<T>`
   - JSON: `jsonParse` → `Result<any, string>`
   - File I/O: functions that can fail → `Result<T, string>`
   - String: `indexOf` → `Option<number>` (return None instead of -1)
4. **Update tests** to use new return types.

## Acceptance Criteria

- [ ] ≥20 stdlib functions return Result<T, E> or Option<T> where currently panicking
- [ ] All existing tests updated for new return types (0 failures)
- [ ] Both engines produce identical output for updated functions
- [ ] Documented which functions changed in a summary comment
