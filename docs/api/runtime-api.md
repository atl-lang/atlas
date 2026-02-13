# Atlas Runtime API

## Purpose
Provide a stable minimal API for embedding Atlas in host applications.

## v0.1 Status
✅ **Implemented** - Core embedding API is available in `atlas-runtime` crate.

## Current API

### Core Runtime
```rust
use atlas_runtime::{Atlas, Value};

// Create runtime instance
let runtime = Atlas::new();

// Evaluate source code
let result: Result<Value, Vec<Diagnostic>> = runtime.eval("1 + 2");
```

### Type: `RuntimeResult<T>`
```rust
pub type RuntimeResult<T> = Result<T, Vec<Diagnostic>>;
```

### Available Functions
- `Atlas::new()` -> Create a new runtime instance
- `Atlas::eval(source: &str) -> RuntimeResult<Value>` -> Evaluate source code

## Value Interop
The `Value` enum is publicly exposed for host usage:

```rust
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(Rc<str>),
    Array(Rc<RefCell<Vec<Value>>>),
    Function(Rc<FunctionRef>),
}
```

## Diagnostics
Errors are returned as `Vec<Diagnostic>` with structured information:
- Error codes (e.g., "AT0001")
- Source location (file, line, column)
- Formatted messages
- JSON serialization support

## Future Enhancements
- `Atlas::eval_file(path: &str)` - Evaluate file contents
- `Atlas::set_stdout(writer)` - Custom output redirection
- C-ABI wrapper for embedding in non-Rust hosts
