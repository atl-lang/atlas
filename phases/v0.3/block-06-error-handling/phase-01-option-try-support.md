# Phase 01: Option `?` Support

**Block:** 6 (Error Handling)
**Depends on:** Block 3 complete

## Current State (verified 2026-03-02)

- `?` operator works for `Result<T, E>` in interpreter, compiler, and typechecker
- `TryExpr` AST node exists (`ast.rs:454`)
- Typechecker `check_try()` only matches `Result` — rejects Option
- Interpreter `eval_try()` only handles `Value::Result` — no Option path
- Compiler `compile_try()` only emits `IsResultOk` — no Option variant
- VM has `IsOptionSome`, `IsOptionNone`, `ExtractOptionValue` opcodes (0x90-0x94)

## What Gets Done

1. **Typechecker** (`typechecker/expr.rs` `check_try`): Accept `Option<T>` in addition to `Result<T,E>`. Function must return `Option<T>` when `?` is used on Option.
2. **Interpreter** (`interpreter/expr.rs` `eval_try`): Add `Value::Option` handling — unwrap `Some`, propagate `None` via `ControlFlow::Return(Value::Option(None))`.
3. **Compiler** (`compiler/expr.rs` `compile_try`): Add Option branch — emit `IsOptionSome` + `JumpIfFalse` + `ExtractOptionValue` + `Return`.
4. **Tests**: Option `?` tests for interpreter (in `tests/stdlib/types.rs` or appropriate file).

## Acceptance Criteria

- [ ] `let x = some_option()?;` unwraps Some value in interpreter
- [ ] `let x = none_option()?;` propagates None in interpreter
- [ ] Typechecker rejects `?` on non-Result/non-Option types (AT3027)
- [ ] Typechecker rejects `?` on Option in Result-returning function and vice versa
- [ ] Compiler emits correct Option opcodes
- [ ] All existing tests pass (0 regressions)
