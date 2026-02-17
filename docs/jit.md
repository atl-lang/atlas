# Atlas JIT Compilation

## Overview

The Atlas JIT engine compiles hot bytecode functions to native machine code using [Cranelift](https://cranelift.dev/) as the code generation backend. This provides 5-10x performance improvements for computation-heavy numeric code.

## Architecture

```
┌──────────────┐     ┌───────────────┐     ┌──────────────┐
│  VM Profiler  │────▶│ Hotspot       │────▶│ IR           │
│  (execution   │     │ Tracker       │     │ Translator   │
│   counts)     │     │ (threshold    │     │ (bytecode →  │
│               │     │  detection)   │     │  Cranelift)  │
└──────────────┘     └───────────────┘     └──────┬───────┘
                                                   │
┌──────────────┐     ┌───────────────┐     ┌──────▼───────┐
│  VM Dispatch  │◀────│ Code Cache    │◀────│ Native       │
│  (JIT or      │     │ (LRU evict,   │     │ Backend      │
│   interpret)  │     │  versioning)  │     │ (Cranelift   │
│               │     │               │     │  JIT module) │
└──────────────┘     └───────────────┘     └──────────────┘
```

## Components

### Hotspot Tracker (`hotspot.rs`)
- Tracks function execution counts by bytecode offset
- Configurable compilation threshold (default: 100 calls)
- Returns pending compilations sorted by call count (highest first)
- Marks compiled functions to avoid recompilation

### IR Translator (`codegen.rs`)
- Translates Atlas bytecode to Cranelift IR
- Supports numeric operations: `Add`, `Sub`, `Mul`, `Div`, `Mod`, `Negate`
- Supports comparisons: `Equal`, `NotEqual`, `Less`, `LessEqual`, `Greater`, `GreaterEqual`
- Supports logical: `Not`
- Supports locals: `GetLocal`, `SetLocal`
- Supports stack: `Dup`, `Pop`
- Constants: `Constant` (numeric only), `True`, `False`, `Null`
- Unsupported opcodes (globals, arrays, calls) fall back to interpreter

### Native Backend (`backend.rs`)
- Uses Cranelift JIT module for native code generation
- Targets host architecture (x86_64, aarch64)
- Configurable optimization level (none, speed, speed_and_size)
- Returns callable function pointers

### Code Cache (`cache.rs`)
- Caches compiled native code keyed by bytecode offset
- Configurable size limit (default: 64 MB)
- LRU eviction of cold (least-used) entries
- Version-based invalidation
- Hit/miss tracking and hit rate reporting

### JIT Engine (`lib.rs`)
- Top-level orchestrator integrating all components
- `notify_call()` — records calls, triggers compilation, dispatches to native code
- Returns `Some(result)` for JIT execution, `None` for interpreter fallback
- Statistics reporting via `stats()`

## Usage

```rust
use atlas_jit::{JitEngine, JitConfig};

// Create engine with default config
let mut engine = JitEngine::new(JitConfig::default())?;

// In VM execution loop, for each function call:
if let Some(result) = engine.notify_call(func_offset, &bytecode, func_end) {
    // Use JIT result
    stack.push(Value::Number(result));
} else {
    // Fall back to interpreter
    interpret_function(func_offset);
}

// Check performance stats
let stats = engine.stats();
println!("JIT compilations: {}", stats.compilations);
println!("Cache hit rate: {:.1}%", stats.cache_hit_rate * 100.0);
```

## Configuration

| Parameter | Default | Description |
|-----------|---------|-------------|
| `compilation_threshold` | 100 | Calls before JIT compilation |
| `cache_size_limit` | 64 MB | Maximum native code cache size |
| `enabled` | true | Whether JIT is active |
| `opt_level` | 1 (speed) | Cranelift optimization level |

## Tiered Compilation

Functions follow a tiered execution model:

1. **Interpreted** — All functions start interpreted
2. **Profiled** — Execution counts tracked per function
3. **Compiled** — Hot functions compiled to native code
4. **Cached** — Native code cached for subsequent calls

If compilation fails (unsupported opcodes), the function permanently falls back to the interpreter.

## Supported Opcodes

| Category | Opcodes | JIT Support |
|----------|---------|-------------|
| Constants | `Constant`, `True`, `False`, `Null` | Numeric only |
| Arithmetic | `Add`, `Sub`, `Mul`, `Div`, `Mod`, `Negate` | Full |
| Comparison | `Equal`, `NotEqual`, `Less`, `LessEqual`, `Greater`, `GreaterEqual` | Full |
| Logical | `Not` | Full |
| Variables | `GetLocal`, `SetLocal` | Full |
| Stack | `Dup`, `Pop` | Full |
| Control | `Return`, `Halt` | Full |
| Globals | `GetGlobal`, `SetGlobal` | Interpreter fallback |
| Functions | `Call` | Interpreter fallback |
| Arrays | `Array`, `GetIndex`, `SetIndex` | Interpreter fallback |
| Pattern | `IsOptionSome`, etc. | Interpreter fallback |

## Performance

JIT compilation targets pure numeric computations (arithmetic, comparisons, local variables). These operations see 5-10x speedup over interpretation because:

- No opcode dispatch overhead
- No value boxing/unboxing
- Native floating-point operations
- Register allocation by Cranelift
- CPU branch prediction works on native code

Functions using strings, arrays, globals, or function calls remain interpreted since they require runtime support that can't be directly compiled.

## Testing

103 tests cover:
- All arithmetic operations with edge cases
- All comparison operations
- Stack manipulation
- Local variable access
- Error handling (unsupported opcodes, stack underflow, non-numeric constants)
- Hotspot tracking lifecycle
- Code cache (insertion, eviction, invalidation, hit/miss tracking)
- Backend (multiple functions, optimization levels, target detection)
- Full pipeline (hotspot → compile → cache → execute)
- JitEngine integration (threshold, enable/disable, stats, reset)
- Performance regression (1M JIT calls < 500ms)
