# Phase 18: Value Arc Refactor (Enable Thread Safety & Async)

## 🚨 CRITICAL - Architecture Foundation Fix

**Status:** BLOCKING async concurrency (Phase-11c), future parallelism, threading
**Priority:** HIGHEST - Everything async/concurrent depends on this

---

## 🚨 DEPENDENCIES - CHECK BEFORE STARTING

**REQUIRED:** None - This IS the foundation fix

**Current State Verification:**
```bash
# Verify current Rc usage
grep "Rc<String>" crates/atlas-runtime/src/value.rs
grep "Rc<RefCell" crates/atlas-runtime/src/value.rs

# Should show Rc everywhere (we're fixing this)
```

**What's broken:**
- ❌ Value uses `Rc<T>` (not Send - can't cross thread boundaries)
- ❌ Blocks tokio::spawn() with multi-threaded runtime
- ❌ Blocks all future concurrency features
- ❌ Phase-11c (async primitives) currently blocked

**What we're fixing:**
- ✅ Change to `Arc<T>` (Send + Sync - thread-safe)
- ✅ Enable true async concurrency
- ✅ Unblock Phase-11c and all future parallel features

---

## Objective

Refactor `Value` enum from `Rc<T>` to `Arc<T>` for thread safety. Enable async concurrency, tokio::spawn(), and future parallelism features. This is a foundational architecture change that unblocks all async/concurrent work.

## Design Decision

**Current (BROKEN for async):**
```rust
pub enum Value {
    String(Rc<String>),              // ❌ Not Send
    Array(Rc<RefCell<Vec<Value>>>),  // ❌ Not Send
    HashMap(Rc<RefCell<AtlasHashMap>>), // ❌ Not Send
    // ... all collections use Rc
}
```

**New (FIXED for async):**
```rust
pub enum Value {
    String(Arc<String>),              // ✅ Send + Sync
    Array(Arc<RefCell<Vec<Value>>>),  // ✅ Send (RefCell still not Sync, but that's ok)
    HashMap(Arc<RefCell<AtlasHashMap>>), // ✅ Send
    // ... all collections use Arc
}
```

**Why Arc:**
- Send: Can be sent between threads (required for tokio::spawn)
- Sync: Can be shared between threads (Arc<RefCell<T>> is Send)
- Atomic reference counting (thread-safe)
- Minimal performance overhead vs Rc (~10-15% slower, but enables concurrency)

**Trade-offs:**
- ✅ Enables: Async, threading, parallelism, tokio::spawn
- ⚠️ Cost: Slightly slower ref counting (atomic ops)
- ✅ Worth it: Atlas is async-first, needs concurrency

**Decision Reference:** See memory/decisions.md - Runtime/DR-001 (will update after completion)

---

## Files

**Update:** `crates/atlas-runtime/src/value.rs` (~50 changes: Rc→Arc)
**Update:** `crates/atlas-runtime/src/interpreter/mod.rs` (~20 changes)
**Update:** `crates/atlas-runtime/src/vm/mod.rs` (~20 changes)
**Update:** `crates/atlas-runtime/src/stdlib/*.rs` (~200 changes across all stdlib)
**Update:** All test files that construct Values (~100 changes)
**Update:** `memory/decisions.md` (document Runtime/DR-001 update)

---

## Implementation

### GATE -1: Sanity Check ✅

```bash
cargo clean
cargo check -p atlas-runtime
# Must pass before starting
```

---

### GATE 0: Create Tracking Branch

**Create feature branch:**
```bash
git checkout -b foundation/phase-18-value-arc-refactor
```

**Rationale:** Large refactor, want clean commits and easy rollback if needed.

---

### GATE 1: Update Value Enum (Core Change)

**File:** `crates/atlas-runtime/src/value.rs`

**Changes:**
1. Replace ALL `Rc<` with `Arc<` in Value enum
2. Update imports: `use std::sync::Arc;` (remove `use std::rc::Rc;`)
3. Update clone implementations (Arc::clone works same as Rc::clone)

**Example changes:**
```rust
// Before
pub enum Value {
    String(Rc<String>),
    Array(Rc<RefCell<Vec<Value>>>),
    // ...
}

// After
pub enum Value {
    String(Arc<String>),
    Array(Arc<RefCell<Vec<Value>>>),
    // ...
}
```

**Test after each change:**
```bash
cargo check -p atlas-runtime
# Fix compilation errors incrementally
```

**Acceptance:**
- ✅ All Rc→Arc in Value enum
- ✅ Imports updated
- ✅ cargo check passes

---

### GATE 2: Update Interpreter

**File:** `crates/atlas-runtime/src/interpreter/mod.rs`

**Changes:**
1. Update all Value::String(Rc::new(...)) → Value::String(Arc::new(...))
2. Update all Rc::clone → Arc::clone (or just .clone())
3. Update pattern matching if any Rc-specific code exists

**Test:**
```bash
cargo check -p atlas-runtime
```

**Acceptance:**
- ✅ All interpreter Rc→Arc
- ✅ No compilation errors
- ✅ Interpreter logic unchanged (only ref counting changed)

---

### GATE 3: Update VM

**File:** `crates/atlas-runtime/src/vm/mod.rs`

**Changes:**
1. Same as interpreter: Rc::new→Arc::new
2. Update constant pool if it stores Rc<Value>
3. Update stack operations

**Test:**
```bash
cargo check -p atlas-runtime
```

**Acceptance:**
- ✅ All VM Rc→Arc
- ✅ No compilation errors
- ✅ VM logic unchanged

---

### GATE 4: Update All Stdlib Functions

**Files:** `crates/atlas-runtime/src/stdlib/*.rs` (ALL modules)

**Modules to update:**
- stdlib/string.rs
- stdlib/array.rs
- stdlib/collections/hashmap.rs
- stdlib/collections/hashset.rs
- stdlib/collections/queue.rs
- stdlib/collections/stack.rs
- stdlib/json.rs
- stdlib/regex.rs
- stdlib/datetime.rs
- stdlib/http.rs
- stdlib/io.rs
- stdlib/reflection.rs

**Pattern:**
```rust
// Before
Value::String(Rc::new(result))

// After
Value::String(Arc::new(result))
```

**Test each module:**
```bash
cargo check -p atlas-runtime
```

**Acceptance:**
- ✅ All stdlib Rc→Arc
- ✅ All modules compile
- ✅ No Rc references remain

---

### GATE 5: Update Test Files

**Files:** All test files in `crates/atlas-runtime/tests/`

**Changes:**
1. Update test Value construction: Rc→Arc
2. Update assertions if they check ref count (unlikely)

**Test:**
```bash
cargo test -p atlas-runtime --lib
# Run all unit tests
```

**Acceptance:**
- ✅ All tests updated
- ✅ All tests pass
- ✅ No test failures from refactor

---

### GATE 6: Full Test Suite

**Run complete test suite:**
```bash
cargo test -p atlas-runtime
```

**Acceptance:**
- ✅ ALL tests pass
- ✅ Zero failures
- ✅ Zero regressions

**If any failures:**
1. Investigate root cause
2. Fix (likely missed Rc somewhere)
3. Re-run tests
4. Do NOT proceed until 100% pass

---

### GATE 7: Interpreter/VM Parity Verification

**Run parity tests:**
```bash
# Run all integration tests that verify interpreter == VM output
cargo test -p atlas-runtime --test '*' -- --test-threads=1
```

**Verify specific parity:**
```bash
# Test string operations
cargo test -p atlas-runtime test_string -- --exact

# Test array operations
cargo test -p atlas-runtime test_array -- --exact

# Test collections
cargo test -p atlas-runtime test_hashmap -- --exact
cargo test -p atlas-runtime test_hashset -- --exact
```

**Acceptance:**
- ✅ Interpreter output == VM output (100% parity)
- ✅ No behavior changes
- ✅ Only ref counting changed (Rc→Arc)

---

### GATE 8: Verify Send Trait (Goal Achievement)

**Create verification test:**

**File:** `crates/atlas-runtime/tests/value_send_test.rs`

```rust
use atlas_runtime::Value;

#[test]
fn test_value_is_send() {
    // This test MUST compile
    fn assert_send<T: Send>() {}
    assert_send::<Value>();
}

#[test]
fn test_value_can_be_sent_to_thread() {
    use std::thread;

    let value = Value::String(std::sync::Arc::new("test".to_string()));

    // This MUST work now
    let handle = thread::spawn(move || {
        // Use value in different thread
        value
    });

    let result = handle.join().unwrap();
    assert!(matches!(result, Value::String(_)));
}
```

**Test:**
```bash
cargo test -p atlas-runtime value_send_test -- --exact
```

**Acceptance:**
- ✅ Test compiles (proves Value is Send)
- ✅ Test passes (proves can actually send between threads)
- ✅ No compilation errors about Send trait

---

### GATE 9: Update Documentation

**File:** `memory/decisions.md`

**Update Runtime/DR-001:**
```markdown
### Runtime/DR-001: Shared Value Enum with Reference Counting
**Status:** Active | **Date:** 2024-01-18 | **Updated:** 2026-02-16

**Decision:** Single `Value` enum for interpreter and VM.
- `Arc<T>` for thread-safe reference counting (v0.2+)
- `Rc<T>` for single-threaded only (v0.1 - DEPRECATED)
- Strings immutable, arrays mutable (shared by reference)

**Why:** Enable async/concurrency. Arc is Send+Sync, required for tokio::spawn.

**Trade-off:** ~10-15% slower ref counting, but enables parallelism (worth it).

**v0.2 Change:** Refactored from Rc→Arc in Phase-18 to unblock async concurrency.
```

**Acceptance:**
- ✅ Decision log updated
- ✅ Rationale documented
- ✅ Trade-offs explained

---

### GATE 10: Clippy & Format

**Run quality checks:**
```bash
cargo clippy -p atlas-runtime -- -D warnings
cargo fmt -p atlas-runtime
```

**Acceptance:**
- ✅ Zero clippy warnings
- ✅ Code formatted
- ✅ Clean build

---

## Acceptance Criteria

**ALL must be met:**

1. ✅ Value enum uses Arc<T> everywhere (zero Rc remaining)
2. ✅ All stdlib functions updated (Rc→Arc)
3. ✅ Interpreter updated and working
4. ✅ VM updated and working
5. ✅ 100% test pass rate (cargo test -p atlas-runtime)
6. ✅ Interpreter/VM parity maintained (no behavior changes)
7. ✅ Value is Send (test compiles and passes)
8. ✅ Can spawn Value across threads (verification test passes)
9. ✅ Zero clippy warnings
10. ✅ Decision log updated
11. ✅ Zero performance regressions (only ~10-15% slower ref counting expected)

---

## Handoff

**Commit message:**
```
refactor(runtime): Change Value from Rc to Arc for thread safety

BREAKING: Value now uses Arc<T> instead of Rc<T>

**Why:**
- Unblocks async concurrency (tokio::spawn requires Send)
- Enables future parallelism features
- Arc is Send+Sync, Rc is not

**Changes:**
- Value enum: Rc→Arc everywhere
- Interpreter: Updated all Value construction
- VM: Updated all Value construction
- Stdlib: All modules updated (string, array, collections, json, etc.)
- Tests: All tests updated and passing

**Impact:**
- ✅ Enables: Async concurrency, threading, parallelism
- ⚠️ Trade-off: ~10-15% slower ref counting (atomic ops)
- ✅ Worth it: Atlas is async-first language

**Tests:**
- All tests pass (100%)
- Interpreter/VM parity maintained
- Value is Send (verified)
- Can send Value between threads (verified)

**Unblocks:**
- Phase-11c (async primitives)
- All future async/concurrent features

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

**Update STATUS.md:**
- Foundation: 22/24 (92%) → mark phase-18 complete
- Note: "Value refactored to Arc - async/threading enabled"

---

## Notes

**Why this is critical:**
- Phase-11c is blocked waiting for this
- All future async/concurrent features blocked
- Demos using async won't work without this
- This should have been done before async phases started

**Performance impact:**
- Arc uses atomic operations (slower than Rc)
- Expect ~10-15% overhead on ref counting
- NOT a bottleneck (ref counting is tiny fraction of runtime)
- Enables MASSIVE wins via parallelism (worth the trade)

**Thread safety:**
- Arc<T> is Send + Sync
- Arc<RefCell<T>> is Send (not Sync, but that's fine)
- RefCell provides interior mutability (same as before)
- No logic changes, only ref counting changed

**Rollback plan:**
- Feature branch allows easy revert if needed
- All tests must pass before merge
- If catastrophic issues, can revert entire branch
