# Phase 02: Inference Module Audit

**Block:** 5 (Type Inference)
**Depends on:** Phase 01 complete

## Goal

`crates/atlas-runtime/src/typechecker/inference.rs` was written ahead of time (1,058 lines,
Block 4 era). Verify it is correct against current AST, Type, and Expr variants before wiring.

## What to verify

1. **`infer_return_type(body: &Block) -> InferredReturn`** (line 415)
   - Walks block statements looking for `Stmt::Return` and tail expressions
   - Returns `InferredReturn { ty: Type, confidence: Confidence }`
   - Verify it handles all `Stmt` variants present in the current AST

2. **`infer_expr_type(expr: &Expr) -> Type`** (line 513)
   - Maps literal expressions to types without running a full typecheck
   - Verify `Expr::AnonFn` (added Block 4) is handled (or gracefully falls through to Unknown)

3. **`BidirectionalChecker`** (line 68)
   - Used for generic call-site inference in Phase 05
   - Verify `CheckingMode`, `BidirectionalResult` match current types

4. **`InferenceHeuristics`** (line 286)
   - `infer_literal_type`, `prefer_primitive`, `infer_union_from_branches`
   - Verify Type variants referenced still exist

## Fix pattern

Any reference to a removed/renamed Type variant or Expr variant: update to current name.
Do not redesign — fix to match current API, not the other way around.

## Acceptance Criteria

- [ ] `inference.rs` compiles without warnings when referenced from typechecker/mod.rs
- [ ] `infer_return_type` handles all current `Stmt` variants
- [ ] `infer_expr_type` handles `Expr::AnonFn` (returns function type or Unknown)
- [ ] No logic changes — this phase is audit + fix only
