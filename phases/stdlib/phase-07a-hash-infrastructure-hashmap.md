# Phase 07a: Hash Infrastructure + HashMap

## 🚨 DEPENDENCIES - CHECK BEFORE STARTING

**REQUIRED:** Stdlib infrastructure and generic types must exist.

**Verification:**
```bash
# Verify stdlib infrastructure
ls crates/atlas-runtime/src/stdlib/mod.rs
ls crates/atlas-runtime/src/stdlib/prelude.rs

# Verify generic type support (Option, Result)
grep -n "enum Value" crates/atlas-runtime/src/value.rs
grep -n "Option<" crates/atlas-runtime/src/value.rs
grep -n "Result<" crates/atlas-runtime/src/value.rs

# Verify existing tests pass
cargo clean && cargo check -p atlas-runtime
cargo test -p atlas-runtime stdlib -- --nocapture

# Verify previous stdlib phases complete
ls crates/atlas-runtime/src/stdlib/string.rs
ls crates/atlas-runtime/src/stdlib/array.rs
```

**What's needed:**
- Stdlib module system from v0.1
- Generic Option<T> and Result<T, E> types (Phase-06a complete)
- Value enum (existing)
- Test infrastructure (rstest, insta)

**If missing:** Previous stdlib phases should be complete - this builds on foundation

---

## Objective

Implement hash function infrastructure for Atlas values and a production-quality HashMap collection. Provide deterministic hashing for number, string, bool, and null types. Implement hash table with separate chaining, automatic resizing, and O(1) average-case operations. This phase establishes the foundation for HashSet (phase 07b) and provides a world-class key-value data structure rivaling Rust, Go, and TypeScript.

## Files

**Create:** `crates/atlas-runtime/src/stdlib/collections/mod.rs` (~150 lines)
**Create:** `crates/atlas-runtime/src/stdlib/collections/hash.rs` (~200 lines)
**Create:** `crates/atlas-runtime/src/stdlib/collections/hashmap.rs` (~450 lines)
**Update:** `crates/atlas-runtime/src/value.rs` (~100 lines - HashMap variant + HashKey)
**Update:** `crates/atlas-runtime/src/stdlib/mod.rs` (~20 lines - collections module)
**Update:** `crates/atlas-runtime/src/stdlib/prelude.rs` (~150 lines - HashMap functions)
**Update:** `docs/api/stdlib.md` (~200 lines - HashMap documentation)
**Update:** `docs/specification/diagnostic-system.md` (~20 lines - new error codes)
**Tests:** `crates/atlas-runtime/tests/hashmap_tests.rs` (~600 lines)
**Tests:** `crates/atlas-runtime/tests/hash_function_tests.rs` (~200 lines)

**Total new code:** ~1,270 lines
**Total tests:** ~800 lines (35+ test cases)

## Dependencies

- Value enum (existing in value.rs)
- Option<T> type (phase-06a complete)
- Stdlib infrastructure (prelude, module system)
- Rust std::hash::Hash trait
- Rust std::collections::HashMap (implementation reference)

## Implementation

### 1. Hash Function Infrastructure (`collections/hash.rs`)

Implement deterministic hash function for Atlas values:

**HashKey enum for hashable values:**
```rust
use ordered_float::OrderedFloat;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

/// Wrapper type for hashable Atlas values
/// Only Number, String, Bool, Null can be hashed
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HashKey {
    Number(OrderedFloat<f64>),  // OrderedFloat handles NaN canonicalization
    String(Rc<String>),
    Bool(bool),
    Null,
}

impl HashKey {
    /// Create HashKey from Value, returns error if not hashable
    pub fn from_value(value: &Value) -> Result<Self, RuntimeError> {
        match value {
            Value::Number(n) => {
                // Canonicalize NaN to ensure consistent hashing
                let normalized = if n.is_nan() { f64::NAN } else { *n };
                Ok(HashKey::Number(OrderedFloat(normalized)))
            }
            Value::String(s) => Ok(HashKey::String(Rc::clone(s))),
            Value::Bool(b) => Ok(HashKey::Bool(*b)),
            Value::Null => Ok(HashKey::Null),
            _ => Err(RuntimeError::new(
                ErrorCode::AT0140,
                format!("Cannot hash type {} - only number, string, bool, null are hashable", value.type_name())
            ))
        }
    }

    /// Convert HashKey back to Value
    pub fn to_value(&self) -> Value {
        match self {
            HashKey::Number(n) => Value::Number(n.0),
            HashKey::String(s) => Value::String(Rc::clone(s)),
            HashKey::Bool(b) => Value::Bool(*b),
            HashKey::Null => Value::Null,
        }
    }
}
```

**Hash computation using Rust DefaultHasher:**
```rust
use std::collections::hash_map::DefaultHasher;

pub fn compute_hash(key: &HashKey) -> u64 {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
}
```

**Why this design:**
- **Deterministic:** No RandomState - same input always produces same hash
- **AI-friendly:** Reproducible for testing and debugging
- **OrderedFloat:** Handles NaN canonicalization (all NaN values hash to same value)
- **Type-safe:** Only hashable types can create HashKey
- **Rust patterns:** Uses std::hash infrastructure (proven, optimized)

### 2. Value Enum Updates (`value.rs`)

Add HashMap variant to Value enum:

```rust
use crate::stdlib::collections::hashmap::AtlasHashMap;
use std::cell::RefCell;
use std::rc::Rc;

pub enum Value {
    // Existing variants
    Number(f64),
    String(Rc<String>),
    Bool(bool),
    Null,
    Array(Rc<RefCell<Vec<Value>>>),
    Function(FunctionRef),
    NativeFunction(NativeFn),
    JsonValue(Rc<JsonValue>),
    Option(Option<Box<Value>>),
    Result(Result<Box<Value>, Box<Value>>),

    // New: HashMap collection
    HashMap(Rc<RefCell<AtlasHashMap>>),
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            // Existing cases...
            Value::HashMap(_) => "hashmap",
        }
    }
}
```

**Update PartialEq for HashMap:**
```rust
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Existing cases...
            (Value::HashMap(_), Value::HashMap(_)) => false, // HashMap never equal (reference type)
            _ => false,
        }
    }
}
```

### 3. HashMap Implementation (`collections/hashmap.rs`)

**AtlasHashMap struct:**
```rust
use crate::stdlib::collections::hash::HashKey;
use crate::value::Value;
use std::collections::HashMap;

/// Atlas HashMap - key-value collection with O(1) average operations
pub struct AtlasHashMap {
    inner: HashMap<HashKey, Value>,
}

impl AtlasHashMap {
    /// Create new empty HashMap with default capacity
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Create HashMap with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: HashMap::with_capacity(capacity),
        }
    }

    /// Insert key-value pair, returns previous value if existed
    pub fn insert(&mut self, key: HashKey, value: Value) -> Option<Value> {
        self.inner.insert(key, value)
    }

    /// Get value by key (returns None if not found)
    pub fn get(&self, key: &HashKey) -> Option<&Value> {
        self.inner.get(key)
    }

    /// Remove key-value pair, returns value if existed
    pub fn remove(&mut self, key: &HashKey) -> Option<Value> {
        self.inner.remove(key)
    }

    /// Check if key exists
    pub fn contains_key(&self, key: &HashKey) -> bool {
        self.inner.contains_key(key)
    }

    /// Get number of entries
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Remove all entries
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Get all keys as vector
    pub fn keys(&self) -> Vec<HashKey> {
        self.inner.keys().cloned().collect()
    }

    /// Get all values as vector
    pub fn values(&self) -> Vec<Value> {
        self.inner.values().cloned().collect()
    }

    /// Get all entries as vector of (key, value) pairs
    pub fn entries(&self) -> Vec<(HashKey, Value)> {
        self.inner.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }
}

impl Default for AtlasHashMap {
    fn default() -> Self {
        Self::new()
    }
}
```

### 4. Stdlib Functions (`stdlib/prelude.rs`)

Implement 15 HashMap functions:

**Creation functions:**
```rust
fn hashmap_new(_args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    Ok(Value::HashMap(Rc::new(RefCell::new(AtlasHashMap::new()))))
}

fn hashmap_from_entries(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    // args[0]: Array of [key, value] pairs
    let entries_array = expect_array(&args[0], "hashMapFromEntries")?;
    let map = AtlasHashMap::new();

    for entry in entries_array.borrow().iter() {
        let pair = expect_array(entry, "entry")?;
        let pair_borrow = pair.borrow();
        if pair_borrow.len() != 2 {
            return Err(RuntimeError::new(
                ErrorCode::AT0103,
                "Entry must be [key, value] array with exactly 2 elements"
            ));
        }
        let key = HashKey::from_value(&pair_borrow[0])?;
        let value = pair_borrow[1].clone();
        map.borrow_mut().insert(key, value);
    }

    Ok(Value::HashMap(Rc::new(RefCell::new(map))))
}
```

**Mutation functions:**
```rust
fn hashmap_put(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    // args[0]: HashMap, args[1]: key (hashable), args[2]: value
    let map = expect_hashmap(&args[0], "hashMapPut")?;
    let key = HashKey::from_value(&args[1])?;
    let value = args[2].clone();

    map.borrow_mut().insert(key, value);
    Ok(Value::Null)
}

fn hashmap_remove(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    // args[0]: HashMap, args[1]: key
    let map = expect_hashmap(&args[0], "hashMapRemove")?;
    let key = HashKey::from_value(&args[1])?;

    let removed = map.borrow_mut().remove(&key);
    Ok(match removed {
        Some(v) => Value::Option(Some(Box::new(v))),
        None => Value::Option(None),
    })
}

fn hashmap_clear(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    let map = expect_hashmap(&args[0], "hashMapClear")?;
    map.borrow_mut().clear();
    Ok(Value::Null)
}
```

**Query functions:**
```rust
fn hashmap_get(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    let map = expect_hashmap(&args[0], "hashMapGet")?;
    let key = HashKey::from_value(&args[1])?;

    let value = map.borrow().get(&key).cloned();
    Ok(match value {
        Some(v) => Value::Option(Some(Box::new(v))),
        None => Value::Option(None),
    })
}

fn hashmap_has(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    let map = expect_hashmap(&args[0], "hashMapHas")?;
    let key = HashKey::from_value(&args[1])?;

    Ok(Value::Bool(map.borrow().contains_key(&key)))
}

fn hashmap_size(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    let map = expect_hashmap(&args[0], "hashMapSize")?;
    Ok(Value::Number(map.borrow().len() as f64))
}

fn hashmap_is_empty(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    let map = expect_hashmap(&args[0], "hashMapIsEmpty")?;
    Ok(Value::Bool(map.borrow().is_empty()))
}
```

**Access functions:**
```rust
fn hashmap_keys(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    let map = expect_hashmap(&args[0], "hashMapKeys")?;
    let keys = map.borrow().keys()
        .into_iter()
        .map(|k| k.to_value())
        .collect();
    Ok(Value::Array(Rc::new(RefCell::new(keys))))
}

fn hashmap_values(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    let map = expect_hashmap(&args[0], "hashMapValues")?;
    let values = map.borrow().values();
    Ok(Value::Array(Rc::new(RefCell::new(values))))
}

fn hashmap_entries(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    let map = expect_hashmap(&args[0], "hashMapEntries")?;
    let entries: Vec<Value> = map.borrow().entries()
        .into_iter()
        .map(|(k, v)| {
            let pair = vec![k.to_value(), v];
            Value::Array(Rc::new(RefCell::new(pair)))
        })
        .collect();
    Ok(Value::Array(Rc::new(RefCell::new(entries))))
}
```

**Helper function:**
```rust
fn expect_hashmap(value: &Value, func_name: &str) -> Result<Rc<RefCell<AtlasHashMap>>, RuntimeError> {
    match value {
        Value::HashMap(map) => Ok(Rc::clone(map)),
        _ => Err(RuntimeError::new(
            ErrorCode::AT0102,
            format!("{} expects HashMap, got {}", func_name, value.type_name())
        ))
    }
}
```

**Register functions in prelude:**
```rust
pub fn create_prelude() -> HashMap<String, Value> {
    let mut prelude = HashMap::new();

    // Existing functions...

    // HashMap functions
    register_native!(prelude, "hashMapNew", hashmap_new, 0);
    register_native!(prelude, "hashMapFromEntries", hashmap_from_entries, 1);
    register_native!(prelude, "hashMapPut", hashmap_put, 3);
    register_native!(prelude, "hashMapGet", hashmap_get, 2);
    register_native!(prelude, "hashMapRemove", hashmap_remove, 2);
    register_native!(prelude, "hashMapHas", hashmap_has, 2);
    register_native!(prelude, "hashMapSize", hashmap_size, 1);
    register_native!(prelude, "hashMapIsEmpty", hashmap_is_empty, 1);
    register_native!(prelude, "hashMapClear", hashmap_clear, 1);
    register_native!(prelude, "hashMapKeys", hashmap_keys, 1);
    register_native!(prelude, "hashMapValues", hashmap_values, 1);
    register_native!(prelude, "hashMapEntries", hashmap_entries, 1);

    prelude
}
```

### 5. Error Codes (`docs/specification/diagnostic-system.md`)

Add new error codes:

```markdown
#### AT0140: Cannot Hash Type
**Category:** Runtime Error
**Message:** "Cannot hash type {type} - only number, string, bool, null are hashable"
**Cause:** Attempted to use unhashable type as HashMap key
**Example:** `hashMapPut(map, [1, 2], "value")` - arrays not hashable

#### AT0141: HashMap Key Type Mismatch
**Category:** Runtime Error
**Message:** "HashMap key must be hashable type"
**Cause:** Invalid key type passed to HashMap operation
```

### 6. Documentation (`docs/api/stdlib.md`)

Add Collections section with HashMap API:

```markdown
## Collections

### HashMap Functions

HashMap provides key-value storage with O(1) average-case operations. Keys must be hashable types (number, string, bool, null). Uses deterministic hashing for reproducible behavior.

#### hashMapNew
**Signature:** `hashMapNew() -> HashMap`
**Behavior:** Creates new empty HashMap
**Example:** `let map = hashMapNew()`
**Errors:** None

#### hashMapFromEntries
**Signature:** `hashMapFromEntries(entries: [hashable, any][]) -> HashMap`
**Behavior:** Creates HashMap from array of [key, value] pairs
**Example:** `let map = hashMapFromEntries([["a", 1], ["b", 2]])`
**Errors:** AT0140 if keys not hashable, AT0103 if entries malformed

#### hashMapPut
**Signature:** `hashMapPut(map: HashMap, key: hashable, value: any) -> void`
**Behavior:** Inserts or updates key-value pair
**Example:** `hashMapPut(map, "name", "Alice")`
**Errors:** AT0102 if not HashMap, AT0140 if key not hashable

[... continue for all 15 functions ...]
```

## Tests (TDD - Use rstest)

### Hash Function Tests (`tests/hash_function_tests.rs`) - 15 tests

**Basic hashing:**
1. Hash number values (integers, floats, zero, negatives)
2. Hash string values (empty, ASCII, Unicode, emojis)
3. Hash bool values (true, false)
4. Hash null value
5. NaN canonicalization (all NaN values hash to same value)
6. Positive zero and negative zero hash differently (IEEE 754 compliance)
7. HashKey equality (same values equal, different values not equal)

**Error cases:**
8. Cannot hash array - returns AT0140
9. Cannot hash function - returns AT0140
10. Cannot hash JsonValue - returns AT0140
11. Cannot hash Option - returns AT0140
12. Cannot hash Result - returns AT0140

**Hash quality:**
13. Different strings hash differently (collision avoidance)
14. Similar numbers hash differently (1.0, 1.1, 1.01)
15. Deterministic (same value always produces same hash)

### HashMap Tests (`tests/hashmap_tests.rs`) - 30 tests

**Creation:**
1. Create empty HashMap with hashMapNew()
2. Create HashMap from entries array
3. Empty HashMap has size 0 and isEmpty true
4. FromEntries with duplicate keys (last wins)
5. FromEntries with malformed entries (error)

**Put and Get:**
6. Put number key, get returns Some(value)
7. Put string key, get returns Some(value)
8. Put bool key, get returns Some(value)
9. Put null key, get returns Some(value)
10. Get nonexistent key returns None (Option)
11. Put updates existing key
12. Put with unhashable key returns AT0140

**Remove:**
13. Remove existing key returns Some(value)
14. Remove nonexistent key returns None
15. Remove and verify size decreases
16. Remove all entries leaves empty HashMap

**Has:**
17. Has returns true for existing key
18. Has returns false for nonexistent key
19. Has works with all hashable types

**Size and IsEmpty:**
20. Size increases with put
21. Size decreases with remove
22. IsEmpty true for new HashMap
23. IsEmpty false after put
24. IsEmpty true after clear

**Keys, Values, Entries:**
25. Keys returns array of all keys
26. Values returns array of all values
27. Entries returns array of [key, value] pairs
28. Keys/values/entries work on empty HashMap (return empty arrays)

**Clear:**
29. Clear removes all entries
30. Clear on empty HashMap is safe

**Integration:**
31. HashMap with mixed key types (number, string, bool, null)
32. HashMap with complex values (arrays, objects, functions)
33. Multiple HashMaps independent (no shared state)
34. HashMap reference semantics (assignment shares reference)
35. Large HashMap (1000+ entries) performance test

**Minimum test count:** 35 tests

**Parity requirement:** Every test runs in both interpreter and VM with identical results.

## Integration Points

- Uses: Stdlib infrastructure (prelude, native functions)
- Uses: Option<T> for optional returns (get, remove)
- Uses: Value enum for storage
- Uses: Rust std::hash::Hash for hashing
- Creates: Hash function infrastructure (foundation for HashSet)
- Creates: HashMap collection type
- Updates: Value enum with HashMap variant
- Output: 15 HashMap functions in stdlib

## Acceptance

- ✅ Hash function infrastructure implemented (hash.rs, HashKey enum)
- ✅ HashMap variant added to Value enum
- ✅ AtlasHashMap struct with Rust HashMap backing
- ✅ All 15 HashMap functions implemented and registered
- ✅ hashMapNew, hashMapFromEntries work correctly
- ✅ hashMapPut, hashMapGet, hashMapRemove work correctly
- ✅ hashMapHas, hashMapSize, hashMapIsEmpty work correctly
- ✅ hashMapKeys, hashMapValues, hashMapEntries work correctly
- ✅ hashMapClear works correctly
- ✅ Error codes AT0140, AT0141 implemented
- ✅ Hash function deterministic (same input → same output)
- ✅ NaN canonicalization works
- ✅ Unhashable types properly rejected
- ✅ 35+ tests pass (15 hash tests + 30 HashMap tests)
- ✅ 100% interpreter/VM parity verified
- ✅ Documentation complete in docs/api/stdlib.md
- ✅ No clippy warnings
- ✅ cargo test -p atlas-runtime passes
- ✅ Decision logs DR-003, DR-004, DR-005 exist and referenced

## References

**Decision Logs:**
- DR-003: Hash function design
- DR-004: Collection value representation
- DR-005: Collection API design

**Specifications:**
- `docs/specification/runtime.md` - Value representation
- `docs/specification/diagnostic-system.md` - Error codes

**Implementation:**
- Rust HashMap: https://doc.rust-lang.org/std/collections/struct.HashMap.html
- OrderedFloat: https://docs.rs/ordered-float/latest/ordered_float/

**Next Phase:** `phase-07b-hashset.md` (depends on HashMap)
