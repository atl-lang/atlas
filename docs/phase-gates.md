# Atlas Phase Gates

## Purpose
Ensure features exist before tests and prevent premature or stubbed work.

## Gate Rules
- A test phase may not start until the corresponding feature phase is complete.
- Diagnostics must be implemented before any diagnostics test phase.
- Bytecode format must be defined before bytecode tests.

## Required Evidence
- Passing minimal end-to-end examples for the feature.
- Diagnostics for invalid inputs are emitted with correct codes.
- No TODO/TBD markers in feature code.

## Enforcement
- Build order in `phases/BUILD-ORDER.md` must respect these gates.
- If a gate is not satisfied, pause and update plan before proceeding.
