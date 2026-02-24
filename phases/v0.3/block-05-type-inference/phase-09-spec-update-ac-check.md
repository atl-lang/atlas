# Phase 09: Spec Update + Acceptance Criteria Check

**Block:** 5 (Type Inference)
**Depends on:** All previous phases complete

## Spec updates

### `docs/specification/syntax.md`

Update the Functions section:
- Remove "Return type must be explicit" from the Rules list
- Add: "Return type may be omitted — it is inferred from the function body"
- Add examples of omitted return type

Update the Type Inference section (new section to add):

```
## Type Inference

Atlas uses local type inference. Full Hindley-Milner is not in scope.

### What is inferred
- Local variable types: `let x = 42` → x: number
- Function return types: `fn f(x: number) { return x * 2; }` → return type number
- Generic type arguments at call sites: `identity(42)` → T=number

### What requires explicit annotation
- Function parameter types (always required)
- Generic type parameters that do not appear in parameter position
- When inference is ambiguous or the inferred type is incorrect

### Inference failure
When inference cannot determine a type, AT3050 or AT3051 is emitted with a suggestion
to add an explicit annotation.
```

## Acceptance Criteria check (from V03_PLAN.md)

- [ ] **Local `let` bindings do not require type annotation**
  Evidence: Phase 04 tests pass, typechecker infers from initializer
- [ ] **Return type can be omitted for single-expression functions**
  Evidence: Phase 03 wires infer_return_type, tests pass
- [ ] **Generic type arguments can be omitted when inferable**
  Evidence: Phase 05 implements call-site inference, tests pass
- [ ] **Inference errors report clearly what was inferred vs. what was needed**
  Evidence: Phase 06 AT3050/AT3051/AT3052 with "inferred from" context
- [ ] **Both engines handle inferred types identically**
  Evidence: Phase 07 parity test suite, 20+ tests, zero divergence

## Final checks

```bash
cargo nextest run --workspace        # 0 new failures
cargo clippy --workspace -- -D warnings  # 0 warnings
cargo fmt --all --check              # clean
```

## STATUS.md update

- `Last Updated` → today
- `Last Completed: Block 5 Phase 09`
- `Next:` Block 6 (Error Handling) or Block 7 (JIT)
- Block 5 row: `✅ Complete (date)`
- Block 5 completion metrics table

## Post-Block-5 architectural discussion

Before scaffolding Block 6, surface:

**Topic: Implicit returns (Block 9 scope overlap)**
Block 9 "Quick Wins" includes implicit last-expression return (like Rust). Block 5 adds return
type inference. These interact: `fn f(x: number) { x * 2 }` (no `return`) would need BOTH
implicit return (Block 9) AND return type inference (Block 5) to work. Decide at Block 5
completion: does implicit return belong in Block 5 (since inference is in flight) or stay
in Block 9 as planned?
