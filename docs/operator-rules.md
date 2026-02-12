# Atlas Operator Rules

## Purpose
Define operator validity and typing rules for consistency.

## Rules
- Arithmetic: `+ - * / %` only for `number`.
- String concat: `string + string` only.
- Comparison: `< <= > >=` only for `number`.
- Equality: `== !=` only for same-type operands.
- Logical: `&& || !` only for `bool`.

## Test Plan
- Valid and invalid cases for each operator.
- Ensure diagnostics use `AT0001` (type mismatch).
