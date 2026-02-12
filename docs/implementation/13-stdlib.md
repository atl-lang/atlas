# Standard Library Implementation

Implementation strategy for `print`, `len`, `str` builtins.

## Stdlib Module

```rust
// stdlib.rs
use crate::value::Value;
use std::rc::Rc;

pub fn call_builtin(name: &str, args: &[Value]) -> Result<Value, String> {
    match name {
        "print" => builtin_print(args),
        "len" => builtin_len(args),
        "str" => builtin_str(args),
        _ => Err(format!("Unknown builtin function: {}", name)),
    }
}

fn builtin_print(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("print expects 1 argument, got {}", args.len()));
    }

    let output = match &args[0] {
        Value::String(s) => s.as_ref().clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        _ => return Err("print: invalid argument type".to_string()),
    };

    println!("{}", output);
    Ok(Value::Null)
}

fn builtin_len(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("len expects 1 argument, got {}", args.len()));
    }

    let length = match &args[0] {
        Value::String(s) => s.chars().count() as f64,  // Unicode scalar count
        Value::Array(arr) => arr.borrow().len() as f64,
        _ => return Err("len: argument must be string or array".to_string()),
    };

    Ok(Value::Number(length))
}

fn builtin_str(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("str expects 1 argument, got {}", args.len()));
    }

    let string = match &args[0] {
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        _ => return Err("str: invalid argument type".to_string()),
    };

    Ok(Value::String(Rc::new(string)))
}
```

## Type Signatures

Add these to the symbol table during initialization:

```rust
impl SymbolTable {
    fn define_builtin(&mut self, name: &str) {
        let ty = match name {
            "print" => Type::Function {
                params: vec![Type::String],  // Simplified for v0.1
                return_type: Box::new(Type::Void),
            },
            "len" => Type::Function {
                params: vec![Type::String],  // Simplified
                return_type: Box::new(Type::Number),
            },
            "str" => Type::Function {
                params: vec![Type::Number],  // Simplified
                return_type: Box::new(Type::String),
            },
            _ => return,
        };

        self.functions.insert(name.to_string(), Symbol {
            name: name.to_string(),
            ty,
            mutable: false,
            kind: SymbolKind::Builtin,
            span: DUMMY_SPAN,
        });
    }
}
```

## VM Integration

For the VM, builtins can be called via a special `Call` opcode path:

```rust
impl VM {
    fn call_function(&mut self, arg_count: usize) -> Result<(), RuntimeError> {
        let callee = self.peek(arg_count);

        match callee {
            Value::Function(func_ref) => {
                // Check if it's a builtin
                if func_ref.bytecode_offset == 0 {
                    // Builtin function
                    let args: Vec<Value> = (0..arg_count)
                        .map(|_| self.pop())
                        .collect();
                    let result = crate::stdlib::call_builtin(&func_ref.name, &args)
                        .map_err(|_| RuntimeError::InvalidStdlibArgument)?;
                    self.pop(); // Pop function
                    self.push(result);
                } else {
                    // User-defined function
                    // ... handle normal call frame
                }
                Ok(())
            }
            _ => Err(RuntimeError::TypeError),
        }
    }
}
```

## Error Codes

Stdlib errors use code `AT0102` (invalid stdlib argument).

## Key Design Decisions

- **Rust functions:** Stdlib implemented as native Rust functions
- **Unicode length:** `len` counts Unicode scalar values, not bytes
- **Type checking:** Typechecker validates args at compile time
- **Runtime validation:** Stdlib functions double-check at runtime
- **Error messages:** Include function name in error
