# Phase 01 - Lexer

## Objective
Implement tokenization with spans and diagnostics.

## Inputs
- `Atlas-SPEC.md`
- `docs/diagnostics.md`

## Deliverables
- Token types for all syntax in `Atlas-SPEC.md`.
- Lexer produces `Token { kind, lexeme, span }`.
- Comment and whitespace handling.
- Keywords are not allowed as identifiers.
- Reserved keywords include `import` even though modules are not in v0.1.

## Steps
- Define token enum and span type.
- Implement scanning loop with lookahead.
- Emit diagnostics on invalid tokens.
- Enforce reserved keywords.

## Exit Criteria
- Lexer golden tests pass.
- Lexer errors include file/line/column and length.
