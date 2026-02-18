# Phase 07d: Collection Integration + Iteration

## Dependencies

**Required:** Phases 07a (HashMap), 07b (HashSet), 07c (Queue/Stack) complete

**Verification:**
```bash
# Check all collection implementations exist
ls crates/atlas-runtime/src/stdlib/collections/hashmap.rs
ls crates/atlas-runtime/src/stdlib/collections/hashset.rs
ls crates/atlas-runtime/src/stdlib/collections/queue.rs
ls crates/atlas-runtime/src/stdlib/collections/stack.rs
ls crates/atlas-runtime/src/stdlib/collections/hash.rs

# Verify Value variants
grep "HashMap\|HashSet\|Queue\|Stack" crates/atlas-runtime/src/value.rs | grep "Rc<RefCell"

# Verify all collection tests pass (if they exist from previous phases)
cargo test -p atlas-runtime -- hashmap --nocapture
cargo test -p atlas-runtime -- hashset --nocapture

# Clean build check
cargo clean && cargo check -p atlas-runtime
```

**If missing:** Complete phases 07a, 07b, 07c first

---

## Objective

Add iteration support (forEach, map, filter) for HashMap and HashSet as interpreter/VM intrinsics. Create cross-collection integration tests. Add performance benchmarks for all collections. Complete documentation with real-world examples.

---

## Files

**Update:** `crates/atlas-runtime/src/interpreter/expr.rs` (~200 lines - intrinsics)
**Update:** `crates/atlas-runtime/src/vm/mod.rs` (~200 lines - intrinsics)
**Update:** `crates/atlas-runtime/src/stdlib/mod.rs` (~20 lines - register intrinsic names)
**Extend:** `crates/atlas-runtime/tests/collection_iteration_tests.rs` (already exists - add more tests)

**Total new code:** ~450 lines (intrinsics)
**Total tests:** ~1,000 lines (30+ test cases)

**Note:** Benchmarks deferred to separate phase per DR-006 (scope management)

---

## Dependencies (Components)

- All collection types (HashMap, HashSet, Queue, Stack from 07a-c)
- Array intrinsics pattern (examine array map/filter/forEach implementation)
- Interpreter/VM execution contexts
- Criterion benchmark library (from phase 06c)

---

## Implementation Notes

**Key patterns to analyze:**
- Examine array intrinsics in interpreter.rs and vm.rs (map, filter, forEach)
- Follow callback intrinsic pattern from memory/patterns.md
- Reference DR-005 (Collection API Design) for iteration strategy

**HashMap/HashSet iteration functions (6 total):**
- `hashMapForEach(map, fn)` - iterate with side effects
- `hashMapMap(map, fn)` - transform values, return new map
- `hashMapFilter(map, fn)` - filter entries, return new map
- `hashSetForEach(set, fn)` - iterate with side effects
- `hashSetFilter(set, fn)` - filter elements, return new set
- `hashSetMap(set, fn)` - transform elements to array (Set→Array)

**Implementation locations:**
- Interpreter intrinsics: interpreter/expr.rs (in eval_expr for Expr::Call)
- VM intrinsics: vm/mod.rs (in execute_call_intrinsic)
- Function registration: stdlib/mod.rs (is_array_intrinsic function)

**Integration tests cover:**
- Cross-collection operations (Map→Set, Set→Array, etc.)
- Mixed type scenarios
- Edge cases (empty collections, large collections)
- Error scenarios (wrong types, callback errors)

**Note on Benchmarks:**
Per DR-006, benchmarks deferred to separate dedicated performance phase. Focus for this phase:
- Complete functional implementation
- Comprehensive testing (33+ tests)
- 100% parity verification
- Correctness over performance optimization

---

## Tests (TDD Approach)

### HashMap Iteration Tests (10 tests)
1. forEach iterates all entries with correct order
2. map transforms values, preserves keys
3. filter keeps entries matching predicate
4. Callback receives (value, key) arguments
5. Empty map iteration (no callback calls)
6. Callback errors propagate correctly
7. Nested callbacks work correctly
8. Large map iteration (1000+ entries)
9. Mixed value types in callbacks
10. Chaining operations (map then filter)

### HashSet Iteration Tests (8 tests)
1. forEach iterates all elements
2. filter keeps elements matching predicate
3. map transforms to array (not set)
4. Empty set iteration
5. Callback errors propagate
6. Large set iteration
7. Set operations + iteration combo
8. Chaining filter operations

### Integration Tests (15 tests)
1. Convert HashMap keys to HashSet
2. Convert HashSet to Array and back
3. Filter HashMap, convert values to Set
4. Queue→Array→HashSet pipeline
5. Stack→Array→HashMap pipeline
6. Mixed collection operations
7. Deep nesting (Map of Arrays, Array of Maps)
8. Error propagation across collections
9. Reference semantics verification (shared collections)
10. Multiple collection types in single function
11. Collection type guards work correctly
12. Empty collection edge cases
13. Large-scale integration (10K+ elements)
14. Memory behavior (no leaks with Rc<RefCell<>>)
15. Parity verification (interpreter vs VM results)

**Minimum test count:** 33 tests (18 iteration + 15 integration)

**Note:** Benchmark tests deferred per DR-006.

**Parity requirement:** All tests run in both interpreter and VM with identical results.

---

## Acceptance Criteria

- ✅ 6 iteration functions implemented (3 HashMap + 3 HashSet)
- ✅ hashMapForEach, hashMapMap, hashMapFilter work correctly
- ✅ hashSetForEach, hashSetFilter, hashSetMap work correctly
- ✅ All intrinsics implemented in both interpreter and VM
- ✅ Callback pattern matches array intrinsics (consistent API)
- ✅ 33+ tests pass (10 HashMap + 8 HashSet + 15 integration)
- ✅ Integration tests verify cross-collection operations
- ✅ Error handling complete (callback errors, type errors)
- ✅ 100% interpreter/VM parity verified
- ✅ No clippy warnings
- ✅ cargo test -p atlas-runtime passes
- ✅ Decision logs DR-005 (Collection API), DR-006 (Benchmarks deferred) referenced

---

## References

**Decision Logs:** DR-005 (Collection API Design and Iteration)
**Specifications:**
- docs/specification/runtime.md (Value representation, intrinsics)
- memory/patterns.md (Callback intrinsic pattern - see "Intrinsic Pattern" section)
**Previous phases:**
- phase-07a-hash-infrastructure-hashmap.md (HashMap)
- phase-07b-hashset.md (HashSet)
- phase-07c-queue-stack.md (Queue/Stack)
**Next phase:** phase-08-regex.md (Regular expressions)
