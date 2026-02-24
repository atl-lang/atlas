# Phase 10: Ownership Integration

**Block:** 4 (Closures + Anonymous Functions)
**Depends on:** Phase 05 + Phase 06 complete

## Current State (verified 2026-02-23)

Block 2 ownership annotations (`own`, `borrow`, `shared`) work on named function parameters. `Param` at `ast.rs:187` has `ownership: Option<OwnershipAnnotation>` — already present. The typechecker checks ownership at call sites for named functions.

`Expr::AnonFn` uses `Vec<Param>` — the same struct — so ownership annotations are structurally supported from Phase 01. This phase verifies and tests that they work end-to-end.

## Goal

Verify `own`, `borrow`, `shared` annotations on anonymous function parameters are:
1. Parsed correctly (should already work via Phase 02 reusing `parse_param()`)
2. Typechecked at call sites (anonymous fn called with wrong ownership → diagnostic)
3. Handled identically in both engines

## Implementation

Mostly verification + test writing. If any gaps are found, fix them.

### Verify parse

```atlas
let f = fn(own x: Buffer) -> string { x.read(); };
let g = fn(borrow x: Buffer) -> number { x.len(); };
let h = fn(shared x: Buffer) -> void { spawn(fn() -> void { x.len(); }); };
```

Parse these and inspect the AST — `params[0].ownership` should be `Some(Own)`, `Some(Borrow)`, `Some(Shared)`.

### Verify typechecker

Passing `borrow x` where `own x` is required → diagnostic AT2xxx (reuse Block 2 error codes).
Check the typechecker's call-site ownership validation runs for anonymous fn calls, not just named fn calls.

### Verify capture interaction

Block 4 canonical rule: non-`Copy` values are captured by move. Combining with ownership:
- `own` param inside anon fn: caller must have owned the value → verified at call site
- `borrow` param inside anon fn: caller retains ownership → no capture (borrow cannot be captured, enforced in Phase 04)
- `shared` param: Arc-wrapped reference → can be captured into multiple closures

## Acceptance Criteria

- [ ] `fn(own x: Buffer)` parses with `ownership = Some(Own)`
- [ ] Passing `borrow` value to `own` param on anon fn produces a diagnostic
- [ ] Both engines handle owned params identically (parity test)
- [ ] Capturing `borrow` in a closure produces a diagnostic (verified from Phase 04)
- [ ] Minimum 6 new tests (2 per ownership annotation)
- [ ] `cargo test` passes
