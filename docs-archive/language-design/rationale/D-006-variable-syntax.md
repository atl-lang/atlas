# D-006: Variable Declaration Syntax

> **Tracking:** `atlas-track decision D-006` — source of truth
> **This file:** Extended rationale and migration guidance

**Principles:** Cohesion Over Sprawl, Small Surface Area

---

## Context

Atlas v0.2 supports three forms of variable declaration:

```atlas
let x = 5;           // Immutable
let mut y = 5;       // Mutable (Rust-style, recommended)
var z = 5;           // Mutable (deprecated, but fully functional)
```

The `var` keyword was included for familiarity with JavaScript/Go developers.
It was marked deprecated but never removed.

### Problems

1. **Two paths to mutability:** `var` and `let mut` do identical things
2. **AI confusion:** Agents generate both styles randomly
3. **"Deprecated" is meaningless:** Code using `var` compiles without error
4. **Contradicts PRD:** "Cohesion over sprawl" violated

---

## Decision

**Remove `var` entirely.** Not deprecate — remove from lexer, parser, and grammar.

Atlas variable syntax becomes:

```atlas
let x = 5;        // Immutable binding
let mut y = 5;    // Mutable binding
```

---

## Rationale

### Why Rust's Model

| Approach | Example | Assessment |
|----------|---------|------------|
| Go | `var x = 5` / `x := 5` | Two syntaxes (violates cohesion) |
| JavaScript | `let`/`const`/`var` | Three syntaxes (worse) |
| Python | `x = 5` | Implicit mutability (violates explicit) |
| **Rust** | `let`/`let mut` | Single keyword, explicit modifier |

Rust's approach:
- One keyword (`let`)
- Explicit mutability (`mut` modifier)
- Immutable by default (safe)
- Already what Atlas recommends

### Why Remove, Not Deprecate

From ANTI-PATTERNS.md:
> "Deprecated But Supported" doubles syntax surface area.
> AI agents see both patterns, generate both randomly.

Deprecation warnings don't prevent generation. Removal does.

---

## Consequences

### Code Changes Required

| File | Change |
|------|--------|
| `token.rs` | Remove `TokenKind::Var` |
| `lexer/mod.rs` | Remove `var` from keyword map |
| `parser/stmt.rs` | Remove `Var` from `parse_var_decl` match |
| `ast.rs` | Remove `uses_deprecated_var` field from `VarDecl` |

### Breaking Changes

All existing code using `var` will fail to parse.

```atlas
// Before: compiles with warning
var x = 5;

// After: parse error
// Error AT1001: Unknown keyword 'var'. Did you mean 'let mut'?
```

### Migration

Mechanical replacement: `var x` → `let mut x`

Migration script in `/docs/language-design/migration/from-var-to-let-mut.md`

---

## Examples

### Before (v0.2)
```atlas
var count = 0;
for item in items {
    var temp = process(item);
    count = count + temp;
}
```

### After (v0.3)
```atlas
let mut count = 0;
for item in items {
    let temp = process(item);  // temp doesn't need mut
    count = count + temp;
}
```

---

## References

- PRINCIPLES.md: "Cohesion Over Sprawl"
- ANTI-PATTERNS.md: "Deprecated But Supported"
- Rust Reference: https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html
