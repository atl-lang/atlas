# Atlas Codebase Patterns

**Purpose:** Documented patterns from actual Atlas codebase for AI agents.

---

## Atlas Grammar Quick Reference

```
// Control flow — ALL require parens around condition (except for-in)
if (cond) { ... }                       if (cond) { ... } else { ... }
while (cond) { ... }                    for (init; cond; step) { ... }
for item in iterable { ... }            // NO parens

// Functions
fn name(p: type, ...) -> RetType { }    // No -> = parser stores "null" (not "void")

// Types
number, string, bool, null              // Primitives
Type<T1, T2>                            // Generics: Result<number, string>
type[]                                  // Array: number[]
(T1, T2) -> T3                          // Function type (parens, NOT fn keyword)

// Variables
let x = 5;                              // Immutable
var x = 5;                              // Mutable
let x: number = 5;                      // With annotation

// Operators & assignment
x++;  x--;  x += 1;  x -= 1;           x = expr;  arr[0] = expr;

// Match, modules, comments
match expr { pattern => body, ... }
import { a, b } from "./path";          import * as ns from "./path";
export fn name() { }                    export let x = 5;
// line    /* block */    /// doc (3 slashes; //// is NOT doc)
```

---

## Collection Types

**Pattern:** `Arc<Mutex<X>>` — thread-safe shared mutable state (phase-18, DR-009)

| Value variant | Rust type | Access |
|--------------|-----------|--------|
| `Array` | `Arc<Mutex<Vec<Value>>>` | `.lock().unwrap()` |
| `HashMap` | `Arc<Mutex<AtlasHashMap>>` | `.lock().unwrap()` |
| `HashSet` | `Arc<Mutex<AtlasHashSet>>` | `.lock().unwrap()` |
| `Queue` | `Arc<Mutex<AtlasQueue>>` | `.lock().unwrap()` |
| `Stack` | `Arc<Mutex<AtlasStack>>` | `.lock().unwrap()` |
| `String` | `Arc<String>` | Immutable, no lock |

**Never:** `.borrow()` / `.borrow_mut()` — that's the old `Rc<RefCell<>>` pattern.

---

## Intrinsic vs Stdlib Function

| | Intrinsic | Stdlib function |
|---|-----------|----------------|
| **When** | Needs execution context (callbacks) | Pure operations (no callbacks) |
| **Registration** | `is_array_intrinsic()` in `stdlib/mod.rs` | `is_builtin()` + `call_builtin()` in `stdlib/mod.rs` |
| **Interpreter** | `eval_expr()` match → `self.intrinsic_X()` | `eval_call()` match → `module::func()` |
| **VM** | `execute_call_intrinsic()` → `self.vm_intrinsic_X()` | `execute_call_builtin()` → `module::func()` |
| **Callback call** | Interpreter: `self.call_value()` / VM: `self.vm_call_function_value()` | N/A |
| **Examples** | `map`, `filter`, `forEach`, `hashMapForEach` | `hashMapNew`, `hashMapPut`, `abs`, `len` |

### Intrinsic return patterns

- **forEach:** iterate → call callback → return `Value::Null`
- **map:** iterate → call callback → collect into new collection
- **filter:** iterate → call predicate → keep truthy → new collection (same type; exception: `hashSetMap` → Array)

### Callback signatures

- HashMap: `fn(value, key)` — value first (JS convention)
- HashSet/Array: `fn(elem)` — single argument

---

## Error Pattern

```rust
// Type error (most common)
Err(RuntimeError::TypeError { msg: "descriptive message".into(), span })
// Unknown function
Err(RuntimeError::UnknownFunction { name: name.into(), span })
// Index out of bounds
Err(RuntimeError::IndexOutOfBounds { index: idx, span })
```

Always include: descriptive `msg` + `span` for source location.

### Type extraction helper

```rust
fn expect_hashmap(value: &Value, span: Span) -> Result<Arc<Mutex<AtlasHashMap>>, RuntimeError> {
    match value {
        Value::HashMap(m) => Ok(Arc::clone(m)),
        _ => Err(RuntimeError::TypeError {
            msg: format!("Expected HashMap, got {}", value.type_name()), span,
        }),
    }
}
```

---

## Test Harness Pattern

```rust
fn run(code: &str) -> Result<String, String> {
    let (tokens, _) = Lexer::new(code).tokenize();
    let (ast, _) = Parser::new(tokens).parse();
    let mut interp = Interpreter::new();
    let security = SecurityContext::allow_all();
    interp.eval(&ast, &security).map(|v| format!("{:?}", v)).map_err(|e| format!("{:?}", e))
}
```

Each domain test file already has a canonical helper — use it, don't redefine.

---

## Summary

- **Intrinsic:** needs callbacks/context → implement in both interpreter AND VM
- **Stdlib:** pure operation → implement in module, call from both engines
- **Parity:** both engines MUST produce identical results — always test both
