# Atlas REPL State

## Purpose
Ensure REPL maintains state across inputs.

## Rules
- Declarations persist across inputs.
- Errors do not reset state.

## Test Plan
- Define variable in one input, use in next.
- Define function in one input, call in next.
