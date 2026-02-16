# Atlas Testing Patterns

**Purpose:** How to test Atlas runtime features effectively.

---

## Test Infrastructure

**Location:** `crates/atlas-runtime/tests/*.rs`

**Libraries:**
- **rstest:** Parameterized tests
- **insta:** Snapshot testing
- **proptest:** Property-based testing
- **pretty_assertions:** Better test output

---

## Integration Test Pattern (Standard)

**Use for:** Testing runtime behavior end-to-end

```rust
use atlas_runtime::interpreter::Interpreter;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::security::SecurityContext;

fn run(code: &str) -> Result<String, String> {
    let mut lexer = Lexer::new(code);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (ast, _) = parser.parse();
    let mut interpreter = Interpreter::new();
    let security = SecurityContext::allow_all();
    match interpreter.eval(&ast, &security) {
        Ok(val) => Ok(format!("{:?}", val)),
        Err(e) => Err(format!("{:?}", e)),
    }
}

#[test]
fn test_basic_operation() {
    let code = r#"
        let x = 42;
        x + 1
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "Number(43.0)");
}

#[test]
fn test_error_case() {
    let code = r#"
        hashMapPut("not a map", "key", "value")
    "#;
    let result = run(code);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("TypeError"));
}
```

---

## Testing Intrinsics (Callbacks)

**Challenge:** Atlas closures capture environment, callbacks execute in caller's scope.

### Pattern 1: Simple Callback

```rust
#[test]
fn test_map_intrinsic() {
    let code = r#"
        let arr = [1, 2, 3];
        let result = map(arr, fn(x) { x * 2; });
        result
    "#;
    let result = run(code).unwrap();
    assert!(result.contains("2.0"));
    assert!(result.contains("4.0"));
    assert!(result.contains("6.0"));
}
```

### Pattern 2: Multi-Argument Callback (HashMap)

```rust
#[test]
fn test_hashmap_foreach_callback_args() {
    let code = r#"
        let map = hashMapNew();
        hashMapPut(map, "a", 1);
        hashMapPut(map, "b", 2);
        let result = hashMapMap(map, fn(value, key) {
            value * 2;
        });
        hashMapGet(result, "a")
    "#;
    let result = run(code).unwrap();
    assert!(result.contains("2.0"));
}
```

**Key:** Callbacks receive `(value, key)` for maps, `(element)` for sets/arrays.

### Pattern 3: Testing Callback Errors

```rust
#[test]
fn test_callback_type_error() {
    let code = r#"
        let arr = [1, 2, 3];
        map(arr, "not a function")
    "#;
    let result = run(code);
    assert!(result.is_err());
}

#[test]
fn test_callback_runtime_error() {
    let code = r#"
        let arr = [1, 2, 3];
        map(arr, fn(x) { x / 0; })
    "#;
    let result = run(code);
    // Depending on error handling, may succeed or error
    // Document expected behavior
}
```

---

## Testing Collections

### Pattern 1: Basic Operations

```rust
#[test]
fn test_hashmap_put_get() {
    let code = r#"
        let map = hashMapNew();
        hashMapPut(map, "key", 42);
        hashMapGet(map, "key")
    "#;
    let result = run(code).unwrap();
    assert!(result.contains("42.0"));
}
```

### Pattern 2: Reference Semantics

```rust
#[test]
fn test_hashmap_reference_semantics() {
    let code = r#"
        let map1 = hashMapNew();
        let map2 = map1;  // Same reference
        hashMapPut(map1, "key", 100);
        hashMapGet(map2, "key")  // Should see the change
    "#;
    let result = run(code).unwrap();
    assert!(result.contains("100.0"));
}
```

### Pattern 3: Empty Collections

```rust
#[test]
fn test_empty_collection_edge_case() {
    let code = r#"
        let map = hashMapNew();
        let result = hashMapMap(map, fn(v, k) { v; });
        hashMapSize(result)
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "Number(0.0)");
}
```

### Pattern 4: Large Collections

```rust
#[test]
fn test_large_collection() {
    let code = r#"
        let map = hashMapNew();
        let i = 0;
        while (i < 100) {
            hashMapPut(map, toString(i), i);
            i = i + 1;
        }
        hashMapSize(map)
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "Number(100.0)");
}
```

---

## Testing Parity (Interpreter vs VM)

**Requirement:** Both engines must produce identical results.

### Pattern 1: Integration Test (Tests Interpreter)

```rust
#[test]
fn test_operation() {
    let code = r#"/* test code */"#;
    let result = run(code).unwrap();
    assert_eq!(result, "expected");
}
```

**Note:** The `run()` helper uses Interpreter by default. VM parity is tested by running the full test suite with VM execution mode.

### Pattern 2: Explicit VM Test (If Needed)

```rust
use atlas_runtime::vm::VM;
use atlas_runtime::bytecode::BytecodeCompiler;

fn run_vm(code: &str) -> Result<String, String> {
    let mut lexer = Lexer::new(code);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (ast, _) = parser.parse();

    let mut compiler = BytecodeCompiler::new();
    let chunk = compiler.compile(&ast).map_err(|e| format!("{:?}", e))?;

    let mut vm = VM::new();
    let security = SecurityContext::allow_all();
    match vm.run(&chunk, &security) {
        Ok(val) => Ok(format!("{:?}", val)),
        Err(e) => Err(format!("{:?}", e)),
    }
}

#[test]
fn test_parity() {
    let code = r#"/* test code */"#;
    let interp_result = run(code).unwrap();
    let vm_result = run_vm(code).unwrap();
    assert_eq!(interp_result, vm_result);
}
```

---

## Parameterized Tests (rstest)

**Use for:** Testing multiple inputs with same logic

```rust
use rstest::rstest;

#[rstest]
#[case(1, 2, 3)]
#[case(10, 20, 30)]
#[case(-5, 5, 0)]
fn test_addition(#[case] a: i64, #[case] b: i64, #[case] expected: i64) {
    let code = format!("let x = {}; let y = {}; x + y", a, b);
    let result = run(&code).unwrap();
    assert!(result.contains(&format!("{}.0", expected)));
}
```

---

## Snapshot Tests (insta)

**Use for:** Testing complex output (AST, bytecode, error messages)

```rust
use insta::assert_snapshot;

#[test]
fn test_error_message() {
    let code = r#"let x = undefined_var;"#;
    let result = run(code).unwrap_err();
    assert_snapshot!(result);
}
```

**Workflow:**
1. Run test first time → creates snapshot
2. Run again → compares to snapshot
3. On mismatch → `cargo insta review` to accept/reject

---

## Property-Based Tests (proptest)

**Use for:** Testing invariants with random inputs

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_hashmap_size_invariant(keys in prop::collection::vec(".*", 0..100)) {
        let mut code = String::from("let map = hashMapNew();\n");
        for (i, key) in keys.iter().enumerate() {
            code.push_str(&format!("hashMapPut(map, \"{}\", {});\n", key, i));
        }
        code.push_str("hashMapSize(map)");

        let result = run(&code).unwrap();
        let expected = keys.len();
        assert!(result.contains(&format!("{}.0", expected)));
    }
}
```

---

## Atlas Language Semantics (Important!)

### Closure Semantics

**Atlas closures capture environment by reference, not value.**

```rust
#[test]
fn test_closure_captures_reference() {
    let code = r#"
        let x = 1;
        let f = fn() { x; };
        x = 2;  // Mutate x
        f()     // Returns 2, not 1!
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "Number(2.0)");
}
```

**Implication:** Callbacks in intrinsics see current environment state.

### Function Return Values

**Last expression in function is the return value (no explicit return needed).**

```rust
#[test]
fn test_implicit_return() {
    let code = r#"
        let f = fn(x) { x * 2; };  // Semicolon required!
        f(5)
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "Number(10.0)");
}
```

### Truthiness

**Truthy:** All values except `false` and `null`
**Falsy:** `false`, `null`

```rust
#[test]
fn test_filter_truthiness() {
    let code = r#"
        let arr = [0, 1, 2, false, null];
        filter(arr, fn(x) { x; })  // 0, 1, 2 are truthy!
    "#;
    let result = run(code).unwrap();
    // 0, 1, 2 pass (numbers are truthy)
}
```

---

## Test Organization

### File Naming

- `{feature}_tests.rs` - Feature integration tests
- `{feature}_integration_tests.rs` - Cross-feature tests
- `diagnostic_*.rs` - Compiler diagnostic tests

### Test Naming

- `test_{feature}_{scenario}` - Descriptive names
- `test_{feature}_error_{case}` - Error cases
- `test_parity_{feature}` - Parity verification

### Test Groups

```rust
// Group related tests with comments
// ========================================
// HashMap Basic Operations
// ========================================

#[test]
fn test_hashmap_new() { /* ... */ }

#[test]
fn test_hashmap_put() { /* ... */ }

// ========================================
// HashMap Iteration
// ========================================

#[test]
fn test_hashmap_foreach() { /* ... */ }
```

---

## Test Execution

### Run All Tests

```bash
cargo test -p atlas-runtime
```

### Run Specific Test

```bash
cargo test -p atlas-runtime test_hashmap_foreach -- --exact
```

### Run With Output

```bash
cargo test -p atlas-runtime test_name -- --nocapture
```

### Run Pattern Match

```bash
cargo test -p atlas-runtime hashmap  # Runs all tests with "hashmap" in name
```

---

## Quality Standards

**Every feature must have:**
1. ✅ Happy path tests (basic operations work)
2. ✅ Edge case tests (empty, large, boundary values)
3. ✅ Error tests (wrong types, invalid arguments)
4. ✅ Integration tests (feature + feature)
5. ✅ Parity verification (interpreter == VM)

**Test coverage guidelines:**
- 10+ tests for new intrinsic
- 15+ tests for new collection type
- 20+ tests for complex feature (pattern matching, etc.)

**Test quality over quantity:**
- Clear test names
- Focused assertions
- Good error messages
- Documented edge cases
