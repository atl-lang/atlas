---
paths: ["crates/atlas-runtime/src/interpreter/**", "crates/atlas-runtime/src/vm/**", "crates/atlas-runtime/src/compiler/**"]
---

# Atlas Parity Rules

Auto-loaded when touching interpreter, VM, or compiler files.

## Parity is Sacred — BLOCKING

Every behavior change must produce **identical output** in both execution engines:
- Interpreter: `crates/atlas-runtime/src/interpreter/`
- VM (bytecode): `crates/atlas-runtime/src/vm/` + `crates/atlas-runtime/src/compiler/`

**Parity break = BLOCKING. Never ship a phase with parity divergence.**

If you touch one engine, you touch both. No exceptions.

## Verifying Parity

```bash
# Run parity test suite
cargo nextest run -p atlas-runtime --test bytecode
cargo nextest run -p atlas-runtime --test stdlib

# Or use assert_parity in any test:
assert_parity(r#"your_atlas_code_here"#, "expected_output");
```

`assert_parity` runs code in both engines and asserts identical output. Use it for every new feature test.

## CoW Write-Back Pattern (DR-004)

Collection mutation builtins return a **new collection**. Both engines write it back to the caller's variable:
- Interpreter: `apply_cow_writeback()`
- VM: `emit_cow_writeback_if_needed()`

When adding a stdlib function that mutates a collection, both write-back paths must be updated.

## Adding a New Opcode

Every new opcode must be handled in:
1. `compiler/` — emit the opcode
2. `vm/mod.rs` — execute the opcode
3. `bytecode/` — serialize/deserialize
4. `optimizer/` — consider folding/peephole opportunity

Missing any of these = incomplete implementation. Build will catch 1-3. Optimizer is not caught by build — check manually.

## Failure Pattern

If `assert_parity` fails: the two engines produced different output. This is always a bug in one of them — find which diverges from the spec and fix it. Do not paper over it with a skip or `#[ignore]`.
