# Phase 02: Error Type Compatibility Validation

**Block:** 6 (Error Handling)
**Depends on:** Phase 01 complete

## Current State (verified 2026-03-02)

- `check_try()` extracts `(ok_type, err_type)` from `Result<T, E>` but does NOT validate that `err_type` matches the enclosing function's return type error parameter.
- This means `fn foo() -> Result<number, number> { let x: Result<number, string> = ...; x?; }` would silently allow mismatched error types.

## What Gets Done

1. **Typechecker** (`typechecker/expr.rs` `check_try`): After extracting `err_type`, look up the enclosing function's return type. If it's `Result<_, E>`, verify `err_type` is compatible with `E`. Emit AT3028 if not.
2. **Tests**: Type error tests for mismatched error types.

## Acceptance Criteria

- [ ] `?` on `Result<T, string>` in function returning `Result<U, number>` emits AT3028
- [ ] `?` on `Result<T, E>` in function returning `Result<U, E>` (same E) passes
- [ ] All existing tests pass (0 regressions)
