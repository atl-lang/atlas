# Atlas Diagnostic Normalization

## Purpose
Ensure diagnostics are stable across machines and paths.

## Rules
- Strip absolute paths; use relative or filename only.
- Normalize line endings to `\n`.
- Remove volatile fields (timestamps, OS-specific paths).

## Test Plan
- Same input on two machines yields identical JSON output.
