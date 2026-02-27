# Atlas Embedding Guide

**Version:** v0.2 | **Status:** Production Ready

Embedding Atlas lets Rust applications execute Atlas code, register native functions, enforce security policies, and interoperate with C via FFI.

---

## Table of Contents

- [Quick Start](#quick-start)
- [Runtime Modes](#runtime-modes)
- [Value System](#value-system)
- [Registering Native Functions](#registering-native-functions)
- [Security and Sandboxing](#security-and-sandboxing)
- [Error Handling](#error-handling)
- [FFI (C Interop)](#ffi-c-interop)
- [Reflection API](#reflection-api)
- [Performance Considerations](#performance-considerations)
- [Complete Examples](#complete-examples)
- [Best Practices](#best-practices)

---

## Quick Start

Add to `Cargo.toml`:

```toml
[dependencies]
atlas-runtime = { path = "path/to/atlas-runtime" }
```

### Minimal Embedding

```rust
use atlas_runtime::{Atlas, Value};

fn main() {
    let runtime = Atlas::new();
    let result = runtime.eval("1 + 2").unwrap();
    println!("{:?}", result); // Number(3.0)
}
```

---

## Runtime Modes

Atlas provides two API structs for different use cases.

### `Atlas` — Simple Interpreter Embedding

Best for simple scripting use cases where you need interpreter-only execution:

```rust
use atlas_runtime::{Atlas, Value};
use atlas_runtime::security::SecurityContext;

fn main() {
    // Default: deny-all security
    let runtime = Atlas::new();

    // Evaluate expressions
    let result = runtime.eval("len([1, 2, 3])").unwrap();
    assert_eq!(result, Value::Number(3.0));

    // Evaluate multi-line programs
    let result = runtime.eval_str(r#"
        fn double(x: number) -> number {
            return x * 2;
        }
        double(21)
    "#).unwrap();
    assert_eq!(result, Value::Number(42.0));

    // Evaluate a file
    let result = runtime.eval_file("script.atl").unwrap();
}
```

### `Runtime` — Full-Featured Embedding

Use when you need VM mode, persistent state across evaluations, or fine-grained control:

```rust
use atlas_runtime::api::{Runtime, ExecutionMode};

fn main() {
    // Interpreter mode (default)
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);

    // Persistent state: variables survive across eval calls
    runtime.eval("let counter = 0;").unwrap();
    runtime.eval("let counter = counter + 1;").unwrap();
    let result = runtime.eval("counter").unwrap();
    println!("{:?}", result); // Number(1.0)

    // VM mode (uses bytecode compiler + optimizer)
    let mut vm_runtime = Runtime::new(ExecutionMode::VM);
    vm_runtime.eval("let x = 42;").unwrap();
}
```

---

## Value System

Atlas values map to the `Value` enum in Rust:

```rust
pub enum Value {
    Number(f64),
    String(Arc<String>),
    Bool(bool),
    Null,
    Array(ValueArray),
    HashMap(ValueHashMap),
    Function(FunctionRef),
    Builtin(Arc<str>),
    // ... and more
}
```

### Converting Rust Values to Atlas

```rust
use atlas_runtime::Value;
use atlas_runtime::value::{ValueArray, ValueHashMap};

// Primitives
let num = Value::Number(42.0);
let s = Value::String(Arc::new("hello".to_string()));
let b = Value::Bool(true);
let n = Value::Null;

// Array (CoW value semantics)
let arr = Value::Array(ValueArray::from_vec(vec![
    Value::Number(1.0),
    Value::Number(2.0),
    Value::Number(3.0),
]));

// HashMap (CoW value semantics)
let mut map = atlas_runtime::stdlib::collections::hashmap::AtlasHashMap::new();
map.insert("name".to_string(), Value::String(Arc::new("Alice".to_string())));
map.insert("age".to_string(), Value::Number(30.0));
let obj = Value::HashMap(ValueHashMap::from_atlas(map));
```

### Converting Atlas Values to Rust

```rust
fn extract_number(val: &Value) -> Option<f64> {
    match val {
        Value::Number(n) => Some(*n),
        _ => None,
    }
}

fn extract_string(val: &Value) -> Option<String> {
    match val {
        Value::String(s) => Some(s.as_ref().clone()),
        _ => None,
    }
}

fn extract_array(val: &Value) -> Option<Vec<Value>> {
    match val {
        Value::Array(arr) => {
            let guard = arr.lock().unwrap();
            Some(guard.clone())
        }
        _ => None,
    }
}
```

---

## Registering Native Functions

Expose Rust functions to Atlas code using the `register_function` API:

```rust
use atlas_runtime::api::Runtime;
use atlas_runtime::value::{Value, RuntimeError};
use atlas_runtime::span::Span;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::stdlib::OutputWriter;

let mut runtime = Runtime::new(ExecutionMode::Interpreter);

// Register a simple function
runtime.register_function("greet", |args, span, _security, _output| {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument {
            msg: "greet() expects 1 argument".to_string(),
            span,
        });
    }
    match &args[0] {
        Value::String(name) => {
            let greeting = format!("Hello, {}!", name);
            Ok(Value::String(Arc::new(greeting)))
        }
        other => Err(RuntimeError::InvalidStdlibArgument {
            msg: format!("greet() expects string, got {}", other.type_name()),
            span,
        }),
    }
});

let result = runtime.eval(r#"greet("World")"#).unwrap();
// Value::String("Hello, World!")
```

### Native Function Signature

```rust
type BuiltinFn = fn(
    args: &[Value],           // Arguments passed from Atlas
    span: Span,               // Source location for error reporting
    security: &SecurityContext, // Current security context
    output: &OutputWriter,    // Output writer (for print-like functions)
) -> Result<Value, RuntimeError>;
```

### Stateful Native Functions

For functions that need access to host state, use closures via `Arc<Mutex<T>>`:

```rust
use std::sync::{Arc, Mutex};

let counter = Arc::new(Mutex::new(0u64));

// Clone the Arc for the closure
let counter_clone = Arc::clone(&counter);
runtime.register_function("increment", move |_args, _span, _, _| {
    let mut count = counter_clone.lock().unwrap();
    *count += 1;
    Ok(Value::Number(*count as f64))
});

let counter_clone2 = Arc::clone(&counter);
runtime.register_function("get_count", move |_args, _span, _, _| {
    let count = counter_clone2.lock().unwrap();
    Ok(Value::Number(*count as f64))
});

runtime.eval("increment(); increment(); increment();").unwrap();
let result = runtime.eval("get_count()").unwrap();
// Value::Number(3.0)
```

---

## Security and Sandboxing

Control what Atlas programs are allowed to do via `SecurityContext`:

```rust
use atlas_runtime::security::SecurityContext;

// Deny-all (default, most restrictive)
let ctx = SecurityContext::new();

// Allow everything (development only!)
let ctx = SecurityContext::allow_all();

// Fine-grained permissions
let ctx = SecurityContext::builder()
    .allow_filesystem_read()           // can read files
    .allow_filesystem_write()          // can write files
    .deny_network()                    // cannot make network requests
    .deny_process()                    // cannot spawn processes
    .allow_environment_read()          // can read env vars
    .deny_environment_write()          // cannot set env vars
    .build();
```

### Using Security with the Atlas Struct

```rust
use atlas_runtime::{Atlas, Value};
use atlas_runtime::security::SecurityContext;

// Read-only filesystem sandbox
let ctx = SecurityContext::builder()
    .allow_filesystem_read()
    .build();

let runtime = Atlas::new_with_security(ctx);

// This will work:
runtime.eval(r#"readFile("data.txt")"#).unwrap();

// This will fail with permission error:
runtime.eval(r#"writeFile("output.txt", "hello")"#);
// Err: permission denied: filesystem write not allowed
```

---

## Error Handling

All Atlas APIs return `Result<Value, Vec<Diagnostic>>` or `Result<Value, RuntimeError>`:

```rust
use atlas_runtime::diagnostic::{Diagnostic, Severity};

let runtime = Atlas::new();

match runtime.eval("let x = ;") {
    Ok(val) => println!("Result: {:?}", val),
    Err(diagnostics) => {
        for d in &diagnostics {
            eprintln!("[{}] {}", d.code, d.message);
            eprintln!("  at {}:{}", d.span.line, d.span.column);
        }
    }
}
```

### Diagnostic Structure

```rust
pub struct Diagnostic {
    pub severity: Severity,   // Error, Warning, Info
    pub code: String,         // e.g., "E0012"
    pub message: String,      // Human-readable message
    pub span: Span,           // Source location
    pub notes: Vec<String>,   // Additional context
}
```

### JSON Diagnostics

```rust
let result = runtime.eval_with_json_diagnostics("broken code");
// Returns JSON string of diagnostics for tooling integration
```

---

## FFI (C Interop)

Atlas supports calling C functions via FFI declarations in Atlas code:

### Declaring Extern Functions in Atlas

```atlas
// Declare C function from math library
extern "m" fn sin(x: CDouble) -> CDouble;
extern "m" fn cos(x: CDouble) -> CDouble;
extern "m" fn sqrt(x: CDouble) -> CDouble;

let result = sin(1.5707963);   // ~1.0
```

### Supported C Types

| Atlas FFI Type | C Equivalent | Notes |
|---------------|-------------|-------|
| `CInt` | `int` | 32-bit signed integer |
| `CLong` | `long` | Platform-native long |
| `CDouble` | `double` | 64-bit float |
| `CCharPtr` | `const char*` | Null-terminated string |
| `CVoid` | `void` | Return type only |
| `CBool` | `int` | C boolean convention |

### Callbacks (Atlas → C → Atlas)

Expose Atlas functions to C code using the callback manager:

```rust
use atlas_runtime::ffi::callbacks::CallbackManager;

let manager = CallbackManager::new();
let atlas_fn = runtime.get_function("my_callback").unwrap();

// Create a C-callable function pointer
let (ptr, handle) = manager.create_callback(atlas_fn);

// Pass ptr to a C library
unsafe { c_lib_register_callback(ptr) };

// Keep `handle` alive as long as the callback is registered!
// Dropping `handle` invalidates the function pointer.
```

---

## Reflection API

Inspect values and types at runtime from Rust:

```rust
use atlas_runtime::reflect::{TypeInfo, ValueInfo};

let val = runtime.eval("[1, 2, 3]").unwrap();

let info = ValueInfo::from(&val);
println!("Type: {}", info.type_name);        // "array"
println!("Length: {:?}", info.length);       // Some(3)
println!("Is primitive: {}", info.is_primitive); // false

// Type information
let type_info = TypeInfo::of(&val);
println!("Kind: {:?}", type_info.kind);
```

---

## Performance Considerations

### Choose the Right Mode

| Use Case | Recommended Mode | Reason |
|----------|-----------------|--------|
| Simple scripting | `Atlas` / `Interpreter` | Lower startup overhead |
| Compute-heavy loops | `VM` | Bytecode execution is faster |
| Frequent short evals | `Interpreter` | No compile overhead |
| Long-running programs | `VM` with optimizer | Best sustained throughput |

### Reuse the Runtime

```rust
// Bad: creates new runtime for each evaluation
for item in items {
    let rt = Atlas::new();
    rt.eval(&format!("process({})", item)).unwrap();
}

// Good: reuse runtime across evaluations
let mut rt = Runtime::new(ExecutionMode::Interpreter);
for item in items {
    rt.eval(&format!("process({})", item)).unwrap();
}
```

### Pre-compile Hot Functions

```rust
// Compile the function once, call it many times
let mut rt = Runtime::new(ExecutionMode::VM);
rt.eval(r#"
    fn process(x: number) -> number {
        return x * x + 2 * x + 1;
    }
"#).unwrap();

// Fast repeated calls
for i in 0..1_000_000 {
    rt.eval(&format!("process({})", i)).unwrap();
}
```

---

## Complete Examples

### Example 1: Template Engine

```rust
use atlas_runtime::{Atlas, Value};

fn render(template: &str, context: &[(&str, &str)]) -> String {
    let rt = Atlas::new();

    // Build context setup code
    let mut setup = String::new();
    for (key, val) in context {
        setup.push_str(&format!("let {} = \"{}\";\n", key, val));
    }

    // Template: replace {{var}} with Atlas interpolation
    let atlas_code = format!(
        "{}\n\"{}\"",
        setup,
        template.replace("{{", "\"+").replace("}}", "+\"")
    );

    match rt.eval(&atlas_code) {
        Ok(Value::String(s)) => s.as_ref().clone(),
        _ => template.to_string(),
    }
}
```

### Example 2: Configuration DSL

```rust
use atlas_runtime::{Atlas, Value};
use atlas_runtime::security::SecurityContext;

fn load_config(path: &str) -> serde_json::Value {
    let ctx = SecurityContext::builder()
        .allow_filesystem_read()
        .build();
    let rt = Atlas::new_with_security(ctx);

    let result = rt.eval_file(path).unwrap();

    // Convert Atlas object to JSON
    match result {
        Value::Object(obj) => {
            let guard = obj.lock().unwrap();
            // ... convert to serde_json::Value
            serde_json::json!({})
        }
        _ => serde_json::json!({}),
    }
}
```

### Example 3: Plugin System

```rust
use atlas_runtime::api::Runtime;
use atlas_runtime::api::ExecutionMode;

struct PluginHost {
    runtime: Runtime,
}

impl PluginHost {
    fn new() -> Self {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);

        // Register host API
        runtime.register_function("host_log", |args, span, _, _| {
            if let Some(Value::String(s)) = args.first() {
                println!("[PLUGIN] {}", s);
            }
            Ok(Value::Null)
        });

        PluginHost { runtime }
    }

    fn load_plugin(&mut self, path: &str) -> Result<(), String> {
        self.runtime.eval_file(path).map(|_| ()).map_err(|e| {
            format!("Plugin error: {:?}", e)
        })
    }

    fn call_hook(&mut self, hook: &str, data: &str) -> Option<String> {
        let code = format!("{}(\"{}\")", hook, data);
        match self.runtime.eval(&code) {
            Ok(Value::String(s)) => Some(s.as_ref().clone()),
            _ => None,
        }
    }
}
```

---

## Best Practices

### 1. Least-Privilege Security

```rust
// Never use allow_all() in production
let ctx = SecurityContext::builder()
    .allow_filesystem_read()    // only what the script needs
    .build();
```

### 2. Validate at the Boundary

```rust
runtime.register_function("set_name", |args, span, _, _| {
    let name = match args.first() {
        Some(Value::String(s)) => s.clone(),
        Some(other) => return Err(RuntimeError::InvalidStdlibArgument {
            msg: format!("expected string, got {}", other.type_name()),
            span,
        }),
        None => return Err(RuntimeError::InvalidStdlibArgument {
            msg: "set_name() requires 1 argument".to_string(),
            span,
        }),
    };

    if name.len() > 100 {
        return Err(RuntimeError::InvalidStdlibArgument {
            msg: "name too long (max 100 chars)".to_string(),
            span,
        });
    }

    Ok(Value::Null)
});
```

### 3. Keep Native Surface Small

Register only the functions Atlas code needs. A smaller API surface is easier to secure and maintain.

### 4. Avoid Global Mutable State

Prefer passing state through function arguments or using `Arc<Mutex<T>>` explicitly.

### 5. Return Typed Errors

Use `RuntimeError` with descriptive messages. Include the function name and what was expected vs. received.

---

*See also: [API Reference](api/stdlib.md) | [Security Model](security-model.md) | [CLI Reference](cli-reference.md)*
