# Phase 03: Parser — Arrow Syntax

**Block:** 4 (Closures + Anonymous Functions)
**Depends on:** Phase 02 complete

## Current State (verified 2026-02-23)

`TokenKind::FatArrow` (`=>`) is only consumed inside `parse_match_expr()` — specifically after each pattern in a match arm (`parser/expr.rs:613`). It never appears in `parse_prefix()` or `parse_infix()`. No disambiguation needed.

`TokenKind::LeftParen` in `parse_prefix()` dispatches to `parse_group()`, which handles `(expr)` grouping and tuple-like forms.

## Goal

Parse `(x) => x + 1` and `(x, y) => x + y` as `Expr::AnonFn` (same node as Phase 02). The arrow form is pure sugar — it produces an identical AST node with `body = Expr` (not `Expr::Block`).

## Disambiguation

`(expr)` = grouped expression — when `=>` does NOT follow the closing `)`
`(params) => expr` = arrow fn — when `=>` DOES follow the closing `)`

The parser handles this with a 1-token lookahead after `parse_group()`:

In `parse_prefix()`, change `LeftParen` handling:

```rust
TokenKind::LeftParen => {
    // Attempt arrow fn: peek ahead after the group
    if let Some(arrow_fn) = self.try_parse_arrow_fn()? {
        Ok(arrow_fn)
    } else {
        self.parse_group()
    }
}
```

`try_parse_arrow_fn()` — speculative parse:
1. Save position
2. Try to parse `(ident, ident, ...)` — identifiers only, no type annotations
3. If `)` followed by `=>` → commit, return `Expr::AnonFn` with untyped params
4. Otherwise → restore position, return `None` (fall through to `parse_group`)

**Untyped params in arrow form**: params without type annotations get `type_ref = None`. The typechecker infers types from context (Phase 04).

## Typed arrow form

`(x: number) => x + 1` — also valid. Parser must handle both:
- `(ident) =>` — untyped
- `(ident: type) =>` — typed

After consuming `(`, peek: if next is `ident` followed by `:` or `,` or `)` followed by `=>` → arrow fn. Otherwise → group.

## Examples

```atlas
let double = (x) => x * 2;
let add = (x, y) => x + y;
let greet = (name: string) => "hello";
```

All desugar to `Expr::AnonFn { params, return_type: None, body: Box::new(expr), span }`.

## Acceptance Criteria

- [ ] `(x) => x + 1` parses to `Expr::AnonFn`
- [ ] `(x, y) => x + y` parses to `Expr::AnonFn`
- [ ] `(x: number) => x * 2` parses to `Expr::AnonFn` with typed param
- [ ] `(x + 1)` still parses as grouped expression (no regression)
- [ ] Match arm `Ok(x) => ...` still parses correctly (no regression)
- [ ] `cargo test` passes
