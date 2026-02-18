# Phase 18d: Arc Refactor - Collections Stdlib

## 🎯 Scope: Collections Only

**What THIS phase does:** Update collections stdlib (HashMap, HashSet, Queue, Stack) to use Arc
**Depends on:** Phases 18a, 18b, 18c complete
**Estimated time:** 2-3 hours

---

## 🚨 DEPENDENCIES

**REQUIRED:** Phases 18a, 18b, 18c complete
**BLOCKS:** Phases 18e, 18f

**Verify dependencies:**
```bash
cargo check -p atlas-runtime --lib 2>&1 | grep -E "(string|array|json).rs"
# Should show zero errors in core stdlib
```

---

## Objective

Update the 4 collection modules to use Arc. Collections use `Rc<RefCell<CollectionType>>` pattern extensively, so there will be many changes.

**Modules in scope:**
- stdlib/collections/hashmap.rs
- stdlib/collections/hashset.rs
- stdlib/collections/queue.rs
- stdlib/collections/stack.rs

**NOT in scope:** datetime, http, regex, reflection (phase-18e)

---

## Files

**Update:** `crates/atlas-runtime/src/stdlib/collections/hashmap.rs` (~40 changes)
**Update:** `crates/atlas-runtime/src/stdlib/collections/hashset.rs` (~40 changes)
**Update:** `crates/atlas-runtime/src/stdlib/collections/queue.rs` (~20 changes)
**Update:** `crates/atlas-runtime/src/stdlib/collections/stack.rs` (~20 changes)

---

## Implementation

### GATE -1: Sanity Check ✅

```bash
cargo clean
cargo check -p atlas-runtime --lib
# Will fail (collections still use Rc) - expected
```

---

### GATE 0: Count Changes Needed

**See scope:**
```bash
grep -n "Rc::new\|Rc::clone" crates/atlas-runtime/src/stdlib/collections/hashmap.rs | wc -l
grep -n "Rc::new\|Rc::clone" crates/atlas-runtime/src/stdlib/collections/hashset.rs | wc -l
grep -n "Rc::new\|Rc::clone" crates/atlas-runtime/src/stdlib/collections/queue.rs | wc -l
grep -n "Rc::new\|Rc::clone" crates/atlas-runtime/src/stdlib/collections/stack.rs | wc -l
```

**Acceptance:**
- ✅ Know the scope (~120 total changes)
- ✅ Collections are the heaviest users of Rc

---

### GATE 1: Update hashmap.rs

**File:** `crates/atlas-runtime/src/stdlib/collections/hashmap.rs`

**Change ALL:**
- `Value::HashMap(Rc::new(RefCell::new(...)))` → `Value::HashMap(Arc::new(RefCell::new(...)))`
- `Rc::clone(&map)` → `Arc::clone(&map)` or `.clone()`
- `Value::String(Rc::new(...))` → `Value::String(Arc::new(...))`
- `Value::Array(Rc::new(RefCell::new(...)))` → `Value::Array(Arc::new(RefCell::new(...)))`

**Common patterns:**
```rust
// Before
let map = Rc::clone(&map_rc);

// After
let map = Arc::clone(&map_rc);

// Before
Ok(Value::HashMap(Rc::new(RefCell::new(new_map))))

// After
Ok(Value::HashMap(Arc::new(RefCell::new(new_map))))
```

**Test THIS MODULE:**
```bash
cargo check -p atlas-runtime --lib 2>&1 | grep hashmap.rs
```

**Acceptance:**
- ✅ All Rc::new → Arc::new in hashmap.rs
- ✅ All Rc::clone → Arc::clone
- ✅ Zero errors in hashmap.rs

---

### GATE 2: Update hashset.rs

**File:** `crates/atlas-runtime/src/stdlib/collections/hashset.rs`

**Change ALL:**
- `Value::HashSet(Rc::new(RefCell::new(...)))` → `Value::HashSet(Arc::new(RefCell::new(...)))`
- `Rc::clone(&set)` → `Arc::clone(&set)`
- Any string/array returns: `Rc::new` → `Arc::new`

**Test THIS MODULE:**
```bash
cargo check -p atlas-runtime --lib 2>&1 | grep hashset.rs
```

**Acceptance:**
- ✅ All Rc::new → Arc::new in hashset.rs
- ✅ All Rc::clone → Arc::clone
- ✅ Zero errors in hashset.rs

---

### GATE 3: Update queue.rs

**File:** `crates/atlas-runtime/src/stdlib/collections/queue.rs`

**Change ALL:**
- `Value::Queue(Rc::new(RefCell::new(...)))` → `Value::Queue(Arc::new(RefCell::new(...)))`
- `Rc::clone(&queue)` → `Arc::clone(&queue)`

**Test THIS MODULE:**
```bash
cargo check -p atlas-runtime --lib 2>&1 | grep queue.rs
```

**Acceptance:**
- ✅ All Rc::new → Arc::new in queue.rs
- ✅ All Rc::clone → Arc::clone
- ✅ Zero errors in queue.rs

---

### GATE 4: Update stack.rs

**File:** `crates/atlas-runtime/src/stdlib/collections/stack.rs`

**Change ALL:**
- `Value::Stack(Rc::new(RefCell::new(...)))` → `Value::Stack(Arc::new(RefCell::new(...)))`
- `Rc::clone(&stack)` → `Arc::clone(&stack)`

**Test THIS MODULE:**
```bash
cargo check -p atlas-runtime --lib 2>&1 | grep stack.rs
```

**Acceptance:**
- ✅ All Rc::new → Arc::new in stack.rs
- ✅ All Rc::clone → Arc::clone
- ✅ Zero errors in stack.rs

---

### GATE 5: Verify No Rc Remains in Collections

**Search all 4 modules:**
```bash
grep "Rc::" crates/atlas-runtime/src/stdlib/collections/*.rs
```

**Acceptance:**
- ✅ Zero matches (no Rc left)
- ✅ All 4 collection modules use Arc only

---

### GATE 6: Compile Check

**Test all collections together:**
```bash
cargo check -p atlas-runtime --lib
```

**Acceptance:**
- ✅ Collections compile
- ✅ Still errors from datetime/http/regex (expected - phase 18e)

---

### GATE 7: Run Collection Tests

**Test ONLY collections:**
```bash
cargo test -p atlas-runtime test_hashmap -- --exact
cargo test -p atlas-runtime test_hashset -- --exact
cargo test -p atlas-runtime test_queue -- --exact
cargo test -p atlas-runtime test_stack -- --exact
```

**Acceptance:**
- ✅ All collection tests pass
- ✅ No regressions from Arc change

---

### GATE 8: Clippy & Format

```bash
cargo clippy -p atlas-runtime --lib -- -D warnings 2>&1 | grep collections
cargo fmt -- crates/atlas-runtime/src/stdlib/collections/*.rs
```

**Acceptance:**
- ✅ Zero clippy warnings in collections
- ✅ Code formatted

---

## Acceptance Criteria

**ALL must be met:**

1. ✅ hashmap.rs uses Arc::new (zero Rc::new)
2. ✅ hashset.rs uses Arc::new (zero Rc::new)
3. ✅ queue.rs uses Arc::new (zero Rc::new)
4. ✅ stack.rs uses Arc::new (zero Rc::new)
5. ✅ All 4 modules compile
6. ✅ All collection tests pass
7. ✅ Zero clippy warnings in collections
8. ✅ Code formatted

**DO NOT:**
- ❌ Touch datetime/http/regex/reflection (that's phase-18e)
- ❌ Touch all tests (that's phase-18f)
- ❌ Run full test suite yet (will fail until 18e done)

---

## Handoff

**Commit message:**
```
refactor(stdlib): Update collections to use Arc (phase 18d)

Part 4/6 of Arc refactor for thread safety.

**Changes:**
- hashmap.rs: All Rc → Arc (~40 changes)
- hashset.rs: All Rc → Arc (~40 changes)
- queue.rs: All Rc → Arc (~20 changes)
- stack.rs: All Rc → Arc (~20 changes)

**Tests:**
- All collection tests pass
- HashMap: ✅
- HashSet: ✅
- Queue: ✅
- Stack: ✅

**Scope:** 4 collection modules (~120 changes)

**Status:** Collections updated, advanced stdlib remaining (18e)

**Next:** Phase-18e (datetime, http, regex, reflection)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

**Update STATUS.md:**
- Foundation: Mark "Phase-18d: Arc Refactor - Collections (Complete)"
- Note: "4/6 sub-phases complete"

---

## Notes

**Why collections separate?**
- Heavy users of `Rc<RefCell<>>` pattern
- ~120 changes (too many to mix with other stdlib)
- Can test collections independently
- Clear scope boundary

**What still breaks:**
- Advanced stdlib (datetime, http, regex, reflection)
- Full test suite
- **Expected** - will be fixed in 18e and 18f

**Time estimate:** 2-3 hours (~120 mechanical changes, plus testing)
