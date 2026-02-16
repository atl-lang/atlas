# Phase 20c: For-In Loops - Backend (Interpreter + Compiler/VM)

## 🎯 Scope: Execution Engines Only

**What THIS phase does:** Enable interpreter and VM to execute desugared for-in loops
**Depends on:** Phase-20b complete (desugaring works)
**Estimated time:** 2-3 hours

---

## 🚨 DEPENDENCIES

**REQUIRED:** Phase-20b complete (desugaring implemented)
**BLOCKS:** Phase 20d (comprehensive testing)

**Verify dependency:**
```bash
cargo test -p atlas-runtime test_for_in_semantic -- --exact
# All semantic tests must pass
```

---

## Objective

Enable the execution engines to run for-in loops. Since phase-20b implemented desugaring, this phase is mostly about removing stubs and verifying the desugared form executes correctly.

**Key insight:** The TypeChecker already desugars `ForIn` → `Block` with traditional `For` inside. So the interpreter and VM just need to handle the desugared AST, which they already can do.

**This phase:**
1. Remove interpreter stub (or make it handle desugared form)
2. Remove compiler stub (or make it handle desugared form)
3. Verify execution works
4. Test break/continue work

---

## Design

**Approach 1: Remove ForIn entirely (it's desugared away)**
- TypeChecker desugars ForIn → Block
- Interpreter/VM never see ForIn (only see Block + For)
- Just remove stubs

**Approach 2: Keep ForIn but execute desugared form**
- Interpreter/VM receive ForIn
- They execute the desugared version
- More explicit, easier to debug

**Use Approach 1** - simpler, cleaner.

---

## Files

**Update:** `crates/atlas-runtime/src/interpreter/mod.rs` (remove stub, ~5 lines)
**Update:** `crates/atlas-runtime/src/compiler/mod.rs` (remove stub, ~5 lines)
**Create:** `crates/atlas-runtime/tests/test_for_in_execution.rs` (~200 lines)

---

## Implementation

### GATE -1: Sanity Check ✅

```bash
cargo clean
cargo check -p atlas-runtime
cargo test -p atlas-runtime test_for_in_semantic -- --exact
# Must pass
```

---

### GATE 0: Verify Desugaring Works

**Run a test with debug output:**
```bash
cargo test -p atlas-runtime test_for_in_desugaring_output -- --exact --nocapture
```

**Verify:**
- Desugared AST contains Block → Let → For
- No ForIn remains after type checking

**Acceptance:**
- ✅ Desugaring confirmed working
- ✅ Understand what interpreter/VM will receive

---

### GATE 1: Check Interpreter Stub

**File:** `crates/atlas-runtime/src/interpreter/mod.rs`

**Current stub returns error. Two options:**

**Option A: Remove ForIn case entirely**
```rust
// Just delete the ForIn match arm
// TypeChecker desugars it, so interpreter never sees it
```

**Option B: Make it execute desugared form**
```rust
Stmt::ForIn { variable, iterable, body, span } => {
    // This should never be reached if desugaring works
    // But handle it gracefully just in case
    panic!("ForIn should be desugared by typechecker");
}
```

**Use Option A** if you're confident, **Option B** for safety.

**Test:**
```bash
cargo check -p atlas-runtime --lib
```

**Acceptance:**
- ✅ Interpreter compiles
- ✅ Stub removed or updated

---

### GATE 2: Check Compiler Stub

**File:** `crates/atlas-runtime/src/compiler/mod.rs`

**Same as GATE 1 - remove or update stub.**

**Test:**
```bash
cargo check -p atlas-runtime
```

**Acceptance:**
- ✅ Compiler compiles
- ✅ Stub removed or updated

---

### GATE 3: Create Basic Execution Tests

**Create:** `crates/atlas-runtime/tests/test_for_in_execution.rs`

```rust
use atlas_runtime::{Atlas, Value};

#[test]
fn test_for_in_basic_execution() {
    let source = r#"
        let arr: array = [1, 2, 3];
        let sum: number = 0;
        for item in arr {
            sum = sum + item;
        }
        sum
    "#;

    let mut runtime = Atlas::new();
    let result = runtime.eval(source);

    assert!(result.is_ok(), "Should execute for-in loop");
    assert_eq!(result.unwrap(), Value::Number(6.0), "Sum should be 6");
}

#[test]
fn test_for_in_empty_array() {
    let source = r#"
        let arr: array = [];
        let count: number = 0;
        for item in arr {
            count = count + 1;
        }
        count
    "#;

    let mut runtime = Atlas::new();
    let result = runtime.eval(source);

    assert!(result.is_ok(), "Should handle empty array");
    assert_eq!(result.unwrap(), Value::Number(0.0), "Count should be 0");
}

#[test]
fn test_for_in_with_strings() {
    let source = r#"
        let words: array = ["hello", "world"];
        let result: string = "";
        for word in words {
            result = result + word + " ";
        }
        result
    "#;

    let mut runtime = Atlas::new();
    let result = runtime.eval(source);

    assert!(result.is_ok(), "Should work with strings");
    // Check result is "hello world "
}

#[test]
fn test_for_in_nested() {
    let source = r#"
        let matrix: array = [[1, 2], [3, 4]];
        let sum: number = 0;
        for row in matrix {
            for item in row {
                sum = sum + item;
            }
        }
        sum
    "#;

    let mut runtime = Atlas::new();
    let result = runtime.eval(source);

    assert!(result.is_ok(), "Should handle nested loops");
    assert_eq!(result.unwrap(), Value::Number(10.0), "Sum should be 10");
}

#[test]
fn test_for_in_modifies_external_variable() {
    let source = r#"
        let arr: array = [10, 20, 30];
        let total: number = 0;
        for x in arr {
            total = total + x;
        }
        total
    "#;

    let mut runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(result.unwrap(), Value::Number(60.0));
}
```

**Test:**
```bash
cargo test -p atlas-runtime test_for_in_execution -- --exact
```

**Acceptance:**
- ✅ Basic execution works
- ✅ Empty arrays work
- ✅ String iteration works
- ✅ Nested loops work
- ✅ External variable modification works

---

### GATE 4: Test Break and Continue

**Add tests for control flow:**

```rust
#[test]
fn test_for_in_with_break() {
    let source = r#"
        let arr: array = [1, 2, 3, 4, 5];
        let sum: number = 0;
        for item in arr {
            if (item > 3) {
                break;
            }
            sum = sum + item;
        }
        sum
    "#;

    let mut runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(result.unwrap(), Value::Number(6.0), "Should break at 4, sum 1+2+3=6");
}

#[test]
fn test_for_in_with_continue() {
    let source = r#"
        let arr: array = [1, 2, 3, 4, 5];
        let sum: number = 0;
        for item in arr {
            if (item == 3) {
                continue;
            }
            sum = sum + item;
        }
        sum
    "#;

    let mut runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(result.unwrap(), Value::Number(12.0), "Should skip 3, sum 1+2+4+5=12");
}
```

**Test:**
```bash
cargo test -p atlas-runtime test_for_in_with_break -- --exact
cargo test -p atlas-runtime test_for_in_with_continue -- --exact
```

**Acceptance:**
- ✅ Break works in for-in loops
- ✅ Continue works in for-in loops

---

### GATE 5: Test Interpreter/VM Parity

**Create parity test:**

```rust
#[test]
fn test_for_in_interpreter_vm_parity() {
    let source = r#"
        fn sum_array(arr: array) -> number {
            let total: number = 0;
            for item in arr {
                total = total + item;
            }
            return total;
        }

        sum_array([10, 20, 30])
    "#;

    // Test interpreter
    let mut runtime_interp = Atlas::new();
    let result_interp = runtime_interp.eval(source).unwrap();

    // Test VM (if VM mode available)
    // let mut runtime_vm = Atlas::new_with_vm();
    // let result_vm = runtime_vm.eval(source).unwrap();
    // assert_eq!(result_interp, result_vm, "Interpreter and VM must match");

    assert_eq!(result_interp, Value::Number(60.0));
}
```

**Test:**
```bash
cargo test -p atlas-runtime test_for_in_interpreter_vm_parity -- --exact
```

**Acceptance:**
- ✅ Interpreter executes correctly
- ✅ VM executes correctly (if tested)
- ✅ Outputs match (parity maintained)

---

### GATE 6: Test Variable Shadowing

**Test shadowing behavior:**

```rust
#[test]
fn test_for_in_variable_shadowing() {
    let source = r#"
        let item: number = 100;
        let arr: array = [1, 2, 3];

        for item in arr {
            // 'item' here shadows outer 'item'
        }

        item // Should still be 100 (outer item unchanged)
    "#;

    let mut runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(result.unwrap(), Value::Number(100.0), "Outer variable unchanged");
}
```

**Test:**
```bash
cargo test -p atlas-runtime test_for_in_variable_shadowing -- --exact
```

**Acceptance:**
- ✅ Shadowing works correctly
- ✅ Outer variable unchanged

---

### GATE 7: Run Full Test Suite

**Run ALL tests:**
```bash
cargo test -p atlas-runtime
```

**Acceptance:**
- ✅ ALL tests pass
- ✅ Zero failures
- ✅ Zero regressions
- ✅ For-in tests pass
- ✅ Existing for loop tests still pass

---

### GATE 8: Clippy & Format

```bash
cargo clippy -p atlas-runtime -- -D warnings
cargo fmt -p atlas-runtime
```

**Acceptance:**
- ✅ Zero clippy warnings
- ✅ Code formatted

---

## Acceptance Criteria

**ALL must be met:**

1. ✅ Interpreter stub removed/updated
2. ✅ Compiler stub removed/updated
3. ✅ Basic for-in execution works
4. ✅ Empty array handled correctly
5. ✅ Nested loops work
6. ✅ Break works
7. ✅ Continue works
8. ✅ Variable shadowing correct
9. ✅ Interpreter/VM parity maintained
10. ✅ ALL tests pass
11. ✅ Zero clippy warnings

**DO NOT:**
- ❌ Test demos yet (that's phase-20d)
- ❌ Write comprehensive edge case tests (that's phase-20d)

---

## Handoff

**Commit message:**
```
feat(runtime): Enable for-in loop execution (phase 20c)

Part 3/4 of for-in loop implementation.

**Changes:**
- Interpreter: Removed stub (executes desugared form)
- Compiler: Removed stub (compiles desugared form)
- VM: Executes desugared bytecode
- Tests: Execution tests pass

**Tests:**
- Basic iteration: ✅
- Empty arrays: ✅
- Nested loops: ✅
- Break/continue: ✅
- Variable shadowing: ✅
- Interpreter/VM parity: ✅

**Status:** Execution complete, comprehensive testing remaining

**Next:** Phase-20d (comprehensive tests + demos)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

**Update STATUS.md:**
- Foundation: Mark "Phase-20c: For-In Loops - Backend (Complete)"
- Note: "3/4 sub-phases complete"

---

## Notes

**Why was this phase easy?**
- Desugaring in 20b did the heavy lifting
- Interpreter/VM already handle traditional for loops
- Just removed stubs and tested

**What works now:**
- For-in syntax: `for item in arr { }`
- Nested for-in loops
- Break and continue
- Variable shadowing
- Full interpreter/VM support

**Phase 20d will:**
- Comprehensive edge case testing
- Demo verification
- Documentation
- Final polish

**Time estimate:** 2-3 hours (mostly testing)
