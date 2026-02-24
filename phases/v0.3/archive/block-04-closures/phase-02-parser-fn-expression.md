# Phase 02: Parser — fn Expression Syntax

**Block:** 4 (Closures + Anonymous Functions)
**Depends on:** Phase 01 complete

## Current State (verified 2026-02-23)

`parse_prefix()` in `parser/expr.rs:27` handles: Number, String, Bool, Null, Identifier, LeftParen, LeftBracket, Minus, Bang, Match.

`TokenKind::Fn` is NOT in `parse_prefix()`. Named functions are parsed as statements in `parser/mod.rs`, not as expressions.

`parse_type_ref()` and `parse_paren_type()` already handle `(T) -> R` type syntax — reuse this pattern for parameter parsing.

## Goal

Parse `fn(x: number, y: string) -> bool { x > 0; }` as `Expr::AnonFn`.

## Implementation

In `parser/expr.rs`, add to `parse_prefix()`:

```rust
TokenKind::Fn => self.parse_anon_fn(),
```

Add `parse_anon_fn()`:

```rust
fn parse_anon_fn(&mut self) -> Result<Expr, ()> {
    let start = self.advance().span; // consume `fn`
    self.consume(TokenKind::LeftParen, "Expected '(' after 'fn'")?;

    let mut params = Vec::new();
    if !self.check(TokenKind::RightParen) {
        loop {
            params.push(self.parse_param()?);
            if !self.match_token(TokenKind::Comma) { break; }
        }
    }
    self.consume(TokenKind::RightParen, "Expected ')' after parameters")?;

    let return_type = if self.match_token(TokenKind::Arrow) {
        Some(self.parse_type_ref()?)
    } else {
        None
    };

    let body = self.parse_block_expr()?; // returns Expr::Block
    let span = start.merge(body_span);

    Ok(Expr::AnonFn { params, return_type, body: Box::new(body), span })
}
```

Reuse `parse_param()` from the function declaration parser (already handles `own`/`borrow`/`shared` ownership annotations).

`parse_block_expr()` — parse `{ stmt* expr? }` returning `Expr::Block`. Check if this already exists; if not, extract from `parse_block()` in `mod.rs`.

## Edge Cases

- `fn() -> void { }` — no params, explicit void return
- `fn(x: number) { x; }` — no return type annotation (return_type = None → inferred)
- `fn(own x: Buffer) -> string { ... }` — ownership annotation on param

## Acceptance Criteria

- [ ] `fn(x: number) -> number { x + 1; }` parses to `Expr::AnonFn`
- [ ] Params with ownership annotations parse correctly
- [ ] Missing return type parses as `None`
- [ ] Parse errors produce clear diagnostics (not panics)
- [ ] `cargo test` passes
