# D-010: Type::Unknown Semantics

> **Tracking:** `atlas-track decision D-010` — source of truth
> **This file:** Extended rationale and migration guidance

**Principles:** Explicit Over Implicit, No Ambiguous Syntax

---

## Context

Atlas v0.2 uses `Type::Unknown` as a permissive wildcard:

```rust
// In typechecker/expr.rs
Expr::ObjectLiteral(_) => Type::Unknown

// In types.rs is_assignable_to
(Type::Unknown, _) => true  // Unknown satisfies ANY type
```

### Problems

1. **Type safety bypassed:** Unknown flows into concrete types silently
2. **Errors hidden:** Type mismatches become runtime errors, not compile errors
3. **AI learns bad patterns:** Generated code appears to typecheck but fails at runtime
4. **Contradicts PRD:** "No implicit any, no implicit nullable"

### Example of Current Broken Behavior

```atlas
let obj = { x: 1 };           // Typechecks to Unknown
let n: number = obj;          // Allowed! (Unknown fits number)
print(n + 1);                 // Runtime error: can't add object to number
```

---

## Decision

**`Type::Unknown` indicates an error state, not a wildcard.**

1. Flowing `Unknown` into a concrete type position = compile error
2. Operations on `Unknown` values = compile error
3. `Unknown` only appears during error recovery, never in valid programs

---

## Rationale

### What Unknown Should Mean

| Interpretation | Behavior | Assessment |
|----------------|----------|------------|
| "Any type" (TypeScript) | Bypasses all checks | Defeats type system |
| "Infer later" | Placeholder during inference | Valid internal use |
| **"Error occurred"** | Signals failed type resolution | Correct for Atlas |

`Unknown` should behave like Rust's `!` (never) or a poison value:
- It propagates through expressions
- It does NOT satisfy concrete type requirements
- It indicates the typechecker already reported an error

### How TypeScript Handles This

TypeScript has both `any` and `unknown`:
- `any`: bypasses all checks (the bad one)
- `unknown`: requires explicit type assertion/narrowing (the safe one)

Atlas's `Unknown` was behaving like TypeScript's `any`.
It should behave more like TypeScript's `unknown` — requiring explicit handling.

---

## Consequences

### Typechecker Changes

```rust
// Before (types.rs)
pub fn is_assignable_to(&self, target: &Type) -> bool {
    match (self, target) {
        (Type::Unknown, _) => true,  // WRONG
        // ...
    }
}

// After
pub fn is_assignable_to(&self, target: &Type) -> bool {
    match (self, target) {
        // Unknown does NOT satisfy concrete types
        (Type::Unknown, Type::Unknown) => true,  // Unknown = Unknown is fine
        (Type::Unknown, _) => false,             // Unknown != concrete
        (_, Type::Unknown) => false,             // concrete != Unknown
        // ...
    }
}
```

### Object Literals Get Concrete Types

```rust
// Before (typechecker/expr.rs)
Expr::ObjectLiteral(_) => Type::Unknown

// After
Expr::ObjectLiteral(obj) => {
    // Build structural type from entries
    let fields = obj.entries.iter().map(|e| {
        (e.key.name.clone(), self.check_expr(&e.value))
    }).collect();
    Type::Structural { fields }
}
```

### Error Messages

When `Unknown` is encountered in a concrete position:

```
Error AT2050: Cannot use value of unknown type
  --> file.atl:5:10
   |
 5 | let n: number = obj;
   |                 ^^^ type could not be determined
   |
   = help: Add explicit type annotation or check for earlier errors
```

### What Still Returns Unknown

`Unknown` is valid ONLY for:
1. **Error recovery:** After a parse/type error, to continue checking
2. **Unresolved references:** Undefined variable (error already reported)
3. **Failed inference:** When inference cannot determine type (error reported)

In all cases, an error diagnostic is ALSO emitted. `Unknown` is never silent.

---

## Examples

### Before (v0.2) — Broken
```atlas
let obj = { x: 1 };
let n: number = obj;     // Silently allowed
let result = n + 1;      // Runtime crash
```

### After (v0.3) — Correct
```atlas
let obj = record { x: 1 };
// Type: { x: number }

let n: number = obj;
// Error AT2001: Type mismatch
//   expected: number
//   found: { x: number }

let result = obj.x + 1;  // Correct: access field, then add
```

---

## Migration

Most code won't be affected because:
1. Well-typed code doesn't produce `Unknown`
2. Code that relied on `Unknown` bypassing checks was already buggy

Code that breaks was hiding type errors. Those errors are now surfaced.

---

## References

- PRINCIPLES.md: "Explicit Over Implicit"
- PRD: "No implicit any, no implicit nullable"
- TypeScript `unknown` vs `any`: https://www.typescriptlang.org/docs/handbook/2/functions.html#unknown
