# Atlas Tests

## Layout
- `tests/lexer/` input -> token list
- `tests/parser/` input -> AST snapshot
- `tests/typecheck/` input -> diagnostics
- `tests/interpreter/` input -> output
- `tests/vm/` input -> output
- `tests/errors/` input -> diagnostics

## Golden Tests
- Each `.atl` test file has a corresponding `.out` or `.diag` file.
- Outputs must be deterministic.

## Running
- `cargo test` runs unit and golden tests.
