# Spec: Compound Assignment Correctness (Indexed Targets)

Target: atlas-runtime compiler + VM
Owner: Codex audit
Status: Draft

## Goal
Ensure compound assignments on indexed targets evaluate target and index expressions exactly once, preserving side effects and parity with interpreter.

## Problem
The current compiler re-evaluates `target` and `index` to avoid missing stack rotation opcodes. This can double-execute side effects (e.g., `arr[f()] += 1`).

## Scope
In scope:
- Index compound assignments (`arr[idx] += x`, `arr[idx] -= x`, ...)
- Introduce minimal stack manipulation or temporary locals

Out of scope:
- General opcode ISA redesign

## Design
Option A (preferred): Introduce `Swap` / `Rotate` opcodes
- Add `Opcode::Swap` (swap top two stack values) and optionally `Opcode::Rot3` (rotate top 3).
- Compiler uses a single evaluation of `target` and `index`, then manipulates stack to reach `[array, index, result]` before `SetIndex`.
- VM + interpreter implement new opcodes with clear invariants.

Option B: Use temporary locals
- Emit bytecode that stores `target` and `index` into temporary locals once.
- Re-load as needed for `GetIndex` and `SetIndex`.
- Requires compiler-managed local slot reservation and cleanup.

## Acceptance Criteria
- No re-evaluation of index or target expressions.
- Parity tests pass (interpreter == VM) for side-effecting indices.
- No regression to bytecode size or performance without justification.

## Test Plan
- Add tests with side-effecting index expressions:
  - `arr[f()] += 1` where `f()` mutates a counter
  - `get_arr()[i()] += x` where `get_arr()` increments a global
- Verify output and side-effect counts in interpreter and VM.

## Files Likely Touched
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/compiler/stmt.rs
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/bytecode/mod.rs (if new opcode)
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/vm/mod.rs
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/interpreter/expr.rs
