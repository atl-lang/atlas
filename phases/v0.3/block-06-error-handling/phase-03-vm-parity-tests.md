# Phase 03: VM Parity Tests for `?` Operator

**Block:** 6 (Error Handling)
**Depends on:** Phase 01 complete

## Current State (verified 2026-03-02)

- Interpreter `?` tests exist in `tests/stdlib/vm_stdlib/types_option_result.rs` (lines 525-669) BUT use `assert_eval_*` from `common/mod.rs` which runs interpreter only (`Atlas::new().eval()`).
- VM has all required opcodes: `IsResultOk`, `IsResultErr`, `ExtractResultValue`, `IsOptionSome`, `IsOptionNone`, `ExtractOptionValue`.
- `compile_try()` emits correct bytecode sequence with jump patching.
- No tests verify `?` through the VM pipeline (Compiler → Bytecode → VM).

## What Gets Done

1. **Add VM-specific `?` tests** to `tests/stdlib/vm_stdlib/types_option_result.rs` using the VM execution helper pattern (Lexer → Parser → Binder → TypeChecker → Compiler → VM).
2. Mirror all existing interpreter `?` tests through VM pipeline.
3. Add Option `?` VM tests (after Phase 01).

## Acceptance Criteria

- [ ] ≥10 VM parity tests for Result `?` operator
- [ ] ≥5 VM parity tests for Option `?` operator (post Phase 01)
- [ ] VM and interpreter produce identical output for all `?` test cases
- [ ] All existing tests pass (0 regressions)
