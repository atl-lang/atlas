# Phase 07b: HashSet - Set Collection with Operations

## 🚨 DEPENDENCIES - CHECK BEFORE STARTING

**REQUIRED:** Phase 07a (Hash Infrastructure + HashMap) must be complete.

**Verification:**
```bash
# Verify HashMap implementation exists
ls crates/atlas-runtime/src/stdlib/collections/hashmap.rs
ls crates/atlas-runtime/src/stdlib/collections/hash.rs

# Verify HashMap Value variant exists
grep -n "HashMap(Rc<RefCell<AtlasHashMap>>)" crates/atlas-runtime/src/value.rs

# Verify HashMap functions registered
grep -n "hashMapNew" crates/atlas-runtime/src/stdlib/prelude.rs

# Verify HashMap tests pass
cargo test -p atlas-runtime hashmap_tests -- --nocapture
cargo test -p atlas-runtime hash_function_tests -- --nocapture

# Clean build check
cargo clean && cargo check -p atlas-runtime
```

**What's needed:**
- Phase 07a complete (hash infrastructure + HashMap)
- HashKey type available
- Hash function infrastructure
- Value::HashMap variant
- All HashMap tests passing

**If missing:** Complete Phase 07a first - HashSet depends on HashMap

---

## Objective

Implement HashSet collection for storing unique values with efficient membership testing and set operations. Backed by HashMap for O(1) average-case operations. Provide classic set operations: union, intersection, difference, symmetric difference, subset, and superset tests. World-class implementation rivaling Rust HashSet, Python set, and JavaScript Set.

## Files

**Create:** `crates/atlas-runtime/src/stdlib/collections/hashset.rs` (~400 lines)
**Update:** `crates/atlas-runtime/src/value.rs` (~30 lines - HashSet variant)
**Update:** `crates/atlas-runtime/src/stdlib/prelude.rs` (~180 lines - HashSet functions)
**Update:** `docs/api/stdlib.md` (~250 lines - HashSet documentation)
**Tests:** `crates/atlas-runtime/tests/hashset_tests.rs` (~550 lines)

**Total new code:** ~610 lines
**Total tests:** ~550 lines (25+ test cases)

## Dependencies

- HashMap implementation (phase 07a)
- HashKey type (phase 07a)
- Hash function infrastructure (phase 07a)
- Option<T> type (phase 06a)
- Value enum (existing)

## Implementation

### 1. HashSet Implementation (`collections/hashset.rs`)

**AtlasHashSet struct backed by HashMap:**
```rust
use crate::stdlib::collections::hash::HashKey;
use std::collections::HashSet as RustHashSet;

/// Atlas HashSet - unique value collection with O(1) operations
/// Backed by Rust's HashSet for proven performance
pub struct AtlasHashSet {
    inner: RustHashSet<HashKey>,
}

impl AtlasHashSet {
    /// Create new empty HashSet
    pub fn new() -> Self {
        Self {
            inner: RustHashSet::new(),
        }
    }

    /// Create HashSet with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: RustHashSet::with_capacity(capacity),
        }
    }

    /// Add element to set, returns true if inserted (false if already existed)
    pub fn insert(&mut self, element: HashKey) -> bool {
        self.inner.insert(element)
    }

    /// Remove element from set, returns true if existed
    pub fn remove(&mut self, element: &HashKey) -> bool {
        self.inner.remove(element)
    }

    /// Check if element exists in set
    pub fn contains(&self, element: &HashKey) -> bool {
        self.inner.contains(element)
    }

    /// Get number of elements
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Remove all elements
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Convert to vector of elements
    pub fn to_vec(&self) -> Vec<HashKey> {
        self.inner.iter().cloned().collect()
    }

    /// Set union: all elements in either set
    pub fn union(&self, other: &AtlasHashSet) -> AtlasHashSet {
        AtlasHashSet {
            inner: self.inner.union(&other.inner).cloned().collect(),
        }
    }

    /// Set intersection: elements in both sets
    pub fn intersection(&self, other: &AtlasHashSet) -> AtlasHashSet {
        AtlasHashSet {
            inner: self.inner.intersection(&other.inner).cloned().collect(),
        }
    }

    /// Set difference: elements in self but not in other
    pub fn difference(&self, other: &AtlasHashSet) -> AtlasHashSet {
        AtlasHashSet {
            inner: self.inner.difference(&other.inner).cloned().collect(),
        }
    }

    /// Symmetric difference: elements in exactly one set
    pub fn symmetric_difference(&self, other: &AtlasHashSet) -> AtlasHashSet {
        AtlasHashSet {
            inner: self.inner.symmetric_difference(&other.inner).cloned().collect(),
        }
    }

    /// Check if self is subset of other
    pub fn is_subset(&self, other: &AtlasHashSet) -> bool {
        self.inner.is_subset(&other.inner)
    }

    /// Check if self is superset of other
    pub fn is_superset(&self, other: &AtlasHashSet) -> bool {
        self.inner.is_superset(&other.inner)
    }

    /// Check if sets are disjoint (no common elements)
    pub fn is_disjoint(&self, other: &AtlasHashSet) -> bool {
        self.inner.is_disjoint(&other.inner)
    }
}

impl Default for AtlasHashSet {
    fn default() -> Self {
        Self::new()
    }
}
```

### 2. Value Enum Update (`value.rs`)

Add HashSet variant:

```rust
use crate::stdlib::collections::hashset::AtlasHashSet;

pub enum Value {
    // Existing variants...
    HashMap(Rc<RefCell<AtlasHashMap>>),

    // New: HashSet collection
    HashSet(Rc<RefCell<AtlasHashSet>>),
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            // Existing cases...
            Value::HashSet(_) => "hashset",
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Existing cases...
            (Value::HashSet(_), Value::HashSet(_)) => false, // Reference type, never equal
            _ => false,
        }
    }
}
```

### 3. Stdlib Functions (`stdlib/prelude.rs`)

Implement 16 HashSet functions:

**Creation:**
```rust
fn hashset_new(_args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    Ok(Value::HashSet(Rc::new(RefCell::new(AtlasHashSet::new()))))
}

fn hashset_from_array(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    // args[0]: Array of hashable elements
    let array = expect_array(&args[0], "hashSetFromArray")?;
    let mut set = AtlasHashSet::new();

    for element in array.borrow().iter() {
        let key = HashKey::from_value(element)?;
        set.insert(key);
    }

    Ok(Value::HashSet(Rc::new(RefCell::new(set))))
}
```

**Mutation:**
```rust
fn hashset_add(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    // args[0]: HashSet, args[1]: element (hashable)
    let set = expect_hashset(&args[0], "hashSetAdd")?;
    let element = HashKey::from_value(&args[1])?;

    set.borrow_mut().insert(element);
    Ok(Value::Null)
}

fn hashset_remove(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    // args[0]: HashSet, args[1]: element
    let set = expect_hashset(&args[0], "hashSetRemove")?;
    let element = HashKey::from_value(&args[1])?;

    let existed = set.borrow_mut().remove(&element);
    Ok(Value::Bool(existed))
}

fn hashset_clear(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    let set = expect_hashset(&args[0], "hashSetClear")?;
    set.borrow_mut().clear();
    Ok(Value::Null)
}
```

**Query:**
```rust
fn hashset_has(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    let set = expect_hashset(&args[0], "hashSetHas")?;
    let element = HashKey::from_value(&args[1])?;

    Ok(Value::Bool(set.borrow().contains(&element)))
}

fn hashset_size(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    let set = expect_hashset(&args[0], "hashSetSize")?;
    Ok(Value::Number(set.borrow().len() as f64))
}

fn hashset_is_empty(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    let set = expect_hashset(&args[0], "hashSetIsEmpty")?;
    Ok(Value::Bool(set.borrow().is_empty()))
}
```

**Set Operations:**
```rust
fn hashset_union(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    // args[0]: HashSet A, args[1]: HashSet B
    let set_a = expect_hashset(&args[0], "hashSetUnion")?;
    let set_b = expect_hashset(&args[1], "hashSetUnion")?;

    let result = set_a.borrow().union(&set_b.borrow());
    Ok(Value::HashSet(Rc::new(RefCell::new(result))))
}

fn hashset_intersection(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    let set_a = expect_hashset(&args[0], "hashSetIntersection")?;
    let set_b = expect_hashset(&args[1], "hashSetIntersection")?;

    let result = set_a.borrow().intersection(&set_b.borrow());
    Ok(Value::HashSet(Rc::new(RefCell::new(result))))
}

fn hashset_difference(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    let set_a = expect_hashset(&args[0], "hashSetDifference")?;
    let set_b = expect_hashset(&args[1], "hashSetDifference")?;

    let result = set_a.borrow().difference(&set_b.borrow());
    Ok(Value::HashSet(Rc::new(RefCell::new(result))))
}

fn hashset_symmetric_difference(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    let set_a = expect_hashset(&args[0], "hashSetSymmetricDifference")?;
    let set_b = expect_hashset(&args[1], "hashSetSymmetricDifference")?;

    let result = set_a.borrow().symmetric_difference(&set_b.borrow());
    Ok(Value::HashSet(Rc::new(RefCell::new(result))))
}

fn hashset_is_subset(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    let set_a = expect_hashset(&args[0], "hashSetIsSubset")?;
    let set_b = expect_hashset(&args[1], "hashSetIsSubset")?;

    Ok(Value::Bool(set_a.borrow().is_subset(&set_b.borrow())))
}

fn hashset_is_superset(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    let set_a = expect_hashset(&args[0], "hashSetIsSuperset")?;
    let set_b = expect_hashset(&args[1], "hashSetIsSuperset")?;

    Ok(Value::Bool(set_a.borrow().is_superset(&set_b.borrow())))
}
```

**Access:**
```rust
fn hashset_to_array(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    let set = expect_hashset(&args[0], "hashSetToArray")?;
    let elements: Vec<Value> = set.borrow().to_vec()
        .into_iter()
        .map(|key| key.to_value())
        .collect();
    Ok(Value::Array(Rc::new(RefCell::new(elements))))
}
```

**Helper:**
```rust
fn expect_hashset(value: &Value, func_name: &str) -> Result<Rc<RefCell<AtlasHashSet>>, RuntimeError> {
    match value {
        Value::HashSet(set) => Ok(Rc::clone(set)),
        _ => Err(RuntimeError::new(
            ErrorCode::AT0102,
            format!("{} expects HashSet, got {}", func_name, value.type_name())
        ))
    }
}
```

**Register functions:**
```rust
// In create_prelude()
register_native!(prelude, "hashSetNew", hashset_new, 0);
register_native!(prelude, "hashSetFromArray", hashset_from_array, 1);
register_native!(prelude, "hashSetAdd", hashset_add, 2);
register_native!(prelude, "hashSetRemove", hashset_remove, 2);
register_native!(prelude, "hashSetHas", hashset_has, 2);
register_native!(prelude, "hashSetSize", hashset_size, 1);
register_native!(prelude, "hashSetIsEmpty", hashset_is_empty, 1);
register_native!(prelude, "hashSetClear", hashset_clear, 1);
register_native!(prelude, "hashSetUnion", hashset_union, 2);
register_native!(prelude, "hashSetIntersection", hashset_intersection, 2);
register_native!(prelude, "hashSetDifference", hashset_difference, 2);
register_native!(prelude, "hashSetSymmetricDifference", hashset_symmetric_difference, 2);
register_native!(prelude, "hashSetIsSubset", hashset_is_subset, 2);
register_native!(prelude, "hashSetIsSuperset", hashset_is_superset, 2);
register_native!(prelude, "hashSetToArray", hashset_to_array, 1);
```

### 4. Documentation (`docs/api/stdlib.md`)

Add HashSet section:

```markdown
### HashSet Functions

HashSet provides unique value storage with O(1) membership testing. Elements must be hashable types (number, string, bool, null). Supports classic set operations.

#### hashSetNew
**Signature:** `hashSetNew() -> HashSet`
**Behavior:** Creates new empty HashSet
**Example:** `let set = hashSetNew()`

#### hashSetFromArray
**Signature:** `hashSetFromArray(elements: hashable[]) -> HashSet`
**Behavior:** Creates HashSet from array (duplicates removed)
**Example:** `let set = hashSetFromArray([1, 2, 2, 3])` → set has {1, 2, 3}

#### hashSetAdd
**Signature:** `hashSetAdd(set: HashSet, element: hashable) -> void`
**Behavior:** Adds element to set (idempotent)
**Example:** `hashSetAdd(set, "apple")`

[... continue for all 16 functions with set operation examples ...]

**Set Operations Examples:**
```atlas
let a = hashSetFromArray([1, 2, 3])
let b = hashSetFromArray([2, 3, 4])

let union = hashSetUnion(a, b)              // {1, 2, 3, 4}
let intersection = hashSetIntersection(a, b) // {2, 3}
let difference = hashSetDifference(a, b)     // {1}
let symDiff = hashSetSymmetricDifference(a, b) // {1, 4}

hashSetIsSubset(a, b)   // false
hashSetIsSuperset(a, b) // false
```
```

## Tests (TDD - Use rstest)

### HashSet Tests (`tests/hashset_tests.rs`) - 25+ tests

**Creation:**
1. Create empty HashSet with hashSetNew()
2. Create HashSet from array with hashSetFromArray()
3. FromArray removes duplicates
4. FromArray with unhashable elements returns AT0140
5. Empty HashSet has size 0 and isEmpty true

**Add and Remove:**
6. Add element increases size
7. Add duplicate element doesn't increase size (idempotent)
8. Add different hashable types (number, string, bool, null)
9. Remove existing element returns true and decreases size
10. Remove nonexistent element returns false
11. Add after remove works correctly
12. Add with unhashable element returns AT0140

**Has:**
13. Has returns true for existing element
14. Has returns false for nonexistent element
15. Has works with all hashable types

**Size and IsEmpty:**
16. Size reflects current element count
17. IsEmpty true for new HashSet
18. IsEmpty false after adding elements
19. IsEmpty true after clear

**Set Operations - Union:**
20. Union of disjoint sets contains all elements
21. Union of overlapping sets contains unique elements
22. Union with empty set returns copy of original
23. Union is commutative (a∪b = b∪a)

**Set Operations - Intersection:**
24. Intersection of overlapping sets contains common elements
25. Intersection of disjoint sets is empty
26. Intersection with empty set is empty
27. Intersection is commutative (a∩b = b∩a)

**Set Operations - Difference:**
28. Difference removes elements in second set
29. Difference of disjoint sets returns first set
30. Difference with empty set returns first set
31. Difference is NOT commutative (a-b ≠ b-a)

**Set Operations - Symmetric Difference:**
32. Symmetric difference contains elements in exactly one set
33. Symmetric difference of identical sets is empty
34. Symmetric difference is commutative

**Subset and Superset:**
35. Empty set is subset of any set
36. Set is subset of itself
37. Proper subset test works correctly
38. Empty set is not superset of non-empty set
39. Set is superset of itself
40. Superset is inverse of subset

**Integration:**
41. Multiple HashSets are independent
42. HashSet reference semantics (assignment shares reference)
43. ToArray preserves all elements (order may vary)
44. Large HashSet (1000+ elements) performance test
45. HashSet with mixed types (number, string, bool, null)

**Minimum test count:** 25 tests (45 total above)

**Parity requirement:** All tests run in both interpreter and VM with identical results.

## Integration Points

- Uses: HashMap infrastructure (HashKey, hash function)
- Uses: Rust HashSet for backing implementation
- Uses: Value enum for storage
- Creates: HashSet collection type
- Updates: Value enum with HashSet variant
- Output: 16 HashSet functions in stdlib

## Acceptance

- ✅ HashSet variant added to Value enum
- ✅ AtlasHashSet struct implemented
- ✅ All 16 HashSet functions implemented and registered
- ✅ hashSetNew, hashSetFromArray work correctly
- ✅ hashSetAdd, hashSetRemove, hashSetHas work correctly
- ✅ hashSetSize, hashSetIsEmpty, hashSetClear work correctly
- ✅ hashSetToArray works correctly
- ✅ Set operations implemented: union, intersection, difference, symmetric difference
- ✅ Subset and superset tests work correctly
- ✅ Set operations follow mathematical set theory
- ✅ 25+ tests pass (all set operations verified)
- ✅ 100% interpreter/VM parity verified
- ✅ Documentation complete in docs/api/stdlib.md
- ✅ No clippy warnings
- ✅ cargo test -p atlas-runtime passes
- ✅ Decision logs DR-003, DR-004, DR-005 referenced

## References

**Decision Logs:**
- DR-003: Hash function design
- DR-004: Collection value representation
- DR-005: Collection API design

**Specifications:**
- `docs/specification/runtime.md` - Value representation
- `docs/api/stdlib.md` - HashSet API

**Implementation:**
- Rust HashSet: https://doc.rust-lang.org/std/collections/struct.HashSet.html
- Set theory: https://en.wikipedia.org/wiki/Set_(mathematics)

**Previous Phase:** `phase-07a-hash-infrastructure-hashmap.md` (dependency)
**Next Phase:** `phase-07c-queue-stack.md` (independent - can proceed)
