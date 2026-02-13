# Phase 10 - Keyword Enforcement

## Objective
Ensure lexer and parser enforce the keyword policy consistently.

## Inputs
- `docs/keyword-policy.md`

## Deliverables
- Lexer rejects reserved keywords as identifiers.
- Parser rejects `import` and `match` in v0.1.

## Steps
- Add keyword tables to lexer.
- Add parser rejection paths for reserved keywords.

## Exit Criteria
- Keyword policy tests pass.
