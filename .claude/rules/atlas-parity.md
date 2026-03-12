---
paths:
  - "crates/atlas-runtime/src/vm/**"
  - "crates/atlas-runtime/src/compiler/**"
  - "crates/atlas-runtime/src/typechecker/**"
  - "crates/atlas-runtime/src/parser/**"
---

# Atlas Parity Rules

Auto-loaded when touching VM, compiler, typechecker, or parser files.

**Note:** Since B36 (D-052), Atlas uses a unified Compiler+VM execution path. The interpreter
was removed. "Parity" now refers to consistency between typechecker knowledge and runtime
behavior, not two execution engines.

## TWO Parity Contracts — Both BLOCKING

### 1. Typechecker/Runtime Parity — Primary contract

The typechecker's knowledge of every stdlib function return type MUST match what the
runtime actually returns. This is the most commonly violated parity rule.

**Rule:** If the runtime returns `Result<T, E>`, the typechecker must declare `Result<T, E>`.
If the runtime always returns a plain value, the typechecker must declare that value type.
`Type::Unknown` as a stdlib return type = a typechecker bug = P1 issue.

**Where to enforce:** `typechecker/expr.rs` → `resolve_namespace_return_type()`

When adding ANY stdlib function: update this function in the same commit.
When adding ANY stdlib function: verify the runtime return type matches the declared type.

### 2. Parser/Typechecker/Runtime Parity — equally binding

If a grammar rule exists in `docs/language/grammar.md`, it MUST be:
1. Implemented in the parser → produces an AST node
2. Handled in the typechecker → knows the type of that node
3. Compiled by the compiler → emits correct bytecode
4. Executed by the VM → runs correctly

A grammar rule that exists in docs but not in the parser = P1 bug, file immediately.
A parser rule not handled in the typechecker = P1 bug, file immediately.
**"I'll add the typechecker later" = parity break = unacceptable.**

## Verifying Parity

```bash
# Run test suite
cargo nextest run -p atlas-runtime --test bytecode
cargo nextest run -p atlas-runtime --test stdlib
cargo nextest run -p atlas-runtime --test vm
```

## CoW Write-Back Pattern (DR-004)

Collection mutation builtins return a **new collection**. The VM writes it back to the
caller's variable via `emit_cow_writeback_if_needed()`.

When adding a stdlib function that mutates a collection, ensure the compiler emits
the write-back sequence correctly.

## Adding a New Opcode

Every new opcode must be handled in:
1. `compiler/` — emit the opcode
2. `vm/mod.rs` — execute the opcode
3. `bytecode/` — serialize/deserialize
4. `optimizer/` — consider folding/peephole opportunity

Missing any of these = incomplete implementation. Build will catch 1-3. Optimizer is
not caught by build — check manually.
