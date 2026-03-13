# Atlas JIT Compiler

**Crate:** `atlas-jit` (`crates/atlas-jit/src/`)
**Backend:** Cranelift (via `cranelift-jit`, `cranelift-codegen`, `cranelift-module`)
**Status:** Foundation complete. Arithmetic-only functions supported. Control flow and global access pending (Block 7).

---

## Architecture Overview

```
VM dispatch loop
    │
    │ (on every Call opcode for numeric functions)
    ▼
JitEngine::notify_call(function_offset, bytecode, function_end, args)
    │
    ├─ [cache hit]  → call_jit_function(code_ptr, args) → f64
    │
    ├─ [hot, not cached] → IrTranslator::translate(bytecode, offset, end)
    │                          → cranelift IR Function
    │                    → NativeBackend::compile(ir_func)
    │                          → CompiledFunction { code_ptr: *const u8 }
    │                    → CodeCache::insert(offset, code_ptr, size, param_count)
    │                    → call_jit_function(code_ptr, args) → f64
    │
    └─ [cold or unsupported] → None → VM falls back to bytecode interpretation
```

The VM holds `jit: Option<Box<dyn JitCompiler>>`. When set, the VM calls `try_execute()` before interpreting a function. If the JIT returns `Some(f64)` the result is used directly; if it returns `None` the VM interprets the function body normally. This fallback is always safe.

The `JitCompiler` trait is defined in `crates/atlas-runtime/src/jit_trait.rs` and re-exported from `atlas-runtime`. `JitEngine` implements it.

---

## Components

### JitEngine (`lib.rs`)

The top-level engine. Owns all sub-components:

```rust
pub struct JitEngine {
    config: JitConfig,
    tracker: HotspotTracker,
    cache: CodeCache,
    backend: NativeBackend,
    translator: IrTranslator,
    compilations: u64,
    jit_executions: u64,
    interpreter_fallbacks: u64,
}
```

**`JitConfig` defaults:**
- `compilation_threshold: 100` — calls before JIT compilation triggers
- `cache_size_limit: 64 MB`
- `opt_level: 1` (Cranelift "speed")
- `enabled: true`

`for_testing()` drops the threshold to 2 and cache to 4 MB.

**`notify_call(function_offset, bytecode, function_end, args) -> Option<f64>`:**
1. Record the call in `HotspotTracker`.
2. If `CodeCache::contains(offset)` → call native code directly.
3. Else if `HotspotTracker::is_hot(offset)` → compile, cache, execute.
4. Else → return `None`.

On compilation failure: `mark_compiled(offset)` prevents retry on every subsequent call.

**ABI constraint:** JIT only handles numeric functions. Args are `&[f64]`; result is `f64`. Functions with non-numeric args or any unsupported opcode bail to `None`. Maximum arity supported: 6 parameters.

---

### HotspotTracker (`hotspot.rs`)

Counts calls per function (keyed by `bytecode_offset`) and determines when to compile.

```rust
pub struct HotspotTracker {
    function_counts: HashMap<usize, u64>, // offset → call count
    threshold: u64,
    compiled: HashMap<usize, bool>,       // offset → compiled flag
}
```

- `record_call(offset)` — increments count
- `is_hot(offset) -> bool` — `call_count >= threshold && !is_compiled`
- `mark_compiled(offset)` — prevents recompilation
- `pending_compilations() -> Vec<HotFunction>` — sorted by call count, highest first

`extract_function_boundaries(bytecode) -> Vec<FunctionBoundary>` scans the bytecode stream for `Constant(FunctionRef)` + `Return` pairs to identify function start/end offsets.

---

### IrTranslator (`codegen.rs`)

Translates a slice of Atlas bytecode (from `offset` to `end`) into a Cranelift `ir::Function`.

**Supported opcodes (translate to native code):**
`Constant`, `True`, `False`, `Null`, `Add`, `Sub`, `Mul`, `Div`, `Mod`, `Negate`, `Equal`, `NotEqual`, `Less`, `LessEqual`, `Greater`, `GreaterEqual`, `Not`, `GetLocal`, `SetLocal`, `Pop`, `Dup`, `Return`, `Halt`

**Unsupported opcodes (return `Err(JitError::UnsupportedOpcode(op))`):**
`GetGlobal`, `SetGlobal`, `Jump`, `JumpIfFalse`, `Loop`, `Call`, `And`, `Or`, `Array`, `GetIndex`, `SetIndex`, `IsOptionSome`, `IsOptionNone`, `IsResultOk`, `IsResultErr`, `ExtractOptionValue`, `ExtractResultValue`, `IsArray`, `GetArrayLen`, and all closure/struct/async opcodes.

Note: `And`/`Or` opcodes are not needed — the compiler uses `JumpIfFalse` for short-circuit evaluation, so they are never actually emitted.

**Translation approach:**
- `translate(bytecode, offset, end)` — no parameters (zero-arg function)
- `translate_with_params(bytecode, offset, end, param_count)` — function with `param_count` f64 parameters

Atlas numbers are `f64` (Value::Number). The translator works exclusively with Cranelift `f64` values. Non-numeric types bail out.

---

### NativeBackend (`backend.rs`)

Wraps Cranelift's `JITModule`. Configured for the host architecture at construction time.

```rust
pub struct NativeBackend {
    module: JITModule,
    compiled_count: usize,
    native_bytes: usize,
}
```

`NativeBackend::new(opt_level)` configures a `cranelift_native::builder()` ISA (auto-detected host architecture) and creates a `JITModule`. Opt levels map to Cranelift `"none"` / `"speed"` / `"speed_and_size"`.

`compile(ir_func) -> JitResult<CompiledFunction>` — declares the function in the JIT module, defines it from Cranelift IR, finalizes definitions, and returns a `code_ptr: *const u8` as the entry point.

---

### CodeCache (`cache.rs`)

Maps `bytecode_offset → CacheEntry`.

```rust
pub struct CacheEntry {
    pub code_ptr: *const u8,
    pub code_size: usize,
    pub version: u64,          // For invalidation
    pub hit_count: u64,
    pub param_count: usize,    // Arity check before calling
}
```

- `get(offset) -> Option<&CacheEntry>` — version-checks and increments hit count
- `contains(offset) -> bool` — version-check only (no hit counting)
- `insert(offset, code_ptr, code_size, param_count)` — checks size limit, rejects if full
- `invalidate_all()` — increments `self.version`; all existing entries become stale on next `get`

`code_ptr` is `unsafe impl Send + Sync` — native code pointers are read-only after compilation.

Size estimate: ~20 bytes of native code per bytecode byte. Max 64 MB of cached native code.

---

## JIT Integration with VM

The VM wires in the JIT via:

```rust
vm.jit = Some(Box::new(JitEngine::new(JitConfig::default())?));
```

At each `Call` opcode, if `jit.is_some()` and all arguments are `Value::Number`, the VM calls:

```rust
jit.try_execute(bytecode, function_offset, function_end, &numeric_args)
```

If `Some(f64)` is returned, it is pushed to the stack as `Value::Number`. If `None`, the VM pushes a new `CallFrame` and begins interpreting the function body normally.

Worker VMs (`VM::new_for_worker()`) do not inherit the JIT — `jit: None` on workers. This is intentional: JIT compilation is not thread-safe in the current Cranelift `JITModule` design.

---

## Cache Invalidation

`invalidate_all()` increments the cache version counter. This invalidates all entries simultaneously without memory deallocation. Subsequent `contains()` / `get()` calls find no valid entries and the tracker re-initiates compilation.

Invalidation is needed for REPL sessions where bytecode changes between evaluations. Block 7 will wire invalidation to bytecode change events.

---

## Block 7 Scope (Not Yet Implemented)

Block 7 adds production-grade JIT:

1. `Jump`, `JumpIfFalse`, `Loop` in `codegen.rs` — enables loop compilation
2. `Call` opcode — indirect dispatch to compiled or interpreted functions
3. `GetGlobal` / `SetGlobal` — access the VM's globals table via pointer
4. Wire hotspot threshold check into the VM execute loop (currently manual)
5. JIT cache invalidation tied to bytecode changes in REPL

Until Block 7, the JIT provides speedup only for leaf arithmetic functions (pure numeric functions with no control flow, global access, or function calls).

---

## JitStats

```rust
pub struct JitStats {
    pub compilations: u64,
    pub jit_executions: u64,
    pub interpreter_fallbacks: u64,
    pub cached_functions: usize,
    pub cache_bytes: usize,
    pub cache_hit_rate: f64,
    pub tracked_functions: usize,
    pub compiled_functions: usize,
}
```

Retrieved via `JitEngine::stats()` or the `JitCompiler` trait's `stats()` method (which returns the runtime-facing `atlas_runtime::JitStats` with a subset of fields).

---

## Invariants

- **Graceful fallback is mandatory.** Any unsupported opcode returns `Err(JitError::UnsupportedOpcode(...))` — never panics. The VM always falls back to interpretation on `None`.
- **JIT output must be identical to VM output** for all supported opcodes. JIT is an optimization; it must never change observable behavior.
- **No JIT in tests by default.** `atlas-runtime` tests run interpreted. JIT-specific tests live in `crates/atlas-jit/` and test the JIT engine directly via `cargo nextest run -p atlas-jit`.
