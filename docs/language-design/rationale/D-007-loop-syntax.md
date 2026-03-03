# D-007: Loop Syntax

> **Tracking:** `atlas-track decision D-007` — source of truth
> **This file:** Extended rationale and migration guidance

**Principles:** Small Surface Area, Cohesion Over Sprawl

---

## Context

Atlas v0.2 supports three loop forms:

```atlas
// C-style for loop
for (var i = 0; i < 10; i++) {
    print(str(i));
}

// For-in loop
for item in items {
    print(item);
}

// While loop
while (condition) {
    doWork();
}
```

Additionally, `++` and `--` operators exist primarily to support C-style for loops.

### Problems

1. **Redundant:** C-style for can always be written as while + manual counter
2. **Error-prone:** Off-by-one errors in `i < 10` vs `i <= 10`
3. **Requires `++`:** The increment operators exist mainly for this syntax
4. **AI variance:** Agents pick between for-styles inconsistently

---

## Decision

**Remove C-style `for(;;)` loops.** Keep only:

```atlas
for item in collection { }   // Iteration over collections
while condition { }          // Conditional repetition
```

**Also remove `++` and `--` operators.** Use `+= 1` and `-= 1`.

---

## Rationale

### How Other Languages Handle This

| Language | For-Each | C-Style For | Infinite/Conditional |
|----------|----------|-------------|---------------------|
| **Rust** | `for x in iter` | No | `loop`, `while` |
| **Go** | `for x := range` | `for i := 0; ...` | `for { }`, `for cond` |
| **Python** | `for x in iter` | No | `while` |

Go keeps C-style for, but Go also uses it as its only loop construct (no `while` keyword).
Rust and Python removed C-style for entirely — both are successful languages.

Atlas already has `for-in` and `while`. C-style for adds nothing but surface area.

### Why Remove `++`/`--`

These operators exist in Atlas specifically for C-style for loops:

```atlas
for (var i = 0; i < 10; i++) { }
//                      ^^^ only real use case
```

Without C-style for, `++`/`--` have no compelling use case. `x += 1` is clearer and consistent with `+=`, `-=`, `*=`, `/=`.

---

## Consequences

### Code Changes Required

| File | Change |
|------|--------|
| `token.rs` | Remove `PlusPlus`, `MinusMinus` |
| `lexer/mod.rs` | Remove `++`, `--` token recognition |
| `parser/stmt.rs` | Remove `parse_for_stmt` (C-style) |
| `parser/stmt.rs` | Remove increment/decrement statement parsing |
| `ast.rs` | Remove `ForStmt` (keep `ForInStmt`), remove increment/decrement nodes |
| `compiler/stmt.rs` | Remove C-style for compilation |
| `vm/mod.rs` | No change (opcodes remain) |

### Breaking Changes

```atlas
// Before: compiles
for (var i = 0; i < 10; i++) { print(str(i)); }
x++;

// After: parse errors
// Use instead:
let mut i = 0;
while i < 10 {
    print(str(i));
    i += 1;
}
x += 1;
```

### Index-Based Iteration

For cases requiring index access:

```atlas
// Option 1: enumerate (if/when added to stdlib)
for (index, item) in enumerate(items) {
    print(str(index) + ": " + item);
}

// Option 2: manual counter
let mut i = 0;
for item in items {
    print(str(i) + ": " + item);
    i += 1;
}

// Option 3: range iteration (if/when ranges added)
for i in range(0, 10) {
    print(str(i));
}
```

---

## Examples

### Before (v0.2)
```atlas
for (var i = 0; i < len(items); i++) {
    let item = items[i];
    process(item);
}
```

### After (v0.3)
```atlas
for item in items {
    process(item);
}

// If index needed:
let mut i = 0;
while i < len(items) {
    let item = items[i];
    process(item);
    i += 1;
}
```

---

## Future Consideration

Consider adding `range()` builtin for numeric iteration:

```atlas
for i in range(0, 10) { }      // 0..9
for i in range(0, 10, 2) { }   // 0, 2, 4, 6, 8
```

This would be added as stdlib function, not new syntax.

---

## References

- PRINCIPLES.md: "Small Surface Area"
- Rust loops: https://doc.rust-lang.org/book/ch03-05-control-flow.html
- Python removed C-style for in initial design
