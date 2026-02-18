# Phase 18b: Arc Refactor - Runtime Engines

## 🎯 Scope: Interpreter + VM Only

**What THIS phase does:** Update interpreter.rs and vm.rs to use Arc instead of Rc
**Depends on:** Phase-18a (Value enum already uses Arc)
**Estimated time:** 2-3 hours

---

## 🚨 DEPENDENCIES

**REQUIRED:** Phase-18a complete (Value uses Arc)
**BLOCKS:** Phases 18c, 18d, 18e, 18f

**Verify dependency:**
```bash
grep "use std::sync::Arc" crates/atlas-runtime/src/value.rs
# Must show Arc import (not Rc)
```

---

## Objective

Update the two execution engines (interpreter and VM) to construct Values using Arc instead of Rc. The Value enum already uses Arc (from 18a), so this is about updating callsites.

**Focus:** ONLY interpreter.rs and vm.rs - no stdlib yet.

---

## Files

**Update:** `crates/atlas-runtime/src/interpreter.rs` (~30 changes)
**Update:** `crates/atlas-runtime/src/vm/mod.rs` (~30 changes)

---

## Implementation

### GATE -1: Sanity Check ✅

```bash
cargo clean
cargo check -p atlas-runtime
# Will fail (interpreter/VM still use Rc) - expected
```

---

### GATE 0: Verify Dependency Complete

**Check Value uses Arc:**
```bash
grep "String(Arc<String>)" crates/atlas-runtime/src/value.rs
grep "String(Rc<String>)" crates/atlas-runtime/src/value.rs
```

**Acceptance:**
- ✅ First grep matches (Arc found)
- ✅ Second grep empty (no Rc)
- ✅ Phase-18a confirmed complete

---

### GATE 1: Update Interpreter Imports

**File:** `crates/atlas-runtime/src/interpreter.rs`

**Check current imports:**
```bash
grep "use std::rc::Rc" crates/atlas-runtime/src/interpreter.rs
```

**If Rc imported, change to:**
```rust
use std::sync::Arc;
```

**If Rc not explicitly imported, no change needed** (Value handles it).

**Test:**
```bash
cargo check -p atlas-runtime --lib
```

**Acceptance:**
- ✅ Imports updated
- ✅ Still has errors (Rc::new calls) - expected

---

### GATE 2: Update Interpreter Value Construction

**File:** `crates/atlas-runtime/src/interpreter.rs`

**Find and replace:**
```bash
# See how many need changing
grep -n "Rc::new" crates/atlas-runtime/src/interpreter.rs | wc -l
```

**Change ALL:**
- `Value::String(Rc::new(...))` → `Value::String(Arc::new(...))`
- `Value::Array(Rc::new(RefCell::new(...)))` → `Value::Array(Arc::new(RefCell::new(...)))`
- Any other `Rc::new` creating Values

**Common patterns:**
```rust
// Before
Value::String(Rc::new(format!("...")))

// After
Value::String(Arc::new(format!("...")))
```

**Test:**
```bash
cargo check -p atlas-runtime --lib 2>&1 | grep interpreter
```

**Acceptance:**
- ✅ All Rc::new → Arc::new in interpreter.rs
- ✅ Fewer errors in interpreter

---

### GATE 3: Update Interpreter Clone Operations

**File:** `crates/atlas-runtime/src/interpreter.rs`

**Find cloning patterns:**
```bash
grep -n "Rc::clone" crates/atlas-runtime/src/interpreter.rs
```

**Change if found:**
- `Rc::clone(&value)` → `Arc::clone(&value)`
- Or just `.clone()` (works for both)

**Test:**
```bash
cargo check -p atlas-runtime --lib
```

**Acceptance:**
- ✅ All Rc clones updated
- ✅ interpreter.rs compiles (or close to it)

---

### GATE 4: Update VM Imports

**File:** `crates/atlas-runtime/src/vm/mod.rs`

**Same as GATE 1 but for VM:**
```bash
grep "use std::rc::Rc" crates/atlas-runtime/src/vm/mod.rs
```

**If found, change to Arc import.**

**Test:**
```bash
cargo check -p atlas-runtime --lib
```

**Acceptance:**
- ✅ VM imports updated

---

### GATE 5: Update VM Value Construction

**File:** `crates/atlas-runtime/src/vm/mod.rs`

**Find and replace:**
```bash
grep -n "Rc::new" crates/atlas-runtime/src/vm/mod.rs | wc -l
```

**Change ALL:**
- Same pattern as interpreter: `Rc::new` → `Arc::new`

**Special attention to:**
- Constant pool (if it stores Rc<Value>)
- Stack operations
- Instruction execution

**Test:**
```bash
cargo check -p atlas-runtime --lib 2>&1 | grep "vm::"
```

**Acceptance:**
- ✅ All Rc::new → Arc::new in vm/mod.rs
- ✅ Fewer errors in VM

---

### GATE 6: Update VM Clone Operations

**File:** `crates/atlas-runtime/src/vm/mod.rs`

**Find and update:**
```bash
grep -n "Rc::clone" crates/atlas-runtime/src/vm/mod.rs
```

**Change to Arc::clone or .clone()**

**Test:**
```bash
cargo check -p atlas-runtime --lib
```

**Acceptance:**
- ✅ All clones updated
- ✅ vm/mod.rs compiles (or close)

---

### GATE 7: Verify No Rc Remains

**Search both files:**
```bash
grep "Rc::" crates/atlas-runtime/src/interpreter.rs
grep "Rc::" crates/atlas-runtime/src/vm/mod.rs
grep "std::rc" crates/atlas-runtime/src/interpreter.rs
grep "std::rc" crates/atlas-runtime/src/vm/mod.rs
```

**Acceptance:**
- ✅ Zero matches (no Rc left)
- ✅ Only Arc used

---

### GATE 8: Clippy & Format

```bash
cargo clippy -p atlas-runtime --lib -- -D warnings 2>&1 | grep -E "(interpreter|vm)"
cargo fmt -- crates/atlas-runtime/src/interpreter.rs
cargo fmt -- crates/atlas-runtime/src/vm/mod.rs
```

**Acceptance:**
- ✅ Zero clippy warnings in interpreter/VM
- ✅ Code formatted

---

## Acceptance Criteria

**ALL must be met:**

1. ✅ interpreter.rs uses Arc::new (zero Rc::new)
2. ✅ vm/mod.rs uses Arc::new (zero Rc::new)
3. ✅ Both files use Arc::clone or .clone() (zero Rc::clone)
4. ✅ No Rc imports in either file
5. ✅ Both files compile (with stdlib errors - expected)
6. ✅ Zero clippy warnings in interpreter/VM
7. ✅ Code formatted

**DO NOT:**
- ❌ Touch stdlib/*.rs (that's phase-18c/d/e)
- ❌ Touch tests (that's phase-18f)
- ❌ Run full test suite (will fail until stdlib updated)

---

## Handoff

**Commit message:**
```
refactor(runtime): Update interpreter and VM to use Arc (phase 18b)

Part 2/6 of Arc refactor for thread safety.

**Changes:**
- Interpreter: All Rc::new → Arc::new
- VM: All Rc::new → Arc::new
- Both engines now construct Arc-based Values

**Scope:** interpreter.rs, vm/mod.rs only

**Status:** Engines updated, stdlib still needs update (18c/d/e)

**Next:** Phase-18c (core stdlib)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

**Update STATUS.md:**
- Foundation: Mark "Phase-18b: Arc Refactor - Engines (Complete)"
- Note: "2/6 sub-phases complete"

---

## Notes

**Why split from stdlib?**
- Different codebases (interpreter/VM vs stdlib)
- Clear separation of concerns
- Easier to verify (2 files vs 12 files)
- Can commit progress incrementally

**What still breaks:**
- Stdlib will have type errors (uses Rc)
- Tests will fail
- **This is expected** - stdlib gets fixed in 18c/d/e

**Time estimate:** 2-3 hours (mechanical but need to check both engines carefully)
