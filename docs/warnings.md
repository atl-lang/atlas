# Atlas Warnings

## Policy
Warnings are non-fatal diagnostics. They follow the same schema as errors.

## Warning Codes (v0.1)
- `AT2001` Unused variable
- `AT2002` Unreachable code

## Emission Rules
- Warnings should not block execution.
- Warnings are emitted even if errors exist.
- Identifiers starting with `_` do not trigger unused warnings.
- Unused parameters are warned unless prefixed with `_`.
