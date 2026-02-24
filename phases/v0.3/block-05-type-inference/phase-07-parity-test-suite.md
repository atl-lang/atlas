# Phase 07: Parity Test Suite

**Block:** 5 (Type Inference)
**Depends on:** Phases 03–05 complete

## Goal

Comprehensive parity test suite. Minimum 20 new parity tests in `tests/typesystem/inference.rs`.

## Test categories

### Return type inference (6 tests)
- `fn f(x: number) { return x * 2; }` → infers number, executes correctly
- `fn f() { return "hello"; }` → infers string
- `fn f() { }` → infers void (no return)
- `fn f(x: bool) { if (x) { return 1; } return 0; }` → infers number from both branches
- Omitted return type on recursive function → infers correctly
- Omitted return type + explicit param types → parity between engines

### Local variable inference (6 tests)
- `let x = 42; x + 1` → number inference + arithmetic
- `let s = "hello"; len(s)` → string inference + stdlib
- `let arr = [1, 2, 3]; arr[0]` → array inference + index
- `let b = true; !b` → bool inference
- `var x = 10; x = 20; x` → var with inferred type reassignment
- Chained inference: `let x = 1; let y = x + 1; y`

### Generic call-site inference (4 tests)
- `identity(42)` without type annotation → 42
- `identity("hello")` → "hello"
- `first([10, 20, 30])` → 10
- Generic with multiple type params, all inferrable

### Edge cases (4 tests)
- Infer through function composition: `let f = (x) => x + 1; f(5)` → 6
- HOF with inferred return: `map([1,2,3], fn(x: number) { return x * 2; })[0]` → 2
- Nested inferred functions
- Inferred return + ownership annotation: `fn f(own x: number) { return x; }` → infers number

## Acceptance Criteria

- [ ] Minimum 20 new tests in `tests/typesystem/inference.rs`
- [ ] All use `assert_parity_*` — both engines verified
- [ ] Zero failures
- [ ] `cargo clippy -- -D warnings` clean
