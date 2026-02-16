# Phase 18e: Arc Refactor - Advanced Stdlib

## 🎯 Scope: Advanced Stdlib Only

**What THIS phase does:** Update advanced stdlib (datetime, http, regex, reflection) to use Arc
**Depends on:** Phases 18a, 18b, 18c, 18d complete
**Estimated time:** 2-3 hours

---

## 🚨 DEPENDENCIES

**REQUIRED:** Phases 18a, 18b, 18c, 18d complete
**BLOCKS:** Phase 18f (tests & verification)

**Verify dependencies:**
```bash
cargo check -p atlas-runtime --lib 2>&1 | grep collections
# Should show zero errors in collections
```

---

## Objective

Update the 4 advanced stdlib modules to use Arc. These are the complex, feature-rich modules (HTTP client, DateTime, Regex, Reflection).

**Modules in scope:**
- stdlib/datetime.rs
- stdlib/http.rs
- stdlib/regex.rs
- stdlib/reflection.rs

**This is the LAST stdlib phase** - after this, only tests remain (18f).

---

## Files

**Update:** `crates/atlas-runtime/src/stdlib/datetime.rs` (~30 changes)
**Update:** `crates/atlas-runtime/src/stdlib/http.rs` (~40 changes)
**Update:** `crates/atlas-runtime/src/stdlib/regex.rs` (~20 changes)
**Update:** `crates/atlas-runtime/src/stdlib/reflection.rs` (~15 changes)

---

## Implementation

### GATE -1: Sanity Check ✅

```bash
cargo clean
cargo check -p atlas-runtime --lib
# Will fail (these modules still use Rc) - expected
```

---

### GATE 0: Count Changes Needed

**See scope:**
```bash
grep -n "Rc::new\|Rc::clone" crates/atlas-runtime/src/stdlib/datetime.rs | wc -l
grep -n "Rc::new\|Rc::clone" crates/atlas-runtime/src/stdlib/http.rs | wc -l
grep -n "Rc::new\|Rc::clone" crates/atlas-runtime/src/stdlib/regex.rs | wc -l
grep -n "Rc::new\|Rc::clone" crates/atlas-runtime/src/stdlib/reflection.rs | wc -l
```

**Acceptance:**
- ✅ Know the scope (~105 total changes)
- ✅ HTTP is the heaviest user

---

### GATE 1: Update datetime.rs

**File:** `crates/atlas-runtime/src/stdlib/datetime.rs`

**Change ALL:**
- `Value::String(Rc::new(...))` → `Value::String(Arc::new(...))`
- DateTime functions return lots of strings (formatted dates)
- Duration functions may create arrays

**Common patterns:**
```rust
// Before
Ok(Value::String(Rc::new(dt.format("%Y-%m-%d").to_string())))

// After
Ok(Value::String(Arc::new(dt.format("%Y-%m-%d").to_string())))
```

**Test THIS MODULE:**
```bash
cargo check -p atlas-runtime --lib 2>&1 | grep datetime.rs
```

**Acceptance:**
- ✅ All Rc::new → Arc::new in datetime.rs
- ✅ Zero errors in datetime.rs

---

### GATE 2: Update http.rs

**File:** `crates/atlas-runtime/src/stdlib/http.rs`

**Change ALL:**
- `Value::String(Rc::new(...))` → `Value::String(Arc::new(...))`
- `Value::HashMap(Rc::new(RefCell::new(...)))` → `Value::HashMap(Arc::new(RefCell::new(...)))`
- HTTP responses have headers (HashMap), body (String), status (Number)
- This is the most complex module

**Common patterns:**
```rust
// Before
let headers = Value::HashMap(Rc::new(RefCell::new(headers_map)));

// After
let headers = Value::HashMap(Arc::new(RefCell::new(headers_map)));

// Before
Ok(Value::String(Rc::new(body)))

// After
Ok(Value::String(Arc::new(body)))
```

**Test THIS MODULE:**
```bash
cargo check -p atlas-runtime --lib 2>&1 | grep http.rs
```

**Acceptance:**
- ✅ All Rc::new → Arc::new in http.rs
- ✅ Zero errors in http.rs

---

### GATE 3: Update regex.rs

**File:** `crates/atlas-runtime/src/stdlib/regex.rs`

**Change ALL:**
- `Value::String(Rc::new(...))` → `Value::String(Arc::new(...))`
- `Value::Array(Rc::new(RefCell::new(...)))` → `Value::Array(Arc::new(RefCell::new(...)))`
- Regex match results return arrays of strings

**Test THIS MODULE:**
```bash
cargo check -p atlas-runtime --lib 2>&1 | grep regex.rs
```

**Acceptance:**
- ✅ All Rc::new → Arc::new in regex.rs
- ✅ Zero errors in regex.rs

---

### GATE 4: Update reflection.rs

**File:** `crates/atlas-runtime/src/stdlib/reflection.rs`

**Change ALL:**
- `Value::String(Rc::new(...))` → `Value::String(Arc::new(...))`
- Type names, property lists return strings/arrays

**Test THIS MODULE:**
```bash
cargo check -p atlas-runtime --lib 2>&1 | grep reflection.rs
```

**Acceptance:**
- ✅ All Rc::new → Arc::new in reflection.rs
- ✅ Zero errors in reflection.rs

---

### GATE 5: Verify No Rc Remains in Advanced Stdlib

**Search all 4 modules:**
```bash
grep "Rc::" crates/atlas-runtime/src/stdlib/datetime.rs
grep "Rc::" crates/atlas-runtime/src/stdlib/http.rs
grep "Rc::" crates/atlas-runtime/src/stdlib/regex.rs
grep "Rc::" crates/atlas-runtime/src/stdlib/reflection.rs
```

**Acceptance:**
- ✅ Zero matches (no Rc left)
- ✅ All 4 modules use Arc only

---

### GATE 6: Compile Check - ALL Stdlib

**Test entire stdlib:**
```bash
cargo check -p atlas-runtime --lib
```

**Acceptance:**
- ✅ ALL stdlib modules compile
- ✅ Zero Rc-related errors
- ✅ Test errors are fine (tests not updated yet)

---

### GATE 7: Run Advanced Stdlib Tests

**Test these modules:**
```bash
cargo test -p atlas-runtime test_datetime -- --exact
cargo test -p atlas-runtime test_http -- --exact
cargo test -p atlas-runtime test_regex -- --exact
```

**Acceptance:**
- ✅ DateTime tests pass
- ✅ HTTP tests pass
- ✅ Regex tests pass
- ✅ No regressions from Arc change

---

### GATE 8: Clippy & Format

```bash
cargo clippy -p atlas-runtime --lib -- -D warnings 2>&1 | grep -E "(datetime|http|regex|reflection).rs"
cargo fmt -- crates/atlas-runtime/src/stdlib/*.rs
```

**Acceptance:**
- ✅ Zero clippy warnings in these modules
- ✅ Code formatted

---

## Acceptance Criteria

**ALL must be met:**

1. ✅ datetime.rs uses Arc::new (zero Rc::new)
2. ✅ http.rs uses Arc::new (zero Rc::new)
3. ✅ regex.rs uses Arc::new (zero Rc::new)
4. ✅ reflection.rs uses Arc::new (zero Rc::new)
5. ✅ All 4 modules compile
6. ✅ ALL stdlib modules now compile (18c + 18d + 18e = complete)
7. ✅ DateTime/HTTP/Regex tests pass
8. ✅ Zero clippy warnings in these modules
9. ✅ Code formatted

**DO NOT:**
- ❌ Touch all test files yet (that's phase-18f)
- ❌ Run full test suite yet (will fail until test files updated)

---

## Handoff

**Commit message:**
```
refactor(stdlib): Update advanced stdlib to use Arc (phase 18e)

Part 5/6 of Arc refactor for thread safety.

**Changes:**
- datetime.rs: All Rc → Arc (~30 changes)
- http.rs: All Rc → Arc (~40 changes)
- regex.rs: All Rc → Arc (~20 changes)
- reflection.rs: All Rc → Arc (~15 changes)

**Tests:**
- DateTime tests: ✅
- HTTP tests: ✅
- Regex tests: ✅

**Scope:** 4 advanced stdlib modules (~105 changes)

**Status:** ALL stdlib now uses Arc (18c + 18d + 18e complete)

**Next:** Phase-18f (update test files & verification)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

**Update STATUS.md:**
- Foundation: Mark "Phase-18e: Arc Refactor - Advanced Stdlib (Complete)"
- Note: "5/6 sub-phases complete - ALL stdlib updated"

---

## Notes

**Why advanced stdlib separate?**
- Different complexity level (HTTP is complex)
- Final stdlib phase (clean separation)
- Can verify all stdlib compiles after this
- ~105 changes (manageable as one phase)

**What's left:**
- Test files still use Rc in test code
- Full test suite will fail
- **Phase 18f** is the final cleanup

**Time estimate:** 2-3 hours (~105 changes in complex modules)
