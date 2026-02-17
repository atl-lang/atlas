# Phase 15: Testing Primitives (Assertions & Test Utilities)

## 🚨 BLOCKERS - CHECK BEFORE STARTING

**REQUIRED:** Basic stdlib functions and Result/Option types.

**Verification:**
```bash
cargo test -p atlas-runtime --lib stdlib
grep -n "Result\|Option" crates/atlas-runtime/src/value.rs
```

**What's needed:**
- Result types from foundation/phase-09
- Module system from foundation/phase-06
- Basic stdlib infrastructure

**If missing:** Complete foundation phases 06 and 09 first

---

## Objective

Implement testing primitives for Atlas - assertion functions and test utilities following the Rust/Go model. This provides the building blocks that the CLI test runner (CLI/phase-02) uses to execute tests.

**Design Philosophy:** Minimal stdlib primitives (like Rust's `std::assert!` or Go's `testing.T`), not a full framework. Test discovery and execution belong in the CLI, not stdlib.

---

## Architecture Decision

**Following: Rust/Go Model**

```
┌──────────────────────────────┐
│ CLI: atlas test (phase-02)   │  ← Test discovery, execution, reporting
│ - Finds test_* functions     │
│ - Runs tests in parallel      │
│ - Formats output             │
└─────────┬────────────────────┘
          │ uses
┌─────────▼────────────────────┐
│ Stdlib: Test primitives      │  ← THIS PHASE
│ - assert(cond, msg)          │
│ - assertEqual(a, b)          │
│ - assertThrows(fn)           │
└──────────────────────────────┘
```

**What this phase does:**
- ✅ Assertion functions
- ✅ Test result types
- ✅ Optional: Basic mocking utilities

**What this phase does NOT do:**
- ❌ Test discovery (CLI/phase-02)
- ❌ Test execution (CLI/phase-02)
- ❌ Test reporting (CLI/phase-02)
- ❌ Parallel execution (CLI/phase-02)
- ❌ CLI integration (CLI/phase-02)

---

## Files

**Create:** `crates/atlas-runtime/src/stdlib/test.rs` (~400 lines)
**Update:** `crates/atlas-runtime/src/stdlib/mod.rs` (~20 lines - register functions)
**Tests:** `crates/atlas-runtime/tests/test_primitives.rs` (~300 lines)

**Total: ~700 lines** (not 2900 like original phase)

---

## Dependencies

- Result/Option types (foundation/phase-09)
- Module system (foundation/phase-06)
- Basic stdlib infrastructure
- String formatting for error messages

---

## Implementation

### GATE -1: Sanity Check ✅

```bash
cargo clean
cargo check -p atlas-runtime
# Must pass before starting
```

---

### GATE 0: Design Assertion API

**Atlas assertion API (inspired by Rust/Go):**

```atlas
// Basic assertions
fn assert(condition: bool, message: string) -> void;
fn assertFalse(condition: bool, message: string) -> void;

// Equality assertions
fn assertEqual<T>(actual: T, expected: T) -> void;
fn assertNotEqual<T>(actual: T, expected: T) -> void;

// Result assertions
fn assertOk<T, E>(result: Result<T, E>) -> T;
fn assertErr<T, E>(result: Result<T, E>) -> E;

// Option assertions
fn assertSome<T>(option: Option<T>) -> T;
fn assertNone<T>(option: Option<T>) -> void;

// Collection assertions
fn assertContains<T>(array: array, value: T) -> void;
fn assertEmpty(array: array) -> void;
fn assertLength(array: array, expected: number) -> void;

// Error assertions
fn assertThrows(fn: Function) -> void;
fn assertNoThrow(fn: Function) -> void;
```

**Design principles:**
- Clear names (assertEqual not eq)
- Explicit types (assertOk for Result)
- Helpful error messages (show diff on failure)
- Simple API (no complex matchers)

**Acceptance:**
- ✅ API designed
- ✅ Follows Rust/Go conventions
- ✅ Clear, simple, complete

---

### GATE 1: Implement Core Assertions

**File:** `crates/atlas-runtime/src/stdlib/test.rs`

**Implement:**

```rust
use crate::value::Value;
use crate::error::{RuntimeError, ErrorCode};
use crate::span::Span;

/// assert(condition, message) - Basic assertion
pub fn assert(args: &[Value], call_span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::new(
            ErrorCode::AT0103,
            format!("assert expects 2 arguments, got {}", args.len()),
        ).with_span(call_span));
    }

    let condition = expect_bool(&args[0], "assert", 0, call_span)?;
    let message = expect_string(&args[1], "assert", 1, call_span)?;

    if !condition {
        return Err(RuntimeError::new(
            ErrorCode::AT9001, // Assertion failure
            format!("Assertion failed: {}", message),
        ).with_span(call_span));
    }

    Ok(Value::Null)
}

/// assertEqual(actual, expected) - Equality assertion with diff
pub fn assert_equal(args: &[Value], call_span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::new(
            ErrorCode::AT0103,
            format!("assertEqual expects 2 arguments, got {}", args.len()),
        ).with_span(call_span));
    }

    let actual = &args[0];
    let expected = &args[1];

    if !values_equal(actual, expected) {
        return Err(RuntimeError::new(
            ErrorCode::AT9001,
            format!(
                "Assertion failed: values not equal\n  Actual:   {:?}\n  Expected: {:?}",
                actual, expected
            ),
        ).with_span(call_span));
    }

    Ok(Value::Null)
}

// Helper: Deep equality check
fn values_equal(a: &Value, b: &Value) -> bool {
    // Implement deep equality (handle arrays, nested values, etc.)
    // Use existing Value equality logic
    a == b
}
```

**Test:**
```bash
cargo test -p atlas-runtime test_assert -- --exact
cargo test -p atlas-runtime test_assert_equal -- --exact
```

**Acceptance:**
- ✅ assert() works
- ✅ assertEqual() works
- ✅ Error messages are clear
- ✅ Tests pass

---

### GATE 2: Implement Result Assertions

**Add to test.rs:**

```rust
/// assertOk(result) - Assert Result is Ok, return value
pub fn assert_ok(args: &[Value], call_span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            ErrorCode::AT0103,
            format!("assertOk expects 1 argument, got {}", args.len()),
        ).with_span(call_span));
    }

    // Check if value is Result
    match &args[0] {
        Value::Result(result) => {
            if result.is_ok {
                // Return the Ok value
                Ok(result.value.as_ref().clone())
            } else {
                Err(RuntimeError::new(
                    ErrorCode::AT9001,
                    format!("Expected Ok, got Err: {:?}", result.value),
                ).with_span(call_span))
            }
        }
        _ => Err(RuntimeError::new(
            ErrorCode::AT0140,
            "assertOk expects Result type",
        ).with_span(call_span))
    }
}

/// assertErr(result) - Assert Result is Err, return error value
pub fn assert_err(args: &[Value], call_span: Span) -> Result<Value, RuntimeError> {
    // Similar to assertOk but checks for Err
    // Returns the error value if Result is Err
    // ...implementation...
}
```

**Test:**
```bash
cargo test -p atlas-runtime test_assert_ok -- --exact
cargo test -p atlas-runtime test_assert_err -- --exact
```

**Acceptance:**
- ✅ assertOk() works with Result types
- ✅ assertErr() works with Result types
- ✅ Returns unwrapped values
- ✅ Clear error messages

---

### GATE 3: Implement Option Assertions

**Add assertSome and assertNone following same pattern as Result assertions.**

**Test:**
```bash
cargo test -p atlas-runtime test_assert_some -- --exact
cargo test -p atlas-runtime test_assert_none -- --exact
```

**Acceptance:**
- ✅ assertSome() works
- ✅ assertNone() works
- ✅ Returns unwrapped values
- ✅ Tests pass

---

### GATE 4: Implement Collection Assertions

**Add:**
- assertContains(array, value)
- assertEmpty(array)
- assertLength(array, expected)

**Test:**
```bash
cargo test -p atlas-runtime test_collection_assertions -- --exact
```

**Acceptance:**
- ✅ Collection assertions work
- ✅ Handle arrays correctly
- ✅ Clear error messages

---

### GATE 5: Implement Error Assertions

**Add:**

```rust
/// assertThrows(fn) - Assert function throws error
pub fn assert_throws(args: &[Value], call_span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            ErrorCode::AT0103,
            format!("assertThrows expects 1 argument, got {}", args.len()),
        ).with_span(call_span));
    }

    // Execute function and expect it to throw
    match &args[0] {
        Value::Function(_) => {
            // Call the function
            // If it succeeds -> assertion fails
            // If it throws -> assertion passes
            // ...implementation needs runtime context...
        }
        _ => Err(RuntimeError::new(
            ErrorCode::AT0140,
            "assertThrows expects function",
        ).with_span(call_span))
    }
}
```

**Note:** assertThrows may need interpreter/runtime context to execute functions.

**Test:**
```bash
cargo test -p atlas-runtime test_assert_throws -- --exact
```

**Acceptance:**
- ✅ assertThrows() works
- ✅ Detects when function throws
- ✅ Fails when function doesn't throw

---

### GATE 6: Register All Functions in Stdlib

**File:** `crates/atlas-runtime/src/stdlib/mod.rs`

**Add to is_builtin():**
```rust
// Test assertions
"assert" | "assertFalse" |
"assertEqual" | "assertNotEqual" |
"assertOk" | "assertErr" |
"assertSome" | "assertNone" |
"assertContains" | "assertEmpty" | "assertLength" |
"assertThrows" | "assertNoThrow" => true,
```

**Add to call_builtin():**
```rust
// Test assertions
"assert" => test::assert(args, call_span),
"assertFalse" => test::assert_false(args, call_span),
"assertEqual" => test::assert_equal(args, call_span),
// ... etc for all assertions
```

**Test:**
```bash
cargo test -p atlas-runtime test_builtin_registration -- --exact
```

**Acceptance:**
- ✅ All assertions registered
- ✅ Callable from Atlas code
- ✅ No registration conflicts

---

### GATE 7: Create Integration Tests

**File:** `crates/atlas-runtime/tests/test_primitives.rs`

**Test actual Atlas code using assertions:**

```rust
use atlas_runtime::Atlas;

#[test]
fn test_assert_in_atlas_code() {
    let source = r#"
        fn test_basic() -> void {
            assert(true, "should pass");
            assertEqual(5, 5);
            assertEqual("hello", "hello");
        }
        test_basic();
    "#;

    let mut runtime = Atlas::new();
    let result = runtime.eval(source);
    assert!(result.is_ok(), "Assertions should pass");
}

#[test]
fn test_assert_failure() {
    let source = r#"
        fn test_fail() -> void {
            assert(false, "this should fail");
        }
        test_fail();
    "#;

    let mut runtime = Atlas::new();
    let result = runtime.eval(source);
    assert!(result.is_err(), "Should fail assertion");

    let err = result.unwrap_err();
    assert!(err.to_string().contains("Assertion failed"));
}

#[test]
fn test_assert_equal_failure_shows_diff() {
    let source = r#"
        assertEqual(5, 10);
    "#;

    let mut runtime = Atlas::new();
    let result = runtime.eval(source);
    assert!(result.is_err());

    let err = result.unwrap_err().to_string();
    assert!(err.contains("Actual:"));
    assert!(err.contains("Expected:"));
}

#[test]
fn test_assert_ok_with_result() {
    let source = r#"
        fn divide(a: number, b: number) -> Result<number, string> {
            if (b == 0) { return Err("division by zero"); }
            return Ok(a / b);
        }

        let result = divide(10, 2);
        let value = assertOk(result);
        assertEqual(value, 5);
    "#;

    let mut runtime = Atlas::new();
    let result = runtime.eval(source);
    assert!(result.is_ok());
}
```

**Test:**
```bash
cargo test -p atlas-runtime test_primitives
```

**Acceptance:**
- ✅ All integration tests pass
- ✅ Assertions work in real Atlas code
- ✅ Error messages helpful
- ✅ Result/Option integration works

---

### GATE 8: Interpreter/VM Parity

**Test assertions work in both engines:**

```bash
# Test with interpreter
cargo test -p atlas-runtime test_primitives -- --exact

# Test with VM (if separate)
cargo test -p atlas-runtime test_primitives_vm -- --exact
```

**Acceptance:**
- ✅ Assertions work in interpreter
- ✅ Assertions work in VM
- ✅ Same behavior in both
- ✅ 100% parity maintained

---

### GATE 9: Documentation

**Update:** `docs/specification/stdlib.md`

**Add section:**

```markdown
## Testing Primitives

Atlas provides built-in assertion functions for writing tests. These are used by the `atlas test` CLI command (see CLI documentation).

### Assertions

**Basic:**
- `assert(condition: bool, message: string) -> void` - Assert condition is true
- `assertFalse(condition: bool, message: string) -> void` - Assert condition is false

**Equality:**
- `assertEqual<T>(actual: T, expected: T) -> void` - Assert values are equal
- `assertNotEqual<T>(actual: T, expected: T) -> void` - Assert values are not equal

**Result:**
- `assertOk<T, E>(result: Result<T, E>) -> T` - Assert Result is Ok, return value
- `assertErr<T, E>(result: Result<T, E>) -> E` - Assert Result is Err, return error

**Option:**
- `assertSome<T>(option: Option<T>) -> T` - Assert Option is Some, return value
- `assertNone<T>(option: Option<T>) -> void` - Assert Option is None

**Collections:**
- `assertContains<T>(array: array, value: T) -> void` - Assert array contains value
- `assertEmpty(array: array) -> void` - Assert array is empty
- `assertLength(array: array, expected: number) -> void` - Assert array length

**Errors:**
- `assertThrows(fn: Function) -> void` - Assert function throws error
- `assertNoThrow(fn: Function) -> void` - Assert function doesn't throw

### Example

```atlas
fn test_divide() -> void {
    let result = divide(10, 2);
    let value = assertOk(result);
    assertEqual(value, 5);
}
```

Run tests with: `atlas test`
```

**Acceptance:**
- ✅ API documented
- ✅ Examples provided
- ✅ Clear usage instructions

---

### GATE 10: Clippy & Format

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

1. ✅ Core assertions implemented (assert, assertEqual, etc.)
2. ✅ Result assertions work (assertOk, assertErr)
3. ✅ Option assertions work (assertSome, assertNone)
4. ✅ Collection assertions work
5. ✅ Error assertions work (assertThrows)
6. ✅ All functions registered in stdlib
7. ✅ Integration tests pass
8. ✅ Interpreter/VM parity maintained
9. ✅ Documentation complete
10. ✅ Zero clippy warnings
11. ✅ NO test discovery (that's CLI/phase-02)
12. ✅ NO test execution (that's CLI/phase-02)
13. ✅ NO CLI updates (that's CLI/phase-02)

---

## Handoff

**Commit message:**
```
feat(stdlib): Add testing primitives (assertions) - phase-15

Following Rust/Go model: minimal stdlib assertions, not full framework.

**What this provides:**
- Assertion functions (assert, assertEqual, assertOk, etc.)
- Test utilities for Result/Option types
- Collection assertions
- Error assertions

**What this does NOT provide:**
- Test discovery (CLI/phase-02)
- Test execution (CLI/phase-02)
- Test reporting (CLI/phase-02)
- CLI integration (CLI/phase-02)

**Architecture:**
- Stdlib: Provides assertion primitives
- CLI: Uses these primitives for test runner

**API:**
- assert(condition, msg)
- assertEqual(actual, expected)
- assertOk(result), assertErr(result)
- assertSome(option), assertNone(option)
- assertContains(array, value)
- assertThrows(fn)

**Tests:**
- Integration tests pass
- Interpreter/VM parity maintained
- Real Atlas code can use assertions

**Next:** CLI/phase-02 (test runner using these primitives)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

**Update STATUS.md:**
- Stdlib: Mark phase-15 complete
- Note: "Testing primitives (Rust/Go model)"

---

## Notes

**Why this approach:**
- Follows Rust (`std::assert!`) and Go (`testing.T`) model
- Clean separation: stdlib = primitives, CLI = orchestration
- No duplication with CLI/phase-02
- Simple, focused, world-class standard

**What CLI/phase-02 will do:**
- Test discovery (find test_* functions)
- Test execution (run discovered tests)
- Parallel execution
- Filtering and reporting
- Use these assertion primitives

**Total code:** ~700 lines (not 2900 like original phase)

**Time estimate:** 4-6 hours (much simpler than original mega-phase)
