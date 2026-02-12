# Atlas String Semantics

## Purpose
Lock string behavior for correctness and AI predictability.

## Rules
- Strings are UTF-8.
- `len(string)` returns Unicode scalar count (not bytes).
- String concatenation uses `+` only between strings.

## Test Plan
- ASCII and multi-byte length checks.
- Concatenation rules.
