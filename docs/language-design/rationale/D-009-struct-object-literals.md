# D-009: Struct and Object Literal Syntax

> **Tracking:** `atlas-track decision D-009` — source of truth
> **This file:** Extended rationale and migration guidance

**Principles:** No Ambiguous Syntax, Explicit Over Implicit

---

## Context

Atlas v0.2 overloads brace syntax `{}` for multiple constructs:

```atlas
// Block expression
let x = { let a = 5; a + 1 };

// Object literal (HashMap at runtime)
let obj = { name: "Alice", age: 30 };

// Structural type (in type position)
fn process(data: { name: string, age: number }) { }

// Struct expression (if identifier starts with uppercase)
let person = Person { name: "Alice", age: 30 };
```

### Problems

1. **Parser backtracking:** Must try object literal, then fall back to block
2. **Empty braces ambiguous:** `{}` — empty object or empty block?
3. **Uppercase convention:** Struct vs object determined by identifier casing
4. **Type confusion:** Object literals typecheck to `Unknown`

---

## Decision

**Disambiguate by requiring type prefix for struct/record literals:**

```atlas
// Block expression (no prefix)
let x = { let a = 5; a + 1 };
let empty_block = { };  // Returns null

// Struct literal (type prefix required)
let person = Person { name: "Alice", age: 30 };

// Anonymous record literal (use `record` keyword)
let obj = record { name: "Alice", age: 30 };
```

---

## Rationale

### How Rust and Go Handle This

**Rust:**
```rust
let block = { let x = 5; x + 1 };           // Block
let person = Person { name: "Alice" };       // Struct (type prefix)
```

**Go:**
```go
person := Person{Name: "Alice", Age: 30}     // Struct (type prefix)
// No block expressions in Go
```

Both languages require the type name before `{` for struct literals.
This completely disambiguates from blocks.

### Why `record` Keyword for Anonymous Records

Atlas needs anonymous record literals (like TypeScript object literals).
Options considered:

| Option | Syntax | Assessment |
|--------|--------|------------|
| Bare braces | `{ name: "x" }` | Ambiguous with blocks |
| Hash braces | `#{ name: "x" }` | Unusual, not from any major language |
| `object` keyword | `object { name: "x" }` | Conflicts with OOP terminology |
| **`record` keyword** | `record { name: "x" }` | Clear, used in C#/F# for similar |

`record` is explicit, unambiguous, and familiar from functional languages.

### Parser Simplification

Current parser in `try_parse_object_literal`:
```rust
// Save position, try object literal, backtrack if fails
let saved_pos = self.current;
// ... try parse ...
if failed { self.current = saved_pos; }  // Backtrack
```

After this change:
```rust
// No backtracking needed
TokenKind::LeftBrace => parse_block(),
TokenKind::Identifier if is_type_name() => parse_struct_literal(),
TokenKind::Record => parse_record_literal(),
```

---

## Consequences

### New Keyword

Add `record` to lexer keywords.

### Code Changes Required

| File | Change |
|------|--------|
| `token.rs` | Add `TokenKind::Record` |
| `lexer/mod.rs` | Add `record` to keyword map |
| `parser/expr.rs` | Remove `try_parse_object_literal` backtracking |
| `parser/expr.rs` | Add `parse_record_literal` for `record { }` |
| `parser/expr.rs` | Struct literals require type prefix |
| `ast.rs` | Distinguish `RecordLiteral` from `StructExpr` |
| `typechecker/expr.rs` | Record literals get structural type, not Unknown |

### Breaking Changes

```atlas
// Before: compiles (object literal)
let obj = { name: "Alice", age: 30 };

// After: parse error (looks like block with bad syntax)
// Use instead:
let obj = record { name: "Alice", age: 30 };

// Or define a struct:
// (struct declaration syntax TBD)
let person = Person { name: "Alice", age: 30 };
```

### Empty Constructs

```atlas
{ }                    // Empty block, returns null
record { }             // Empty record (allowed? TBD)
Person { }             // Struct with all default fields (if supported)
```

---

## Examples

### Before (v0.2)
```atlas
let config = { host: "localhost", port: 8080 };
let result = { let x = compute(); x * 2 };
```

### After (v0.3)
```atlas
let config = record { host: "localhost", port: 8080 };
let result = { let x = compute(); x * 2 };

// With defined struct
let config = Config { host: "localhost", port: 8080 };
```

---

## Type System Integration

Record literals now have concrete structural types:

```atlas
let obj = record { name: "Alice", age: 30 };
// Type: { name: string, age: number }

// Type annotation works
let obj: { name: string, age: number } = record { name: "Alice", age: 30 };

// Type mismatch caught at compile time
let obj: { name: string } = record { name: "Alice", age: 30 };
// Error: Record has extra field 'age'
```

---

## References

- PRINCIPLES.md: "No Ambiguous Syntax"
- Rust struct syntax: https://doc.rust-lang.org/book/ch05-01-defining-structs.html
- Go struct literals: https://go.dev/tour/moretypes/5
