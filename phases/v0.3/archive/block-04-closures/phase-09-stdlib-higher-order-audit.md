# Phase 09: Stdlib Higher-Order Function Audit

**Block:** 4 (Closures + Anonymous Functions)
**Depends on:** Phase 06 complete (interpreter executes anon fns)

## Current State (verified 2026-02-23)

The stdlib has higher-order functions (`map`, `filter`, `reduce`, `forEach`, `sort`, `find`, `any`, `all`, etc.) in `crates/atlas-runtime/src/stdlib/`. These currently accept `Value::Function` or `Value::Closure` and call them via the runtime's `call_value` mechanism. They were written before anonymous function expressions existed in the language.

## Goal

Verify every stdlib higher-order function works correctly when passed an anonymous function expression or arrow function. Fix any that don't. Add tests proving each one works.

## Audit checklist

For each function below, test with both `fn(...) { ... }` and `(x) => expr` forms:

| Function | Module | Test needed |
|----------|--------|-------------|
| `map` | array | `[1,2,3].map(fn(x: number) -> number { x * 2; })` |
| `filter` | array | `[1,2,3].filter((x) => x > 1)` |
| `reduce` | array | `[1,2,3].reduce(fn(acc: number, x: number) -> number { acc + x; }, 0)` |
| `forEach` | array | `[1,2,3].forEach((x) => print(x))` |
| `find` | array | `[1,2,3].find((x) => x == 2)` |
| `any` | array | `[1,2,3].any((x) => x > 2)` |
| `all` | array | `[1,2,3].all((x) => x > 0)` |
| `sort` | array | `[3,1,2].sort((a, b) => a - b)` |
| `flatMap` | array | `[1,2].flatMap((x) => [x, x*2])` |

For each: test both engines produce identical output (parity). Add to `tests/stdlib.rs` or `tests/closures.rs`.

## Known risk

Some stdlib functions may call the provided function via a Rust closure or internal mechanism that doesn't go through the standard Atlas call path. If an anon fn fails to execute in that path, trace to the `call_value` dispatch and fix it.

## Acceptance Criteria

- [ ] All 9 functions above work with anonymous fn and arrow syntax
- [ ] Both engines produce identical results for all tests (parity)
- [ ] Minimum 18 new tests (2 per function: fn form + arrow form)
- [ ] `cargo test` passes
