# Atlas JSON Dump Stability

## Purpose
Ensure AST and typecheck JSON dumps are deterministic and stable across environments.

## Rules
- Deterministic field ordering.
- Consistent formatting (no trailing spaces, stable indentation).
- Include version fields (`ast_version`, `typecheck_version`).

## Test Plan
- Re-run dumps and verify exact output match.
- Compare dumps across OSes.
