# Atlas Interpreter/VM Parity

## Purpose
Ensure interpreter and VM produce identical outputs and diagnostics.

## Rules
- Same input program yields same stdout and diagnostics.
- Runtime errors produce same error codes and spans.

## Test Plan
- Arithmetic + control flow program.
- Function calls and recursion.
- Array mutation and aliasing.
