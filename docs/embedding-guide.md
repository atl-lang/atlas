# Atlas Embedding Guide

**Status:** Implemented (Phases 01–02, 10a–10c, 12, 15)  
**Last Updated:** 2026-02-15

## Overview
Embedding Atlas lets host applications execute Atlas code, register native Rust functions, enforce security policies, and interoperate with C via FFI. This guide walks through setup, native function registration, security, callbacks, and examples.

## Quick Start (Rust)
```rust
use atlas_runtime::{Atlas, Value, Permissions};

fn main() -> anyhow::Result<()> {
    let mut rt = Atlas::with_config(Default::default());
    rt.set_permissions(Permissions::safe_defaults());
    rt.inject_native("host_log", |args, _span| {
        println!("LOG: {}", args[0]);
        Ok(Value::Null)
    })?;
    let out = rt.eval(r#"host_log("hi"); 1 + 2"#)?;
    println!("{out:?}");
    Ok(())
}
```

## Native Functions
- Register with `inject_native(name, func)`.
- Functions receive `&[Value]` and return `Result<Value, RuntimeError>`.
- Support for closures and state via captured environment.
- Error handling surfaces as diagnostics to the caller.

## Security & Sandbox
- Permissions configured via `Permissions` (filesystem, network, process, FFI, env, reflection).
- Defaults are restrictive; enable only what hosts need.
- Build scripts and native functions run under the same policy.

## FFI (C Interop)
- Extern types: `CInt`, `CLong`, `CDouble`, `CCharPtr`, `CVoid`, `CBool`.
- Declare externs in Atlas: `extern "m" fn sqrt(x: CDouble) -> CDouble;`
- Dynamic loading handled by `ffi::loader` with marshaling in `ffi::marshal`.
- Callbacks: use `ffi::callbacks::CallbackManager::create_callback` to expose Atlas functions to C; keep handle alive for pointer validity.

## Reflection
- `TypeInfo` / `ValueInfo` available to hosts; mirror functions exposed in `stdlib.reflect`.
- Useful for serializers, test discovery, and debugging.

## Examples
- `examples/embedding/01_hello_world.rs` – minimal eval
- `examples/embedding/02_custom_functions.rs` – native functions
- `examples/embedding/03_value_conversion.rs` – Value interop
- `examples/embedding/04_persistent_state.rs` – stateful host data
- `examples/embedding/05_error_handling.rs` – diagnostics
- `examples/embedding/06_sandboxing.rs` – permissions
- `examples/ffi/call_c_library.atl` – calling C
- `examples/ffi/c_callback_example.c` – callbacks from C

## Diagnostics
- All APIs return `Vec<Diagnostic>`; includes codes, spans, JSON serialization.

## Best Practices
- Keep native surface small and capability-scoped.
- Prefer deterministic functions; avoid global mutable state.
- Validate inputs at the boundary; return typed errors with spans.

