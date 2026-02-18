# Phase 18c: Arc Refactor - Core Stdlib

## 🎯 Scope: Core Stdlib Modules Only

**What THIS phase does:** Update core stdlib (string, array, math, json, io) to use Arc
**Depends on:** Phase-18a (Value uses Arc), 18b (engines use Arc)
**Estimated time:** 2-3 hours

---

## 🚨 DEPENDENCIES

**REQUIRED:** Phases 18a, 18b complete
**BLOCKS:** Phases 18d, 18e, 18f

**Verify dependencies:**
```bash
grep "use std::sync::Arc" crates/atlas-runtime/src/value.rs
grep "Arc::new" crates/atlas-runtime/src/interpreter.rs | head -1
# Both must show Arc usage
```

---

## Objective

Update the 5 core stdlib modules to construct Values using Arc instead of Rc. These are the fundamental modules used everywhere.

**Modules in scope:**
- stdlib/string.rs
- stdlib/array.rs
- stdlib/math.rs
- stdlib/json.rs
- stdlib/io.rs

**NOT in scope:** collections, datetime, http, regex, reflection (later phases)

---

## Files

**Update:** `crates/atlas-runtime/src/stdlib/string.rs` (~20 changes)
**Update:** `crates/atlas-runtime/src/stdlib/array.rs` (~15 changes)
**Update:** `crates/atlas-runtime/src/stdlib/math.rs` (~5 changes)
**Update:** `crates/atlas-runtime/src/stdlib/json.rs` (~25 changes)
**Update:** `crates/atlas-runtime/src/stdlib/io.rs` (~10 changes)

---

## Implementation

### GATE -1: Sanity Check ✅

```bash
cargo clean
cargo check -p atlas-runtime --lib
# Will fail (stdlib still uses Rc) - expected
```

---

### GATE 0: Count Changes Needed

**See scope:**
```bash
grep -n "Rc::new" crates/atlas-runtime/src/stdlib/string.rs | wc -l
grep -n "Rc::new" crates/atlas-runtime/src/stdlib/array.rs | wc -l
grep -n "Rc::new" crates/atlas-runtime/src/stdlib/math.rs | wc -l
grep -n "Rc::new" crates/atlas-runtime/src/stdlib/json.rs | wc -l
grep -n "Rc::new" crates/atlas-runtime/src/stdlib/io.rs | wc -l
```

**Acceptance:**
- ✅ Know the scope (usually 5-25 per file)
- ✅ Ready to proceed systematically

---

### GATE 1: Update string.rs

**File:** `crates/atlas-runtime/src/stdlib/string.rs`

**Change ALL:**
- `Value::String(Rc::new(...))` → `Value::String(Arc::new(...))`
- `Rc::clone(...)` → `Arc::clone(...)` or `.clone()`

**Common patterns:**
```rust
// Before
Ok(Value::String(Rc::new(result)))

// After
Ok(Value::String(Arc::new(result)))
```

**Test THIS MODULE:**
```bash
cargo check -p atlas-runtime --lib 2>&1 | grep string.rs
```

**Acceptance:**
- ✅ All Rc::new → Arc::new in string.rs
- ✅ Zero errors in string.rs

---

### GATE 2: Update array.rs

**File:** `crates/atlas-runtime/src/stdlib/array.rs`

**Change ALL:**
- `Value::String(Rc::new(...))` → `Value::String(Arc::new(...))`
- `Value::Array(Rc::new(RefCell::new(...)))` → `Value::Array(Arc::new(RefCell::new(...)))`

**Test THIS MODULE:**
```bash
cargo check -p atlas-runtime --lib 2>&1 | grep array.rs
```

**Acceptance:**
- ✅ All Rc::new → Arc::new in array.rs
- ✅ Zero errors in array.rs

---

### GATE 3: Update math.rs

**File:** `crates/atlas-runtime/src/stdlib/math.rs`

**Change ALL:**
- Probably minimal (math mostly returns Number)
- Check for any string conversions: `Rc::new` → `Arc::new`

**Test THIS MODULE:**
```bash
cargo check -p atlas-runtime --lib 2>&1 | grep math.rs
```

**Acceptance:**
- ✅ All Rc::new → Arc::new in math.rs
- ✅ Zero errors in math.rs

---

### GATE 4: Update json.rs

**File:** `crates/atlas-runtime/src/stdlib/json.rs`

**Change ALL:**
- `Value::String(Rc::new(...))` → `Value::String(Arc::new(...))`
- `Value::Array(Rc::new(RefCell::new(...)))` → `Value::Array(Arc::new(RefCell::new(...)))`
- JsonValue handling may have many Rc references

**Test THIS MODULE:**
```bash
cargo check -p atlas-runtime --lib 2>&1 | grep json.rs
```

**Acceptance:**
- ✅ All Rc::new → Arc::new in json.rs
- ✅ Zero errors in json.rs

---

### GATE 5: Update io.rs

**File:** `crates/atlas-runtime/src/stdlib/io.rs`

**Change ALL:**
- `Value::String(Rc::new(...))` → `Value::String(Arc::new(...))`
- File reading returns strings (likely many Rc::new)

**Test THIS MODULE:**
```bash
cargo check -p atlas-runtime --lib 2>&1 | grep io.rs
```

**Acceptance:**
- ✅ All Rc::new → Arc::new in io.rs
- ✅ Zero errors in io.rs

---

### GATE 6: Verify No Rc Remains in Core Stdlib

**Search all 5 modules:**
```bash
grep "Rc::" crates/atlas-runtime/src/stdlib/string.rs
grep "Rc::" crates/atlas-runtime/src/stdlib/array.rs
grep "Rc::" crates/atlas-runtime/src/stdlib/math.rs
grep "Rc::" crates/atlas-runtime/src/stdlib/json.rs
grep "Rc::" crates/atlas-runtime/src/stdlib/io.rs
```

**Acceptance:**
- ✅ Zero matches (no Rc left)
- ✅ All 5 modules use Arc only

---

### GATE 7: Compile Check

**Test all 5 together:**
```bash
cargo check -p atlas-runtime --lib
```

**Acceptance:**
- ✅ Core stdlib compiles
- ✅ Still errors from collections/datetime/http (expected)

---

### GATE 8: Clippy & Format

```bash
cargo clippy -p atlas-runtime --lib -- -D warnings 2>&1 | grep -E "(string|array|math|json|io).rs"
cargo fmt -- crates/atlas-runtime/src/stdlib/*.rs
```

**Acceptance:**
- ✅ Zero clippy warnings in these 5 modules
- ✅ Code formatted

---

## Acceptance Criteria

**ALL must be met:**

1. ✅ string.rs uses Arc::new (zero Rc::new)
2. ✅ array.rs uses Arc::new (zero Rc::new)
3. ✅ math.rs uses Arc::new (zero Rc::new)
4. ✅ json.rs uses Arc::new (zero Rc::new)
5. ✅ io.rs uses Arc::new (zero Rc::new)
6. ✅ All 5 modules compile
7. ✅ Zero clippy warnings in these modules
8. ✅ Code formatted

**DO NOT:**
- ❌ Touch collections/*.rs (that's phase-18d)
- ❌ Touch datetime/http/regex/reflection (that's phase-18e)
- ❌ Touch tests (that's phase-18f)
- ❌ Run full test suite (will fail until 18d/e done)

---

## Handoff

**Commit message:**
```
refactor(stdlib): Update core stdlib to use Arc (phase 18c)

Part 3/6 of Arc refactor for thread safety.

**Changes:**
- string.rs: All Rc::new → Arc::new
- array.rs: All Rc::new → Arc::new
- math.rs: All Rc::new → Arc::new
- json.rs: All Rc::new → Arc::new
- io.rs: All Rc::new → Arc::new

**Scope:** 5 core stdlib modules

**Status:** Core stdlib updated, collections still need update (18d)

**Next:** Phase-18d (collections stdlib)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

**Update STATUS.md:**
- Foundation: Mark "Phase-18c: Arc Refactor - Core Stdlib (Complete)"
- Note: "3/6 sub-phases complete"

---

## Notes

**Why these 5 together?**
- Core functionality (used everywhere)
- No complex interdependencies
- Can be done as a unit
- ~75 total changes across 5 files (manageable)

**What still breaks:**
- Collections stdlib (phase-18d)
- Advanced stdlib (phase-18e)
- Tests (phase-18f)
- **Expected** - will be fixed in next phases

**Time estimate:** 2-3 hours (mechanical but need to check each function)
