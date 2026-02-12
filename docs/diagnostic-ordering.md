# Atlas Diagnostic Ordering

## Purpose
Ensure deterministic ordering for diagnostics output.

## Rules
- Errors before warnings.
- Within same level: sort by file, then line, then column.

## Test Plan
- Input with multiple errors yields sorted output.
- Mixed errors/warnings follow ordering rules.
