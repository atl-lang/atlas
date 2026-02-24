# Phase 04: Local Variable Inference — Verify + Harden

**Block:** 5 (Type Inference)
**Depends on:** Phase 01 complete

## Current state (verified 2026-02-23)

`let x = 42` ALREADY infers `number`. The typechecker at `mod.rs:1151` already does:
```rust
} else {
    // No explicit type annotation - use inferred type
    init_type  // ← this IS type inference, already working
}
```

This phase verifies it is complete, adds missing edge cases, and writes the test suite.

## What to verify

1. **Primitive literals infer correctly**
   - `let x = 42` → `number`
   - `let s = "hi"` → `string`
   - `let b = true` → `bool`

2. **Collection literals infer correctly**
   - `let arr = [1, 2, 3]` → `number[]`
   - `let m = {}` → current behavior (may be `unknown`)

3. **Expression inference**
   - `let x = 1 + 2` → `number`
   - `let x = "a" + "b"` → `string`
   - `let f = (x) => x * 2` → `(unknown) -> unknown` (untyped arrow — already tested in Block 4)

4. **Diagnostic for wrong-type usage**
   - `let x = 42; x + "string"` → AT3001 type mismatch (inferred number, got string)

## Binder alignment

The binder sets `ty: Type::Unknown` for variables without annotations. The typechecker
updates this via `symbol_table.lookup_mut`. Verify the symbol table update reaches all
usage sites (LSP hover, completion, references).

## Acceptance Criteria

- [ ] All primitive literals infer correct types without annotation
- [ ] `let arr = [1,2,3]` infers `number[]`
- [ ] Inferred type flows into downstream type checks (wrong usage → error)
- [ ] LSP hover shows inferred type (not "unknown") for `let x = 42`
- [ ] Minimum 8 new tests across `tests/typesystem/inference.rs`
