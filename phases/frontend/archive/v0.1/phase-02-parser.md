# Phase 02 - Parser

## Objective
Parse tokens into the AST defined in `docs/ast.md`.

## Inputs
- `Atlas-SPEC.md`
- `docs/ast.md`
- `docs/diagnostics.md`

## Deliverables
- Recursive descent parser covering all v0.1 constructs.
- Diagnostics on first error per file.
- Assignment targets support identifiers and indexed expressions.

## Steps
- Implement parsing functions for expressions, statements, and declarations.
- Ensure operator precedence and associativity match spec.
- Attach spans to every AST node.
- Add parsing for array element assignment (`arr[i] = expr;`).

## Exit Criteria
- Parser golden tests pass.
- AST nodes include spans for all constructs.
