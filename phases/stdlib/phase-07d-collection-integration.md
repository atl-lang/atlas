# Phase 07d: Collection Integration + Iteration

## 🚨 DEPENDENCIES - CHECK BEFORE STARTING

**REQUIRED:** All previous collection phases must be complete.

**Verification:**
```bash
# Verify all collection implementations exist
ls crates/atlas-runtime/src/stdlib/collections/hashmap.rs
ls crates/atlas-runtime/src/stdlib/collections/hashset.rs
ls crates/atlas-runtime/src/stdlib/collections/queue.rs
ls crates/atlas-runtime/src/stdlib/collections/stack.rs
ls crates/atlas-runtime/src/stdlib/collections/hash.rs

# Verify Value variants exist
grep -n "HashMap(Rc<RefCell<AtlasHashMap>>)" crates/atlas-runtime/src/value.rs
grep -n "HashSet(Rc<RefCell<AtlasHashSet>>)" crates/atlas-runtime/src/value.rs
grep -n "Queue(Rc<RefCell<AtlasQueue>>)" crates/atlas-runtime/src/value.rs
grep -n "Stack(Rc<RefCell<AtlasStack>>)" crates/atlas-runtime/src/value.rs

# Verify all collection tests pass
cargo test -p atlas-runtime hashmap_tests -- --nocapture
cargo test -p atlas-runtime hashset_tests -- --nocapture
cargo test -p atlas-runtime queue_tests -- --nocapture
cargo test -p atlas-runtime stack_tests -- --nocapture

# Clean build check
cargo clean && cargo check -p atlas-runtime
```

**What's needed:**
- Phase 07a complete (HashMap + hash infrastructure)
- Phase 07b complete (HashSet)
- Phase 07c complete (Queue + Stack)
- All individual collection tests passing
- Interpreter/VM parity verified for each collection

**If missing:** Complete phases 07a, 07b, 07c first

---

## Objective

Add iteration support for collections using callback-based pattern (like array intrinsics). Implement forEach, map, and filter for HashMap and HashSet as VM/interpreter intrinsics. Create comprehensive cross-collection integration tests. Add performance benchmarks for all collections. Complete documentation with real-world usage examples. Ensure world-class quality and 100% parity across all collection operations.

## Files

**Update:** `crates/atlas-runtime/src/interpreter.rs` (~200 lines - collection iteration intrinsics)
**Update:** `crates/atlas-runtime/src/vm.rs` (~200 lines - collection iteration intrinsics)
**Update:** `crates/atlas-runtime/src/stdlib/prelude.rs` (~50 lines - register iteration functions)
**Update:** `docs/api/stdlib.md` (~200 lines - iteration documentation + examples)
**Tests:** `crates/atlas-runtime/tests/collection_integration_tests.rs` (~600 lines)
**Tests:** `crates/atlas-runtime/tests/collection_iteration_tests.rs` (~400 lines)
**Create:** `crates/atlas-runtime/benches/collection_benchmarks.rs` (~300 lines)

**Total new code:** ~450 lines
**Total tests:** ~1,000 lines (30+ test cases)
**Total benchmarks:** ~300 lines

## Dependencies

- All collection types (HashMap, HashSet, Queue, Stack)
- Array intrinsics pattern (map, filter, forEach already implemented)
- Interpreter and VM execution contexts
- Criterion for benchmarks (already exists from phase 06c)

## Implementation

### 1. HashMap Iteration Intrinsics

**Interpreter implementation (`interpreter.rs`):**
```rust
// In builtin function handlers

fn hashmap_foreach_intrinsic(
    map: &Value,
    callback: &Value,
    interpreter: &mut Interpreter,
) -> Result<Value, RuntimeError> {
    let map_ref = expect_hashmap(map)?;
    let func_ref = expect_function(callback)?;

    // Iterate over entries
    for (key, value) in map_ref.borrow().entries() {
        let args = vec![value, key.to_value()];
        interpreter.call_function(&func_ref, args)?;
    }

    Ok(Value::Null)
}

fn hashmap_map_intrinsic(
    map: &Value,
    callback: &Value,
    interpreter: &mut Interpreter,
) -> Result<Value, RuntimeError> {
    let map_ref = expect_hashmap(map)?;
    let func_ref = expect_function(callback)?;

    let mut new_map = AtlasHashMap::new();

    for (key, value) in map_ref.borrow().entries() {
        let args = vec![value.clone(), key.to_value()];
        let new_value = interpreter.call_function(&func_ref, args)?;
        new_map.insert(key.clone(), new_value);
    }

    Ok(Value::HashMap(Rc::new(RefCell::new(new_map))))
}

fn hashmap_filter_intrinsic(
    map: &Value,
    callback: &Value,
    interpreter: &mut Interpreter,
) -> Result<Value, RuntimeError> {
    let map_ref = expect_hashmap(map)?;
    let func_ref = expect_function(callback)?;

    let mut new_map = AtlasHashMap::new();

    for (key, value) in map_ref.borrow().entries() {
        let args = vec![value.clone(), key.to_value()];
        let result = interpreter.call_function(&func_ref, args)?;

        if let Value::Bool(true) = result {
            new_map.insert(key.clone(), value);
        }
    }

    Ok(Value::HashMap(Rc::new(RefCell::new(new_map))))
}
```

**VM implementation (`vm.rs`):**
```rust
// Add new opcodes for collection iteration
pub enum OpCode {
    // Existing opcodes...
    HashMapForEach,
    HashMapMap,
    HashMapFilter,
    HashSetForEach,
    HashSetFilter,
}

// In VM execute loop
OpCode::HashMapForEach => {
    let callback = self.pop()?;
    let map = self.pop()?;

    let map_ref = expect_hashmap(&map)?;
    let func_ref = expect_function(&callback)?;

    for (key, value) in map_ref.borrow().entries() {
        let args = vec![value, key.to_value()];
        self.call_function(&func_ref, args)?;
    }

    self.push(Value::Null);
}

OpCode::HashMapMap => {
    let callback = self.pop()?;
    let map = self.pop()?;

    let map_ref = expect_hashmap(&map)?;
    let func_ref = expect_function(&callback)?;

    let mut new_map = AtlasHashMap::new();

    for (key, value) in map_ref.borrow().entries() {
        let args = vec![value.clone(), key.to_value()];
        let new_value = self.call_function(&func_ref, args)?;
        new_map.insert(key.clone(), new_value);
    }

    self.push(Value::HashMap(Rc::new(RefCell::new(new_map))));
}

OpCode::HashMapFilter => {
    let callback = self.pop()?;
    let map = self.pop()?;

    let map_ref = expect_hashmap(&map)?;
    let func_ref = expect_function(&callback)?;

    let mut new_map = AtlasHashMap::new();

    for (key, value) in map_ref.borrow().entries() {
        let args = vec![value.clone(), key.to_value()];
        let result = self.call_function(&func_ref, args)?;

        if let Value::Bool(true) = result {
            new_map.insert(key.clone(), value);
        }
    }

    self.push(Value::HashMap(Rc::new(RefCell::new(new_map))));
}
```

### 2. HashSet Iteration Intrinsics

**Similar pattern for HashSet:**
```rust
// Interpreter
fn hashset_foreach_intrinsic(
    set: &Value,
    callback: &Value,
    interpreter: &mut Interpreter,
) -> Result<Value, RuntimeError> {
    let set_ref = expect_hashset(set)?;
    let func_ref = expect_function(callback)?;

    for element in set_ref.borrow().to_vec() {
        let args = vec![element.to_value()];
        interpreter.call_function(&func_ref, args)?;
    }

    Ok(Value::Null)
}

fn hashset_filter_intrinsic(
    set: &Value,
    callback: &Value,
    interpreter: &mut Interpreter,
) -> Result<Value, RuntimeError> {
    let set_ref = expect_hashset(set)?;
    let func_ref = expect_function(callback)?;

    let mut new_set = AtlasHashSet::new();

    for element in set_ref.borrow().to_vec() {
        let args = vec![element.to_value()];
        let result = interpreter.call_function(&func_ref, args)?;

        if let Value::Bool(true) = result {
            new_set.insert(element);
        }
    }

    Ok(Value::HashSet(Rc::new(RefCell::new(new_set))))
}
```

**VM implementation mirrors interpreter pattern with opcodes.**

### 3. Register Iteration Functions (`prelude.rs`)

```rust
// These are intrinsics (not pure stdlib functions)
// Register as native functions that delegate to interpreter/VM

register_native!(prelude, "hashMapForEach", hashmap_foreach, 2);
register_native!(prelude, "hashMapMap", hashmap_map, 2);
register_native!(prelude, "hashMapFilter", hashmap_filter, 2);
register_native!(prelude, "hashSetForEach", hashset_foreach, 2);
register_native!(prelude, "hashSetFilter", hashset_filter, 2);
```

### 4. Type Guards (`stdlib/types.rs`)

Add collection type guards:

```rust
fn is_hashmap(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    Ok(Value::Bool(matches!(args[0], Value::HashMap(_))))
}

fn is_hashset(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    Ok(Value::Bool(matches!(args[0], Value::HashSet(_))))
}

fn is_queue(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    Ok(Value::Bool(matches!(args[0], Value::Queue(_))))
}

fn is_stack(args: Vec<Value>, _env: &mut Environment) -> Result<Value, RuntimeError> {
    Ok(Value::Bool(matches!(args[0], Value::Stack(_))))
}

// Register
register_native!(prelude, "isHashMap", is_hashmap, 1);
register_native!(prelude, "isHashSet", is_hashset, 1);
register_native!(prelude, "isQueue", is_queue, 1);
register_native!(prelude, "isStack", is_stack, 1);
```

### 5. Performance Benchmarks (`benches/collection_benchmarks.rs`)

**Benchmark all collection operations:**
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use atlas_runtime::stdlib::collections::*;

fn bench_hashmap_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("HashMap");

    // Insert benchmark
    group.bench_function("insert_1000", |b| {
        b.iter(|| {
            let mut map = AtlasHashMap::new();
            for i in 0..1000 {
                let key = HashKey::Number(OrderedFloat(i as f64));
                map.insert(key, Value::Number(i as f64));
            }
            black_box(map)
        })
    });

    // Lookup benchmark
    group.bench_function("lookup_1000", |b| {
        let mut map = AtlasHashMap::new();
        for i in 0..1000 {
            let key = HashKey::Number(OrderedFloat(i as f64));
            map.insert(key, Value::Number(i as f64));
        }

        b.iter(|| {
            for i in 0..1000 {
                let key = HashKey::Number(OrderedFloat(i as f64));
                black_box(map.get(&key));
            }
        })
    });

    // Remove benchmark
    group.bench_function("remove_1000", |b| {
        b.iter(|| {
            let mut map = AtlasHashMap::new();
            for i in 0..1000 {
                let key = HashKey::Number(OrderedFloat(i as f64));
                map.insert(key, Value::Number(i as f64));
            }
            for i in 0..1000 {
                let key = HashKey::Number(OrderedFloat(i as f64));
                black_box(map.remove(&key));
            }
        })
    });

    group.finish();
}

fn bench_hashset_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("HashSet");

    // Insert benchmark
    group.bench_function("insert_1000", |b| {
        b.iter(|| {
            let mut set = AtlasHashSet::new();
            for i in 0..1000 {
                let key = HashKey::Number(OrderedFloat(i as f64));
                set.insert(key);
            }
            black_box(set)
        })
    });

    // Union benchmark
    group.bench_function("union_500_500", |b| {
        let mut set_a = AtlasHashSet::new();
        let mut set_b = AtlasHashSet::new();
        for i in 0..500 {
            set_a.insert(HashKey::Number(OrderedFloat(i as f64)));
            set_b.insert(HashKey::Number(OrderedFloat((i + 250) as f64)));
        }

        b.iter(|| {
            black_box(set_a.union(&set_b))
        })
    });

    // Intersection benchmark
    group.bench_function("intersection_500_500", |b| {
        let mut set_a = AtlasHashSet::new();
        let mut set_b = AtlasHashSet::new();
        for i in 0..500 {
            set_a.insert(HashKey::Number(OrderedFloat(i as f64)));
            set_b.insert(HashKey::Number(OrderedFloat((i + 250) as f64)));
        }

        b.iter(|| {
            black_box(set_a.intersection(&set_b))
        })
    });

    group.finish();
}

fn bench_queue_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Queue");

    group.bench_function("enqueue_dequeue_1000", |b| {
        b.iter(|| {
            let mut queue = AtlasQueue::new();
            for i in 0..1000 {
                queue.enqueue(Value::Number(i as f64));
            }
            for _ in 0..1000 {
                black_box(queue.dequeue());
            }
        })
    });

    group.finish();
}

fn bench_stack_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Stack");

    group.bench_function("push_pop_1000", |b| {
        b.iter(|| {
            let mut stack = AtlasStack::new();
            for i in 0..1000 {
                stack.push(Value::Number(i as f64));
            }
            for _ in 0..1000 {
                black_box(stack.pop());
            }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_hashmap_operations,
    bench_hashset_operations,
    bench_queue_operations,
    bench_stack_operations
);
criterion_main!(benches);
```

### 6. Documentation Updates (`docs/api/stdlib.md`)

**Add iteration section and real-world examples:**

```markdown
## Collection Iteration

All collections support iteration through callback-based functions.

### hashMapForEach
**Signature:** `hashMapForEach(map: HashMap, fn: (value: any, key: hashable) -> void) -> void`
**Behavior:** Calls function for each entry (value, key order)
**Example:**
```atlas
let map = hashMapFromEntries([["a", 1], ["b", 2]])
hashMapForEach(map, fn(value, key) {
    print(key + ": " + str(value))
})
// Output: a: 1, b: 2
```

### hashMapMap
**Signature:** `hashMapMap(map: HashMap, fn: (value: any, key: hashable) -> any) -> HashMap`
**Behavior:** Creates new HashMap with transformed values
**Example:**
```atlas
let map = hashMapFromEntries([["a", 1], ["b", 2]])
let doubled = hashMapMap(map, fn(value, key) { value * 2 })
// doubled: {"a": 2, "b": 4}
```

### hashMapFilter
**Signature:** `hashMapFilter(map: HashMap, fn: (value: any, key: hashable) -> bool) -> HashMap`
**Behavior:** Creates new HashMap with entries where predicate returns true
**Example:**
```atlas
let map = hashMapFromEntries([["a", 1], ["b", 2], ["c", 3]])
let filtered = hashMapFilter(map, fn(value, key) { value > 1 })
// filtered: {"b": 2, "c": 3}
```

[... similar for HashSet iteration ...]

## Real-World Examples

### Example 1: Word Frequency Counter
```atlas
fn wordFrequency(text: string) -> HashMap {
    let words = split(toLowerCase(text), " ")
    let freq = hashMapNew()

    forEach(words, fn(word) {
        let current = hashMapGet(freq, word)
        match current {
            Some(count) => hashMapPut(freq, word, count + 1)
            None => hashMapPut(freq, word, 1)
        }
    })

    return freq
}

let text = "hello world hello atlas world"
let freq = wordFrequency(text)
// freq: {"hello": 2, "world": 2, "atlas": 1}
```

### Example 2: Unique Elements Filter
```atlas
fn uniqueElements(arr: any[]) -> any[] {
    let set = hashSetNew()
    forEach(arr, fn(elem) { hashSetAdd(set, elem) })
    return hashSetToArray(set)
}

let data = [1, 2, 2, 3, 3, 3, 4]
let unique = uniqueElements(data)
// unique: [1, 2, 3, 4]
```

### Example 3: BFS with Queue
```atlas
fn bfs(graph: HashMap, start: string) -> string[] {
    let visited = hashSetNew()
    let queue = queueNew()
    let result = []

    queueEnqueue(queue, start)
    hashSetAdd(visited, start)

    while !queueIsEmpty(queue) {
        match queueDequeue(queue) {
            Some(node) => {
                push(result, node)

                let neighbors = hashMapGet(graph, node)
                match neighbors {
                    Some(list) => {
                        forEach(list, fn(neighbor) {
                            if !hashSetHas(visited, neighbor) {
                                queueEnqueue(queue, neighbor)
                                hashSetAdd(visited, neighbor)
                            }
                        })
                    }
                    None => {}
                }
            }
            None => {}
        }
    }

    return result
}
```

### Example 4: Expression Evaluator with Stack
```atlas
fn evaluateRPN(tokens: string[]) -> number {
    let stack = stackNew()

    forEach(tokens, fn(token) {
        if isOperator(token) {
            match stackPop(stack) {
                Some(b) => {
                    match stackPop(stack) {
                        Some(a) => {
                            let result = applyOperator(a, b, token)
                            stackPush(stack, result)
                        }
                        None => {}
                    }
                }
                None => {}
            }
        } else {
            stackPush(stack, parseNumber(token))
        }
    })

    match stackPop(stack) {
        Some(result) => result
        None => 0
    }
}

// ["2", "1", "+", "3", "*"] → ((2 + 1) * 3) = 9
```
```

## Tests (TDD - Use rstest)

### Collection Integration Tests (`tests/collection_integration_tests.rs`) - 20 tests

**HashMap + HashSet Integration:**
1. Build HashMap, extract keys with hashMapKeys, create HashSet
2. Filter HashMap, check filtered keys with HashSet
3. Map HashMap values, verify transformations
4. Combine multiple HashMaps with union via HashSet

**HashMap + Array Integration:**
5. HashMap to entries array, process with array functions
6. Array to HashMap with hashMapFromEntries
7. HashMapMap with array manipulation
8. HashMapFilter with array predicates

**HashSet Operations Integration:**
9. Multiple set operations chained (union → intersection → difference)
10. Set operations with array conversion
11. HashSet from array, array from HashSet roundtrip
12. Set membership testing in complex scenarios

**Queue + Stack Integration:**
13. Queue and Stack together (reversing with stack)
14. Queue for BFS, Stack for DFS simulation
15. Convert between Queue and Stack via toArray

**Cross-Collection:**
16. HashMap with Queue values
17. HashSet with Stack-based processing
18. All four collections in single algorithm
19. Type guards work for all collections
20. Reference semantics consistent across collections

**Minimum: 20 integration tests**

### Collection Iteration Tests (`tests/collection_iteration_tests.rs`) - 15 tests

**HashMap Iteration:**
1. hashMapForEach calls callback for each entry
2. hashMapMap creates new map with transformed values
3. hashMapFilter creates new map with filtered entries
4. HashMap iteration with complex callbacks
5. HashMap iteration parity (interpreter vs VM)

**HashSet Iteration:**
6. hashSetForEach calls callback for each element
7. hashSetFilter creates new set with filtered elements
8. HashSet iteration with complex callbacks
9. HashSet iteration parity (interpreter vs VM)

**Callback Edge Cases:**
10. Iteration with empty collections
11. Iteration with single element
12. Callback throws error (proper propagation)
13. Callback returns wrong type (error handling)
14. Nested iteration (map inside forEach)
15. Iteration doesn't modify original collection

**Minimum: 15 iteration tests**

**Total minimum test count:** 35 tests (20 integration + 15 iteration)

**Parity requirement:** All tests run in both interpreter and VM with identical results.

## Integration Points

- Uses: All collection types (HashMap, HashSet, Queue, Stack)
- Uses: Array intrinsic pattern for iteration
- Uses: Interpreter and VM execution contexts
- Creates: Collection iteration functions
- Creates: Type guard functions
- Creates: Performance benchmarks
- Updates: Documentation with real-world examples
- Output: Complete, world-class collection system

## Acceptance

- ✅ HashMap iteration (forEach, map, filter) implemented in interpreter and VM
- ✅ HashSet iteration (forEach, filter) implemented in interpreter and VM
- ✅ Type guards (isHashMap, isHashSet, isQueue, isStack) implemented
- ✅ All iteration functions registered in prelude
- ✅ 35+ integration and iteration tests pass
- ✅ 100% interpreter/VM parity verified for all iteration
- ✅ Performance benchmarks created and run
- ✅ HashMap operations O(1) average verified
- ✅ HashSet operations O(1) average verified
- ✅ Queue operations O(1) verified
- ✅ Stack operations O(1) verified
- ✅ Documentation complete with real-world examples
- ✅ No clippy warnings
- ✅ cargo test -p atlas-runtime passes (all tests)
- ✅ cargo bench -p atlas-runtime collection_benchmarks runs
- ✅ Decision logs DR-003, DR-004, DR-005 complete and accurate
- ✅ All collection phases (07a, 07b, 07c, 07d) complete

## References

**Decision Logs:**
- DR-003: Hash function design (referenced throughout)
- DR-004: Collection value representation (complete)
- DR-005: Collection API design (complete)

**Specifications:**
- `docs/specification/runtime.md` - Value representation, execution model
- `docs/api/stdlib.md` - Complete collection API reference

**Previous Phases:**
- `phase-07a-hash-infrastructure-hashmap.md` - HashMap foundation
- `phase-07b-hashset.md` - HashSet implementation
- `phase-07c-queue-stack.md` - Queue and Stack

**Implementation References:**
- Rust HashMap/HashSet documentation
- Array intrinsics pattern (DR-002)
- Performance benchmarking guide (phase-06c)

**Collection System Complete:** All 4 phases delivered, world-class quality achieved.
