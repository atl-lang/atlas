# Atlas CLI End-to-End

## Purpose
Ensure CLI commands run the full pipeline correctly.

## Commands
- `atlas run file.atl`
- `atlas build file.atl`
- `atlas repl`

## Test Plan
- Run simple program and verify output.
- Type error should return non-zero exit code.
- Bytecode build should emit `.atb`.
