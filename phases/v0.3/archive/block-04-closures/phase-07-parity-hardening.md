# Phase 07: Parity Hardening — Close var-Mutation Divergence

**Block:** 4 (Closures + Anonymous Functions)
**Depends on:** Phase 05 + Phase 06 complete

## Current State (verified 2026-02-23)

`closures.rs` header documents: "For `var` mutations after closure creation, behavior may diverge."
All 49 existing closure tests pass. The divergence is theoretical for named closures — the VM snapshots upvalues at definition time, the interpreter reads live scope at call time.

With anonymous functions introduced, this divergence becomes a concrete test surface.

## Goal

Define the canonical behavior for `var` mutation post-capture, make both engines agree, and lock it with tests.

## The Decision (make it explicit in this phase)

**Canonical rule:** A closure captures a `var` **by value at creation time**. Mutations to the outer `var` after closure creation are NOT visible inside the closure.

Rationale: This matches the VM's existing behavior (snapshot semantics), is consistent with value semantics established in Block 1, and is predictable. The interpreter must be brought into alignment.

## Implementation

### Interpreter alignment

Currently the interpreter uses live scope — outer `var` mutations ARE visible. To align:

When evaluating `Expr::AnonFn`, capture the current values of all referenced outer variables into the `upvalues` vec (instead of leaving it empty as in Phase 06).

At call time, use the upvalue snapshot instead of walking the live scope for those variables.

Variables NOT referenced by the closure body: still use live scope (no change needed).

The mechanism: during Phase 06's `Expr::AnonFn` eval, walk the body AST to find free variable references, look them up in the current scope, and snapshot their values.

### Verification

Write test cases that previously showed divergence:

```atlas
var x = 10;
let f = fn() -> number { x; };
x = 99;
f();  // must return 10 in BOTH engines (snapshot at creation)
```

```atlas
var counter = 0;
let inc = fn() -> number { counter = counter + 1; counter; };
// counter mutation inside closure: this is SetUpvalue in VM
// interpreter must agree
```

## Acceptance Criteria

- [ ] Outer `var` mutation after closure creation is NOT visible inside closure (both engines)
- [ ] Mutation of a `var` INSIDE a closure is reflected correctly (both engines)
- [ ] All 49 existing closure tests still pass
- [ ] Minimum 5 new parity tests covering the var-mutation cases
- [ ] Zero divergence — `assert_parity_*` passes for all new tests
