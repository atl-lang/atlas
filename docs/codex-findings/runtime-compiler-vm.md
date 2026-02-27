# Compiler/VM Parity and Correctness Risks

Target: atlas-runtime compiler + VM
Severity: High
Status: Open

## Finding 1: Compound assignment on indexed targets re-evaluates target/index expressions

Evidence:
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/compiler/stmt.rs:700-776

What/Why:
- The compiler handles compound assignment on index expressions by recompiling `target` and `index` multiple times to avoid missing stack-rotation opcodes. This changes evaluation order and can re-run side effects (function calls, getters, or index expressions with mutations).

Impact:
- Semantic divergence vs. interpreter and spec for expressions like `arr[f()] += 1` or `get_arr()[i()] += 1` where `f()` or `i()` have side effects. This breaks parity and can lead to incorrect results or double side effects.

Recommendation:
- Introduce stack rotation/swap opcodes (or a compiler-managed temp local) so target/index expressions are evaluated exactly once. Add parity tests for side-effecting indices.

---

## Finding 2: VM defines `And/Or` opcodes but executes them as UnknownOpcode

Evidence:
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/vm/mod.rs:894-912
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/vm/dispatch.rs:40-58

What/Why:
- The VM dispatch table includes `And` and `Or`, but the VM execution loop treats them as `UnknownOpcode` with a TODO for short-circuit logic. The compiler currently lowers `&&`/`||` using `JumpIfFalse` and `Not`, but any bytecode containing `And/Or` would fail at runtime.

Impact:
- Incompatible bytecode versions or tooling that emits `And/Or` will break at runtime, creating parity hazards and migration friction.

Recommendation:
- Either implement `And/Or` in VM or remove them from the opcode set/dispatch table. Add guardrails in bytecode serializer to prevent emission if unsupported.
