# atlas-jit/src/

Cranelift-based JIT compiler. Wires into the VM's hotspot profiler.
Block 7 (v0.3) completes this crate — adds control flow + wires to VM.

## File Map

| File | What it does |
|------|-------------|
| `lib.rs` | `JitEngine`, `JitConfig`, `JitError`, `JitStats` — public API |
| `hotspot.rs` | `HotspotTracker` — counts function calls, identifies compilation candidates |
| `codegen.rs` | Cranelift IR generation — translates Atlas bytecode → native via Cranelift |
| `cache.rs` | Compiled function cache — maps bytecode offset → native function pointer |
| `backend.rs` | Cranelift backend setup, module configuration |

## Current State (H-003 fixed, Block 7 pending)

**Supported opcodes** (implemented in codegen.rs):
`Constant`, `True`, `False`, `Null`, `Add`, `Sub`, `Mul`, `Div`, `Mod`, `Negate`,
`Equal`, `NotEqual`, `Less`, `LessEqual`, `Greater`, `GreaterEqual`, `Not`,
`GetLocal`, `SetLocal`, `Pop`, `Dup`, `Return`, `Halt`

**Unsupported opcodes** (bail out to interpreter — Block 7 adds these):
`GetGlobal`, `SetGlobal`, `Jump`, `JumpIfFalse`, `Loop`, `Call`,
and all collection/closure opcodes

**Note:** `And`/`Or` opcodes exist in the enum but are never emitted by the compiler.
The compiler uses `JumpIfFalse` for short-circuit evaluation instead.

**Threshold:** Default 100 invocations → compilation triggered.

**VM Integration:** JIT is wired via `VM::set_jit()`. When set, the VM calls
`JitCompiler::try_execute()` for hot numeric functions. Functions with
non-numeric args or unsupported opcodes fall back to interpretation.

**ABI:** Functions compile with correct arity (0-6 params supported). Cache
stores param_count and uses correct call convention.

## Block 7 Scope (what gets added)

1. `Jump`, `JumpIfFalse`, `Loop` opcodes in `codegen.rs` — enables loop compilation
2. `Call` opcode — indirect dispatch to compiled or interpreted functions
3. `GetGlobal`/`SetGlobal` — access VM's global value array via pointer
4. Wire `hotspot.rs` threshold check into VM execution loop
5. Replace interpreter loop for hot functions with native function pointer
6. JIT cache invalidation on bytecode change (REPL support)

**Note:** And/Or opcodes are not needed — compiler uses JumpIfFalse for short-circuit.

## Key Types

- `JitEngine` — top-level, holds tracker + cache + config
- `JitConfig` — `compilation_threshold: u64` (default 1000)
- `HotspotTracker` — `record_call(offset)` → `should_compile(offset) -> bool`
- `JitResult<T>` = `Result<T, JitError>`
- `JitError::UnsupportedOpcode(Opcode)` — graceful fallback signal

## Critical Rules

**Graceful fallback is required.** Any unsupported opcode must return
`Err(JitError::UnsupportedOpcode(...))` — never panic. The VM falls back to interpreted
execution on this error. This invariant must hold after every Block 7 phase.

**Parity with interpreter.** JIT output must be identical to interpreter output for all
supported opcodes. JIT is an optimization — it must never change observable behavior.

**No JIT in tests by default.** atlas-runtime tests run interpreted. JIT-specific tests
live in `crates/atlas-jit/` and test the JIT engine directly.

## Tests

Tests live in `crates/atlas-jit/` (no separate tests/ dir — inline in src or adjacent).
Run with: `cargo nextest run -p atlas-jit`
