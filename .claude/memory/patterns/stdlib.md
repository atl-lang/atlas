# Stdlib Patterns

## P-001: JsonValue - Controlled Dynamic Typing
**Status:** Active

`JsonValue` type for dynamic JSON. Exception to strict typing (necessary for AI).

## P-002: Array API - Intrinsics vs Stdlib Split
**Status:** Active

- **Pure functions:** `stdlib/array.rs` (push, pop, slice)
- **Callback intrinsics:** Interpreter/VM direct (map, filter, forEach)

## P-003: Hash Function Design
**Status:** Active

Deterministic hashing via `DefaultHasher`.
- Hashable: number, string, bool, null
- Not hashable: array, function, JsonValue → error AT0140

## P-004: Collection Value Representation
**Status:** Active

Collections use `Arc<Mutex<X>>`:
```rust
Value::HashMap(Arc<Mutex<AtlasHashMap>>)
Value::HashSet(Arc<Mutex<AtlasHashSet>>)
```

## P-005: Collection API Design
**Status:** Active

Function-based API: `hashMapPut(map, key, value)` not `map.put()`.
Explicit type names, callback-based iteration.

## P-006: HashMap Stdlib Architecture
**Status:** Active

Use `stdlib/mod.rs` pattern. Register in `is_builtin()`, implement in `call_builtin()`.
