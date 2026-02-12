# Atlas Top-Level Execution Rules

## Purpose
Lock rules for top-level execution order and function hoisting.

## Rules
- Top-level statements execute in source order.
- Top-level function declarations are hoisted.
- Variables must be declared before use (no forward reference).

## Test Plan
- Calling a function before its declaration works.
- Using a variable before declaration produces `AT0002`.
- Side effects from top-level statements occur in order.
