# Atlas Numeric Edge Cases

## Purpose
Lock behavior for numeric edge cases and avoid runtime surprises.

## Rules
- Divide by zero is a runtime error (`AT0005`).
- Any NaN or Infinity result is a runtime error (`AT0007`).

## Test Plan
- `1 / 0` -> `AT0005`
- `0 / 0` -> `AT0005` (before NaN)
- `1e308 * 1e308` -> `AT0007`
