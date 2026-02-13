# Phase 16 - Bytecode Format Tests

## Objective
Validate `.atb` serialization matches `docs/bytecode-format.md`.

## Inputs
- `docs/bytecode-format.md`

## Deliverables
- Tests that serialize and deserialize sample programs.
- Tests for version mismatch diagnostics.

## Steps
- Create fixtures for known bytecode outputs.
- Verify round-trip correctness.

## Exit Criteria
- Bytecode format tests pass.
