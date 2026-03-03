# D-008: Function Syntax

> **Tracking:** `atlas-track decision D-008` — source of truth
> **This file:** Extended rationale and migration guidance

**Principles:** Cohesion Over Sprawl, Small Surface Area

---

## Context

Atlas v0.2 supports two anonymous function syntaxes:

```atlas
// fn expression (full form)
let double = fn(x: number) -> number { return x * 2; };

// Arrow expression (short form)
let double = (x) => x * 2;
```

Named functions use only `fn`:

```atlas
fn add(a: number, b: number) -> number {
    return a + b;
}
```

### Problems

1. **Two syntaxes for anonymous functions:** AI generates both inconsistently
2. **Arrow functions have implicit types:** Parameters untyped, return inferred
3. **Different body semantics:** Arrow = expression, fn = block with return
4. **Mixed in real code:** Some files use arrows, some use fn expressions

---

## Decision

**Remove arrow function syntax.** Keep only `fn` for all functions:

```atlas
// Named function
fn add(a: number, b: number) -> number {
    return a + b;
}

// Anonymous function (assigned to variable)
let double = fn(x: number) -> number { return x * 2; };

// Inline anonymous (e.g., in higher-order functions)
let result = map(items, fn(x: number) -> number { return x * 2; });
```

---

## Rationale

### Why Keep `fn`, Remove `=>`

| Syntax | Typed Params | Return Type | Body |
|--------|--------------|-------------|------|
| `fn(x: number) -> number { }` | Yes | Explicit/inferred | Block |
| `(x) => expr` | No | Implicit | Expression |

Arrow functions were added for brevity in callbacks. But they:
- Bypass Atlas's type annotation requirements
- Create a second "style" of function
- Don't match the named function syntax

### How Other Languages Handle This

| Language | Named | Anonymous |
|----------|-------|-----------|
| **Rust** | `fn name() {}` | `\|x\| expr` (closures) |
| **Go** | `func name() {}` | `func() {}` (same keyword) |
| **Python** | `def name():` | `lambda x: expr` |

Go's approach is cleanest: same `func` keyword for named and anonymous.
Atlas should do the same with `fn`.

### Verbosity Concern

Arrow functions are shorter:
```atlas
// Arrow (removed)
map(items, (x) => x * 2)

// fn expression
map(items, fn(x: number) -> number { return x * 2; })
```

This verbosity is acceptable because:
1. Types are explicit (Atlas principle)
2. Consistency > brevity
3. Return type inference reduces annotation burden

With return type inference (already implemented):
```atlas
map(items, fn(x: number) { return x * 2; })  // return type inferred
```

---

## Consequences

### Code Changes Required

| File | Change |
|------|--------|
| `token.rs` | `FatArrow` token remains (used in match arms) |
| `parser/expr.rs` | Remove `try_parse_arrow_fn` |
| `parser/expr.rs` | Remove arrow function path from expression parsing |
| `ast.rs` | Potentially simplify `AnonFn` node |

### Breaking Changes

```atlas
// Before: compiles
let f = (x) => x * 2;

// After: parse error
// Use instead:
let f = fn(x: number) -> number { return x * 2; };
```

### Higher-Order Functions

```atlas
// Before
let doubled = map([1, 2, 3], (x) => x * 2);
let evens = filter([1, 2, 3, 4], (x) => x % 2 == 0);

// After
let doubled = map([1, 2, 3], fn(x: number) -> number { return x * 2; });
let evens = filter([1, 2, 3, 4], fn(x: number) -> bool { return x % 2 == 0; });
```

---

## Examples

### Before (v0.2)
```atlas
let numbers = [1, 2, 3, 4, 5];
let doubled = map(numbers, (x) => x * 2);
let sum = reduce(numbers, (acc, x) => acc + x, 0);
```

### After (v0.3)
```atlas
let numbers = [1, 2, 3, 4, 5];
let doubled = map(numbers, fn(x: number) -> number { return x * 2; });
let sum = reduce(numbers, fn(acc: number, x: number) -> number { return acc + x; }, 0);
```

---

## Note on `=>`

The `=>` token (FatArrow) is NOT removed from the language.
It is still used in match arms:

```atlas
match value {
    Some(x) => process(x),
    None() => default(),
}
```

Only arrow FUNCTIONS are removed. Match arm syntax is unchanged.

---

## References

- PRINCIPLES.md: "Cohesion Over Sprawl"
- Go function syntax: https://go.dev/tour/moretypes/24
