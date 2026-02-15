# Atlas Runtime API

## Purpose
Stable embedding and runtime interaction API for host applications (Rust and FFI).

## Status (v0.2)
✅ Runtime API expansion (phase 01)
✅ Embedding API with native functions and sandboxing (phase 02)
✅ FFI extern types/loading/callbacks (phases 10a–10c)
✅ Reflection API exposure (phase 12)
✅ Security/permissions integration (phase 15)

## Core Runtime
```rust
use atlas_runtime::{Atlas, Value, RuntimeResult};

let mut runtime = Atlas::new();
let result: RuntimeResult<Value> = runtime.eval("1 + 2");
```

### API Surface
- `Atlas::new()` / `Atlas::with_config(config: RuntimeConfig)`
- `eval(source: &str) -> RuntimeResult<Value>`
- `eval_file(path: &Path) -> RuntimeResult<Value>`
- `call_function(name: &str, args: &[Value]) -> RuntimeResult<Value>`
- `set_stdout(writer: impl Write + Send + 'static)` / `set_stderr(...)`
- `set_permissions(permissions: Permissions)` (capability-based security)
- `inject_native(name, func: NativeFunction)` (register host functions)
- `register_callback(name, extern_sig) -> *const c_void` (FFI callbacks)
- `type_info(value: &Value) -> TypeInfo` (reflection)

### Value Interop (selected)
```rust
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(Rc<str>),
    Array(Rc<RefCell<Vec<Value>>>),
    Json(serde_json::Value),
    Function(Rc<FunctionRef>),
    NativeFunction(NativeFnHandle),
    Option(Option<Box<Value>>),
    Result(Result<Box<Value>, Box<Value>>),
}
```

### FFI Interop
- Extern types: `CInt`, `CLong`, `CDouble`, `CCharPtr`, `CVoid`, `CBool`
- Extern declarations via Atlas `extern "lib" fn ...` map to `ffi::ExternFn`
- Dynamic loading via `ffi::loader` and marshaling via `ffi::marshal`
- Callbacks: `ffi::callbacks::CallbackManager::create_callback` returns C-callable fn ptr from Atlas function

### Security
- Permissions model (`Permissions`): filesystem, network, process, FFI, environment, reflection scopes
- Sandbox enforcement in runtime and native/ffi execution paths

### Reflection
- `TypeInfo` / `ValueInfo` APIs exposed for hosts; stdlib reflect functions mirror them inside Atlas code

### Diagnostics
- Errors returned as `Vec<Diagnostic>` with codes, spans, JSON serialization

## Examples
- `examples/embedding/01_hello_world.rs` through `06_sandboxing.rs`
- `examples/ffi/call_c_library.atl`, `examples/ffi/c_callback_example.c`

## Compatibility
- Interpreter and VM parity guaranteed; API stable for v0.2
