# Atlas Operator Rules

## Purpose
Define operator validity and typing rules for consistency.

## Rules
- Arithmetic: `+ - * / %` only for `number`.
- String concat: `string + string` only.
- Comparison: `< <= > >=` only for `number`.
- Equality: `== !=` only for same-type operands.
- Logical: `&& || !` only for `bool`.
- Compound assignment: `+= -= *= /= %=` only for `number` targets and values.
- Increment/Decrement: `++ --` only for `number` targets.
- All assignment operators require mutable targets.

## Test Plan
- Valid and invalid cases for each operator.
- Ensure diagnostics use `AT0001` (type mismatch).
- Ensure diagnostics use `AT3003` (immutability violations).
