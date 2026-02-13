# Phase 02 - Workspace Layout

## Objective
Define the concrete Rust workspace layout and module boundaries.

## Inputs
- `docs/guides/code-quality-standards.md`
- `docs/guides/code-quality-standards.md`

## Deliverables
- `crates/atlas-runtime` with submodules: `lexer`, `parser`, `ast`, `binder`, `types`, `typecheck`, `interpreter`, `bytecode`, `vm`, `diagnostics`.
- `crates/atlas-cli` with `repl` and `run` entrypoints.
- `docs/` references added to README.

## Steps
- Create empty module files per the list above.
- Ensure each module compiles with placeholder structs.
- Confirm no circular dependencies.

## Exit Criteria
- Workspace compiles with empty modules.
- Module boundaries reflect `docs/guides/code-quality-standards.md`.
