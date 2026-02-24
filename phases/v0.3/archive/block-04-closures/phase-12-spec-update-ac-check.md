# Phase 12: Spec Update + Acceptance Criteria Check

**Block:** 4 (Closures + Anonymous Functions)
**Depends on:** All previous phases complete

## Goal

Update the language specification and verify all Block 4 acceptance criteria are met against the actual codebase.

## Spec updates

### `docs/specification/syntax.md`

Add closure/anonymous function syntax section:

```
Anonymous Functions
-------------------

fn expression:
    fn ( param-list ) -> return-type block-expr
    fn ( param-list ) block-expr           -- return type inferred

Arrow expression:
    ( param-list ) => expression

param-list:
    (empty)
    param
    param , param-list

param:
    [ownership] identifier : type
    identifier                          -- type inferred (arrow form only)

ownership:
    own | borrow | shared

Function type:
    ( type-list ) -> type

Capture semantics:
    Copy types: captured by value at closure creation time
    non-Copy types: moved into closure at creation time (caller loses ownership)
    borrow parameters: cannot be captured into closures
```

### `docs/specification/memory-model.md`

Add section on closure capture semantics under ownership model — cross-reference Block 2 ownership annotations.

## Acceptance Criteria Check (from V03_PLAN.md)

Verify each AC against the codebase:

- [ ] **Anonymous functions parse, compile, and execute**
  - Evidence: phases 02–06 complete, tests pass
- [ ] **Arrow functions parse as sugar for anonymous functions**
  - Evidence: phase 03 complete, same AST node produced
- [ ] **Closures capture `Copy` values by copy**
  - Evidence: phase 04 typechecker + phase 07 parity tests
- [ ] **Closures capture non-`Copy` values by move (caller loses ownership)**
  - Evidence: phase 04 + phase 10 ownership integration tests
- [ ] **Both engines handle closures identically**
  - Evidence: phase 07 + phase 11 parity suite — all `assert_parity_*` pass
- [ ] **Existing closure tests continue to pass**
  - Evidence: all 49 original tests in `closures.rs` still pass

## Final checks

```bash
cargo test --workspace          # 0 failures required
cargo clippy -- -D warnings     # 0 warnings required
cargo fmt --all --check         # clean required
```

Record test count delta in STATUS.md.

## STATUS.md update

Update:
- `Last Updated`
- `Last Completed: Block 4 Phase 12`
- `Next: Scaffold Block 5 (Type Inference)`
- Block 4 row: `✅ Complete (date)`
- Block 4 completion metrics table

## Commit

Single commit on `block/closures` branch. PR opens after this phase — block is complete.

## Post-Block-4 Architectural Discussion (REQUIRED before scaffolding Block 5)

Before scaffolding Block 5, surface the following to the user. This is a **conversation, not a unilateral decision** — wait for input.

**Topic: Leading indicators beyond coverage and benchmarks**

Current monitoring is lagging (coverage after merge, benchmarks after merge). Two gaps identified:

1. **Test-to-production-code ratio per crate** — we track total test count but not ratio. A crate could balloon in production code while tests stay flat. Should we track lines-of-test / lines-of-production per crate in STATUS.md at block completion?

2. **Zero-allocation paths in the interpreter hot path** — the interpreter's eval loop should never allocate on the happy path. This isn't tested or tracked anywhere. Options: a `dhat` heap profiler run in CI, a custom allocator in tests that panics on unexpected allocation, or document it as a manual audit per block.

Raise these in the block completion summary and get the user's direction before proceeding.
