# Atlas Codebase Patterns

**Purpose:** Active patterns for AI reference. Archived stable patterns in `archive/2026-02-patterns-v1.md`.

---

## Runtime API

```rust
let atlas = Atlas::new();
let atlas = Atlas::new_with_security(SecurityContext::allow_all());
let result: RuntimeResult<Value> = atlas.eval("let x = 1;");
```

**Test helpers** (`tests/common/mod.rs`):
```rust
common::assert_eval_number("1 + 2", 3.0);
common::assert_eval_string(r#""hello""#, "hello");
common::assert_error_code("bad code", "AT0001");
common::compile_source("let x = 1;");
common::run_bytecode(bytecode);
```

---

## Collection Types (CoW — Phase 12–15)

All collections use CoW wrappers (Arc, no Mutex):
```rust
Value::Array(ValueArray)          // Arc<Vec<Value>> — .as_slice(), .len(), .push(), .set()
Value::HashMap(ValueHashMap)      // Arc<AtlasHashMap> — .inner(), .inner_mut()
Value::HashSet(ValueHashSet)      // Arc<AtlasHashSet> — .inner(), .inner_mut()
Value::Queue(ValueQueue)          // Arc<VecDeque<Value>>
Value::Stack(ValueStack)          // Arc<Vec<Value>>
```

**NEVER use `.lock().unwrap()`** — Mutex is gone. Use CoW API.

**Mutation builtins return new collection** (or `[extracted, new_col]` for remove/pop/dequeue).
Interpreter and VM automatically write-back to first-arg variable via CoW write-back.

**Array method syntax** (Phase 16):
- `arr.push(x)` → `arrayPush`, mutating (writes back to receiver)
- `arr.pop()` → `arrayPop`, RETURNS_PAIR `[removed, new_arr]`, writes back new_arr, returns removed
- `arr.shift()` → `arrayShift`, same as pop pattern
- `arr.unshift(x)` → `arrayUnshift`, mutating (writes back)
- `arr.reverse()` → `arrayReverse`, mutating (writes back)
- `arr.sort()` → `arraySort`, NON-mutating (returns sorted copy, receiver unchanged)
- TypeTag::Array in method_dispatch.rs; typechecker sets it in check_member
- Dynamic fallback in eval_member when type_tag is None (runtime check on Value::Array)

**HashKey** (not String) for HashMap/HashSet keys:
```rust
HashKey::String(Arc::new("x".to_string()))
HashKey::Number(OrderedFloat(1.0))
```

---

## Stdlib Function Pattern

Register in `stdlib/mod.rs`:
- `is_builtin(name) -> bool`
- `call_builtin(name, args, span, security) -> Result<Value, RuntimeError>`

```rust
fn expect_string(value: &Value, arg_name: &str, span: Span) -> Result<String, RuntimeError> {
    match value {
        Value::String(s) => Ok((**s).clone()),
        _ => Err(RuntimeError::TypeError { msg: "...", span })
    }
}
```

---

## Error Handling

Use struct variants (NOT `::new()`):
```rust
RuntimeError::TypeError { msg: "message".to_string(), span }
```
