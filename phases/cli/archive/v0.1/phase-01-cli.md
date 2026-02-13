# Phase 01 - CLI Tooling

## Objective
Deliver the `atlas` CLI with REPL/run/build commands.

## Inputs
- `Atlas-SPEC.md`

## Deliverables
- `atlas repl`
- `atlas run path/to/file.atl`
- `atlas build path/to/file.atl`

## Steps
- Implement command parsing with `clap`.
- Wire commands to runtime functions.

## Exit Criteria
- CLI runs scripts, compiles bytecode, and launches REPL.
