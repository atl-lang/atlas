# Phase 18a: Arc Refactor - Core Value Enum

## 🎯 Scope: MINIMAL - Foundation Only

**What THIS phase does:** Refactor core `Value` enum from `Rc<T>` to `Arc<T>`
**What this UNLOCKS:** Phase-18b (engines), 18c-e (stdlib), 18f (tests)
**Estimated time:** 1-2 hours

---

## 🚨 DEPENDENCIES

**REQUIRED:** None - This IS the foundation
**BLOCKS:** Phases 18b, 18c, 18d, 18e, 18f (all depend on this)

---

## Objective

Refactor the core `Value` enum and its helper methods from `Rc<T>` to `Arc<T>`. This is the foundational change that enables thread safety. All other phases depend on this being done first and correctly.

**Focus:** ONLY `value.rs` - nothing else.

---

## Files

**Update:** `crates/atlas-runtime/src/value.rs` (~50 changes)

---

## Implementation

### GATE -1: Sanity Check ✅

```bash
cargo clean
cargo check -p atlas-runtime
# Must pass before starting
```

---

### GATE 0: Verify Current State

**Check current Rc usage:**
```bash
grep -n "use std::rc::Rc" crates/atlas-runtime/src/value.rs
grep -n "Rc<" crates/atlas-runtime/src/value.rs | head -20
```

**Acceptance:**
- ✅ Confirms Rc is used extensively
- ✅ Know the scope of changes needed

---

### GATE 1: Update Imports

**File:** `crates/atlas-runtime/src/value.rs`

**Change:**
```rust
// Remove
use std::rc::Rc;

// Add
use std::sync::Arc;
```

**Test:**
```bash
cargo check -p atlas-runtime 2>&1 | head -20
# Will fail with Rc errors - that's expected
```

**Acceptance:**
- ✅ Import updated
- ✅ Compilation fails (expected - Rc not imported)

---

### GATE 2: Update Value Enum Variants

**File:** `crates/atlas-runtime/src/value.rs`

**Find and replace ALL in Value enum:**
- `Rc<String>` → `Arc<String>`
- `Rc<RefCell<Vec<Value>>>` → `Arc<RefCell<Vec<Value>>>`
- `Rc<RefCell<AtlasHashMap>>` → `Arc<RefCell<AtlasHashMap>>`
- `Rc<RefCell<AtlasHashSet>>` → `Arc<RefCell<AtlasHashSet>>`
- `Rc<RefCell<AtlasQueue>>` → `Arc<RefCell<AtlasQueue>>`
- `Rc<RefCell<AtlasStack>>` → `Arc<RefCell<AtlasStack>>`
- Any other `Rc<...>` in enum

**Test:**
```bash
cargo check -p atlas-runtime 2>&1 | grep error | head -10
```

**Acceptance:**
- ✅ All Value enum variants use Arc
- ✅ Fewer errors (but still some from helper methods)

---

### GATE 3: Update Value Constructor Helpers

**File:** `crates/atlas-runtime/src/value.rs`

**Find all methods that construct Values:**
- Look for `Rc::new(...)` in impl blocks
- Change to `Arc::new(...)`
- Look for `Rc::clone(...)` or `.clone()` on Rc values
- Works same with Arc (Arc::clone or .clone())

**Common patterns:**
```rust
// Before
pub fn new_string(s: String) -> Self {
    Value::String(Rc::new(s))
}

// After
pub fn new_string(s: String) -> Self {
    Value::String(Arc::new(s))
}
```

**Test:**
```bash
cargo check -p atlas-runtime
```

**Acceptance:**
- ✅ All Rc::new → Arc::new in value.rs
- ✅ value.rs compiles (may have errors from other files - ignore)

---

### GATE 4: Update Pattern Matching

**File:** `crates/atlas-runtime/src/value.rs`

**Find match statements on Value:**
- Check for any `Rc::clone()` calls in pattern arms
- Verify Arc::clone works the same
- Update any Rc-specific logic (shouldn't be any)

**Test:**
```bash
cargo check -p atlas-runtime --lib
```

**Acceptance:**
- ✅ All pattern matching updated
- ✅ No Rc references remain in value.rs
- ✅ value.rs module compiles

---

### GATE 5: Verify No Rc Remains

**Search for any Rc left:**
```bash
grep -n "Rc::" crates/atlas-runtime/src/value.rs
grep -n "std::rc" crates/atlas-runtime/src/value.rs
```

**Acceptance:**
- ✅ Zero matches (no Rc left in value.rs)
- ✅ Only Arc used

---

### GATE 6: Clippy & Format

```bash
cargo clippy -p atlas-runtime --lib -- -D warnings 2>&1 | grep value.rs
cargo fmt -- crates/atlas-runtime/src/value.rs
```

**Acceptance:**
- ✅ Zero clippy warnings in value.rs
- ✅ Code formatted

---

## Acceptance Criteria

**ALL must be met:**

1. ✅ Value enum uses Arc<T> everywhere (zero Rc)
2. ✅ All imports updated (Arc not Rc)
3. ✅ All constructor helpers updated (Arc::new)
4. ✅ All pattern matching updated
5. ✅ value.rs compiles successfully
6. ✅ Zero Rc references in value.rs
7. ✅ Zero clippy warnings in value.rs

**DO NOT:**
- ❌ Touch interpreter.rs (that's phase-18b)
- ❌ Touch vm.rs (that's phase-18b)
- ❌ Touch stdlib/*.rs (that's phase-18c/d/e)
- ❌ Run full test suite (will fail until other phases done)

---

## Handoff

**Commit message:**
```
refactor(runtime): Change Value enum from Rc to Arc (phase 18a)

Part 1/6 of Arc refactor for thread safety.

**Changes:**
- Value enum: All Rc<T> → Arc<T>
- Imports: std::rc::Rc → std::sync::Arc
- Constructor helpers: Rc::new → Arc::new
- value.rs fully updated and compiling

**Scope:** ONLY value.rs (foundation layer)

**Next:** Phase-18b (interpreter + VM)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

**Update STATUS.md:**
- Foundation: Mark "Phase-18a: Arc Refactor - Core Value (In Progress)"
- Note: "1/6 sub-phases complete"

---

## Notes

**Why split this?**
- Isolated scope (just value.rs)
- Clear dependencies (everything else depends on this)
- Easy to verify (one file)
- Low risk (if it compiles, it's right)

**What breaks during this phase:**
- Interpreter will have type errors (uses Rc)
- VM will have type errors (uses Rc)
- Stdlib will have type errors (uses Rc)
- Tests will fail
- **This is expected** - they get fixed in subsequent phases

**Time estimate:** 1-2 hours (it's mechanical search/replace)
