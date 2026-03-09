---
paths:
  - "crates/atlas-runtime/src/interpreter/**"
  - "crates/atlas-runtime/src/vm/**"
  - "crates/atlas-runtime/src/compiler/**"
  - "crates/atlas-runtime/src/typechecker/**"
  - "crates/atlas-runtime/src/parser/**"
---

# Atlas Parity Rules

Auto-loaded when touching interpreter, VM, compiler, typechecker, or parser files.

## THREE Parity Contracts — All BLOCKING

### 1. Execution Parity (interpreter ↔ VM) — Original contract
Every behavior change must produce **identical output** in both execution engines:
- Interpreter: `crates/atlas-runtime/src/interpreter/`
- VM (bytecode): `crates/atlas-runtime/src/vm/` + `crates/atlas-runtime/src/compiler/`

If you touch one engine, you touch both. No exceptions.

### 2. Typechecker/Runtime Parity — NEW — equally binding

The typechecker's knowledge of every stdlib function return type MUST match what the
runtime actually returns. This is the most commonly violated parity rule.

**Rule:** If the runtime returns `Result<T, E>`, the typechecker must declare `Result<T, E>`.
If the runtime always returns a plain value, the typechecker must declare that value type.
`Type::Unknown` as a stdlib return type = a typechecker bug = P1 issue.

**Where to enforce:** `typechecker/expr.rs` → `resolve_namespace_return_type()`

When adding ANY stdlib function: update this function in the same commit.
When adding ANY stdlib function: verify the runtime return type matches the declared type.

### 3. Parser/Typechecker/Runtime Parity — NEW — equally binding

If a grammar rule exists in `docs/language/grammar.md`, it MUST be:
1. Implemented in the parser → produces an AST node
2. Handled in the typechecker → knows the type of that node
3. Evaluated in the interpreter → runs correctly
4. Compiled + executed in the VM → runs correctly with identical output

A grammar rule that exists in docs but not in the parser = P1 bug, file immediately.
A parser rule not handled in the typechecker = P1 bug, file immediately.
**"I'll add the typechecker later" = parity break = unacceptable.**

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
