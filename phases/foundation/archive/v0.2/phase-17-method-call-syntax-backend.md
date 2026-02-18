# Phase 17: Method Call Syntax - Backend (Interpreter + VM)

## 🚨 DEPENDENCIES - CHECK BEFORE STARTING

**REQUIRED:** Phase 16 (Method Call Frontend) must be complete.

**Verification:**
```bash
# Parser parses member expressions
cargo test parser::tests | grep member

# Type checker validates methods
cargo test typechecker::tests | grep method

# Method table exists
grep "MethodTable\|method_table" crates/atlas-runtime/src/typechecker/*.rs
```

**What's needed:**
- MemberExpr parses correctly
- Type checker resolves methods
- Method table populated for JSON

**If missing:** Complete Phase 16 first

---

## Objective

Implement runtime evaluation and bytecode compilation for method calls. Enable method execution in both interpreter and VM with 100% parity. Methods desugar to stdlib function calls at runtime.

## Design

**Desugaring approach:**
```atlas
// Source
json["user"].as_string()

// Desugars to
jsonAsString(json["user"])

// Becomes function call to stdlib
```

**Mapping:**
- `json.as_string()` → `jsonAsString(value)`
- `json.as_number()` → `jsonAsNumber(value)`
- `json.as_bool()` → `jsonAsBool(value)`
- `json.is_null()` → `jsonIsNull(value)`

No new opcodes needed - method calls become regular function calls.

## Files

**Update:** `crates/atlas-runtime/src/interpreter/expr.rs` (~80 lines added)
**Update:** `crates/atlas-runtime/src/compiler/expr.rs` (~80 lines added)
**Update:** `crates/atlas-runtime/src/stdlib/json.rs` (extraction functions - DONE)
**Update:** `crates/atlas-runtime/src/stdlib/mod.rs` (register extraction functions)
**Tests:** `crates/atlas-runtime/tests/interpreter_member_tests.rs` (~200 lines)
**Tests:** `crates/atlas-runtime/tests/vm_member_tests.rs` (~200 lines)
**Tests:** Parity verification tests (~100 lines)

## Implementation

### 1. Stdlib Function Registration

Register JSON extraction functions in `stdlib/mod.rs`:

```rust
pub fn is_builtin(name: &str) -> bool {
    matches!(
        name,
        // ... existing functions ...
        // JSON extraction functions
        | "jsonAsString" | "jsonAsNumber" | "jsonAsBool" | "jsonIsNull"
    )
}

pub fn call_builtin(name: &str, args: &[Value], span: Span, security: &SecurityContext)
    -> Result<Value, RuntimeError>
{
    match name {
        // ... existing cases ...
        "jsonAsString" => json::json_as_string(args, span),
        "jsonAsNumber" => json::json_as_number(args, span),
        "jsonAsBool" => json::json_as_bool(args, span),
        "jsonIsNull" => json::json_is_null(args, span),
        // ...
    }
}
```

### 2. Interpreter - Method Evaluation

Desugar method calls to function calls:

```rust
Expr::Member(m) => {
    // 1. Evaluate target expression
    let target_value = self.eval_expr(&m.target)?;

    // 2. Build desugared function name
    let func_name = method_to_function_name(&target_value, &m.member.name);

    // 3. Build argument list (target + method args)
    let mut args = vec![target_value];
    if let Some(method_args) = &m.args {
        for arg in method_args {
            args.push(self.eval_expr(arg)?);
        }
    }

    // 4. Call stdlib function
    stdlib::call_builtin(&func_name, &args, m.span, self.security_context)?
}
```

**Helper function:**
```rust
fn method_to_function_name(target: &Value, method: &str) -> String {
    match target {
        Value::JsonValue(_) => format!("json{}", capitalize(method)),
        // Future: String, Array, etc.
        _ => panic!("Method on unsupported type"),
    }
}
```

### 3. Compiler - Bytecode Generation

Compile method calls to function call bytecode:

```rust
Expr::Member(m) => {
    // 1. Compile target expression
    self.compile_expr(&m.target)?;

    // 2. Compile arguments
    let arg_count = if let Some(args) = &m.args {
        for arg in args {
            self.compile_expr(arg)?;
        }
        args.len() + 1  // +1 for target
    } else {
        1  // Just target
    };

    // 3. Emit function call (desugared name)
    let func_name = method_to_function_name_static(&m);
    self.emit_call(&func_name, arg_count)?;
}
```

**Note:** Type checker ensures method is valid, so compiler can trust desugaring is correct.

### 4. Parity Verification

Every method call test must verify interpreter and VM produce identical results.

**Test pattern:**
```rust
#[rstest]
fn test_json_as_string_parity() {
    let code = r#"
        let data: json = parseJSON('{"name":"Alice"}');
        data["name"].as_string()
    "#;

    // Test interpreter
    let interp_result = eval_interpreter(code);

    // Test VM
    let vm_result = eval_vm(code);

    // Verify parity
    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Ok(Value::String("Alice".into())));
}
```

## Tests (TDD - Use rstest)

**Interpreter tests:**
1. JSON as_string() returns string
2. JSON as_number() returns number
3. JSON as_bool() returns bool
4. JSON is_null() returns bool
5. Chained method calls work
6. Error when extracting wrong type
7. Method on null errors

**VM tests:**
8-14. Identical to interpreter tests

**Parity tests:**
15. All basic extractions match
16. All error cases match
17. Chained calls match
18. Complex expressions match

**Integration tests:**
19. JSON in real programs
20. Multiple extractions
21. Method calls in conditions
22. Method calls in loops

**Minimum test count:** 50+ tests (25 interpreter, 25 VM)

## Integration Points

- Uses: Phase 16 (parsed + type-checked member expressions)
- Uses: stdlib/json.rs (extraction functions)
- Updates: Interpreter (expr evaluation)
- Updates: Compiler (bytecode generation)
- Updates: stdlib/mod.rs (function registration)
- Output: Working method calls in both engines

## Acceptance

- JSON extraction methods work in interpreter
- JSON extraction methods work in VM
- 100% interpreter/VM parity verified
- All 50+ tests pass
- Error messages match between engines
- Chained method calls work
- Method calls integrate with existing features
- No performance regression
- cargo test passes
- No clippy warnings

## Notes

**Method to function mapping:**
```
json.as_string()  → jsonAsString(json)
json.as_number()  → jsonAsNumber(json)
json.as_bool()    → jsonAsBool(json)
json.is_null()    → jsonIsNull(json)
```

**Future methods (not in Phase 17):**
- String methods: string.split(), string.trim(), etc.
- Array methods: array.length(), array.is_empty(), etc.
- These will be added in stdlib expansion phases

**This phase completes:**
- Full method call support (syntax to execution)
- JSON extraction (critical for Phase 06)
- Foundation for future method expansion

**After Phase 17:**
- Method calls are production-ready
- Can return to Phase 06 (stdlib integration tests)
- Can add more methods to stdlib as needed
