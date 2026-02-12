# Atlas Testing

## Principles
- Tests must be deterministic.
- Prefer small, focused test inputs.
- Avoid flaky or time-sensitive assertions.

## Test Types
- Unit tests: individual components (lexer, parser, type checker, interpreter, VM).
- Golden tests: `.atl` input paired with expected output/diagnostics.
- Integration tests: CLI behaviors.

## Golden Test Conventions
- Inputs live in `tests/<category>/`.
- Expected output lives next to input:
  - `example.atl` -> `example.out` (stdout)
  - `example.atl` -> `example.diag` (diagnostics)
- If both output and diagnostics are needed, keep separate files.

## Diagnostic Tests
- For error cases, compare normalized diagnostic output.
- JSON diagnostics should be used for machine-stable assertions.

## Suggested Categories
- `tests/lexer`
- `tests/parser`
- `tests/typecheck`
- `tests/interpreter`
- `tests/vm`
- `tests/errors`
- `tests/ast`
- `tests/typecheck-dump`
