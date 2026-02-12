# Atlas Array Aliasing Semantics

## Purpose
Define mutation visibility and aliasing rules for arrays.

## Rules
- Arrays are reference-counted and shared by reference.
- Assignment and argument passing copy references (no deep copies).
- Mutation through one reference is visible to all aliases.

## Test Plan
- Assign array to another variable; mutate and verify both view changes.
- Pass array to function; mutate and verify caller sees change.
