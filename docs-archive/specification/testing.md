# Atlas Testing System

**Version:** 0.3
**Status:** Complete
**Date:** 2026-03-04

This document specifies the Atlas testing system, including test discovery, file naming conventions, and assertion primitives provided by the standard library.

---

## Table of Contents

1. [Overview](#overview)
2. [Test File Convention](#test-file-convention)
3. [Test Discovery](#test-discovery)
4. [Running Tests](#running-tests)
5. [Assertion Primitives](#assertion-primitives)
6. [Test Examples](#test-examples)

---

## Overview

Atlas follows the **Rust/Go testing model:**

- **Stdlib provides assertion primitives** — 13 functions for various assertion types
- **CLI handles discovery, execution, and reporting** — `atlas test` command
- **No test frameworks** — assertions are simple functions in stdlib

This design keeps the language minimal while providing powerful testing capabilities.

### Philosophy

- Testing is first-class: test functions are regular Atlas code
- Tests use standard assertion functions from stdlib
- Discovery is automatic based on file naming and function naming
- Execution is parallel by default (sequential with `--sequential` flag)

---

## Test File Convention

### Naming

- Test files must have `.test.atl` extension
- Example: `math.test.atl`, `utils.test.atl`, `integration.test.atl`

### Structure

Test files contain regular Atlas code. Test functions are identified by their names:

```atlas
// Single test function (discovered automatically)
fn test_addition() -> void {
    assertEqual(2 + 2, 4);
}

// Multiple test functions in one file
fn test_subtraction() -> void {
    assertEqual(5 - 3, 2);
}

fn test_multiplication() -> void {
    assertEqual(3 * 4, 12);
}
```

### Discovery Rules

1. **Files:** Only `*.test.atl` files are considered
2. **Functions:** Functions matching `test_*` pattern are discovered
3. **Signature:** Test functions must have **no parameters**. Return values are ignored.
4. **Recursive:** Test discovery is recursive through project directories

### Valid Test Functions

```atlas
// Correct: no parameters, void return
fn test_basic() -> void {
    assert(true, "always true");
}

// Correct: return type can be omitted (inferred as void)
fn test_inferred_return() {
    assertEqual(1, 1);
}

// Also OK: return value is ignored by the test runner
fn test_returns_number() -> number {
    assert(true, "still a valid test");
    return 42;
}
```

### Invalid Test Functions

```atlas
// Wrong: takes parameters
fn test_with_args(x: number) -> void {
    assertEqual(x, 5);
}

// Not discovered: doesn't match pattern
fn helper_function() -> void {
    print("helper");
}
```

---

## Test Discovery

### Discovery Process

1. **Scan phase:** Walk project directory recursively
2. **Filter phase:** Find all `*.test.atl` files
3. **Parse phase:** Parse test files and extract test functions
4. **Group phase:** Organize tests by file for reporting

### Example Discovery

Given this file structure:

```
project/
├── main.atl
├── math.test.atl       ✅ Discovered
├── utils/
│   ├── helpers.atl
│   └── string.test.atl ✅ Discovered (recursive)
└── integration.test.atl ✅ Discovered
```

The following tests are discovered:

- `math.test.atl::test_add`
- `math.test.atl::test_subtract`
- `utils/string.test.atl::test_trim`
- `integration.test.atl::test_full_workflow`

### Test Names

Test names are constructed from file path and function name:

```
<file>::<function>
```

Example: `tests/unit/math.test.atl::test_fibonacci`

---

## Running Tests

### CLI Command

```bash
# Run all tests
atlas test

# Run tests matching pattern (positional argument)
atlas test substring

# Run tests sequentially (default is parallel)
atlas test --sequential

# Verbose output (show all test names)
atlas test --verbose

# JSON output for CI/CD integration
atlas test --json

# Run tests in specific directory
atlas test --dir ./tests
```

### CLI Flags

| Flag | Description |
|------|-------------|
| `--sequential` | Run tests one at a time (default: parallel) |
| `--verbose` | Show all test names and detailed output |
| `--no-color` | Disable colored output |
| `--json` | Output results in JSON format |
| `--dir <path>` | Test directory (default: current directory) |

### Exit Code

- **0:** All tests passed
- **1:** One or more tests failed
- **2:** Error during test discovery or execution

---

## Assertion Primitives

The Atlas standard library provides 13 assertion functions organized in 6 categories:

### Basic Assertions

#### `assert(condition: bool, message: string) -> void`

Assert that a condition is true.

```atlas
fn test_truthy() -> void {
    assert(true, "condition is true");
    assert(1 > 0, "1 is greater than 0");
}
```

**Fails with:** `AssertionError: Assertion failed: [message]`

#### `assertFalse(condition: bool, message: string) -> void`

Assert that a condition is false.

```atlas
fn test_falsy() -> void {
    assertFalse(false, "condition is false");
    assertFalse(1 < 0, "1 is not less than 0");
}
```

**Fails with:** `AssertionError: Assertion failed (expected false): [message]`

---

### Equality Assertions

#### `assertEqual(actual: T, expected: T) -> void`

Assert that two values are deeply equal. Arrays are compared element-by-element (not by reference).

```atlas
fn test_equality() -> void {
    assertEqual(2 + 2, 4);
    assertEqual("hello", "hello");
    assertEqual([1, 2, 3], [1, 2, 3]);  // Deep comparison
}
```

**Fails with:** `AssertionError: Assertion failed: values not equal\n  Actual: ...\n  Expected: ...`

#### `assertNotEqual(actual: T, expected: T) -> void`

Assert that two values are not deeply equal.

```atlas
fn test_inequality() -> void {
    assertNotEqual(1, 2);
    assertNotEqual("hello", "world");
    assertNotEqual([1, 2], [1, 2, 3]);
}
```

**Fails with:** `AssertionError: Assertion failed: values are equal (expected them to differ)\n  Value: ...`

---

### Result Assertions

#### `assertOk(result: Result<T, E>) -> T`

Assert that a `Result` is `Ok` and return the unwrapped value.

```atlas
fn test_result_ok() -> void {
    let result = Ok(42);
    let value = assertOk(result);
    assertEqual(value, 42);
}
```

**Fails with:** `AssertionError: assertOk: expected Ok, got Err(...)`

#### `assertErr(result: Result<T, E>) -> E`

Assert that a `Result` is `Err` and return the unwrapped error value.

```atlas
fn test_result_err() -> void {
    let result = Err("division by zero");
    let error = assertErr(result);
    assertEqual(error, "division by zero");
}
```

**Fails with:** `AssertionError: assertErr: expected Err, got Ok(...)`

---

### Option Assertions

#### `assertSome(option: Option<T>) -> T`

Assert that an `Option` is `Some` and return the unwrapped value.

```atlas
fn test_option_some() -> void {
    let opt = Some(42);
    let value = assertSome(opt);
    assertEqual(value, 42);
}
```

**Fails with:** `AssertionError: assertSome: expected Some, got None`

#### `assertNone(option: Option<T>) -> void`

Assert that an `Option` is `None`.

```atlas
fn test_option_none() -> void {
    let opt: Option<number> = None();
    assertNone(opt);
}
```

**Fails with:** `AssertionError: assertNone: expected None, got Some(...)`

---

### Collection Assertions

#### `assertContains(array: array, value: T) -> void`

Assert that an array contains a value (using deep equality).

```atlas
fn test_contains() -> void {
    let arr = [1, 2, 3, 4, 5];
    assertContains(arr, 3);
    assertContains(["a", "b", "c"], "b");
}
```

**Fails with:** `AssertionError: assertContains: array does not contain [value]`

#### `assertEmpty(array: array) -> void`

Assert that an array has zero elements.

```atlas
fn test_empty() -> void {
    assertEmpty([]);
    let empty: number[] = [];
    assertEmpty(empty);
}
```

**Fails with:** `AssertionError: assertEmpty: expected empty array, got length N`

#### `assertLength(array: array, expected: number) -> void`

Assert that an array has exactly the expected number of elements.

```atlas
fn test_length() -> void {
    assertLength([1, 2, 3], 3);
    assertLength([true, false], 2);
    assertLength([], 0);
}
```

**Fails with:** `AssertionError: assertLength: expected length N, got M`

---

### Error Assertions

#### `assertThrows(fn: NativeFunction) -> void`

Assert that a function throws an error. Works with `NativeFunction` values (Rust closures passed via Atlas embedding API).

```atlas
// Note: Requires native function - use with Atlas embedding API
let failing_fn = create_failing_native_fn();  // From embedding API
assertThrows(failing_fn);
```

**Fails with:** `AssertionError: assertThrows: expected function to throw, but it returned successfully`

**Note:** For bytecode functions (defined in Atlas code), wrap test logic in a native function via the embedding API.

#### `assertNoThrow(fn: NativeFunction) -> void`

Assert that a function does NOT throw an error.

```atlas
// Note: Requires native function - use with Atlas embedding API
let succeeding_fn = create_succeeding_native_fn();  // From embedding API
assertNoThrow(succeeding_fn);
```

**Fails with:** `AssertionError: assertNoThrow: expected function to succeed, but it threw: ...`

---

## Test Examples

### Example 1: Basic Math Tests

```atlas
fn test_addition() -> void {
    assertEqual(1 + 1, 2);
    assertEqual(0 + 5, 5);
    assertEqual(-1 + 1, 0);
}

fn test_subtraction() -> void {
    assertEqual(5 - 3, 2);
    assertEqual(0 - 5, -5);
}

fn test_multiplication() -> void {
    assertEqual(3 * 4, 12);
    assertEqual(0 * 100, 0);
}
```

### Example 2: String Tests

```atlas
fn test_string_equality() -> void {
    assertEqual("hello", "hello");
    assertNotEqual("hello", "world");
}

fn test_string_contains() -> void {
    let words = ["apple", "banana", "cherry"];
    assertContains(words, "banana");
    assertNotEqual(words[0], "banana");
}
```

### Example 3: Array Tests

```atlas
fn test_array_length() -> void {
    assertLength([1, 2, 3], 3);
    assertLength([], 0);
}

fn test_array_contains() -> void {
    let numbers = [1, 2, 3, 4, 5];
    assertContains(numbers, 3);
    assertNotEqual(indexOf(numbers, 3), -1);
}

fn test_array_comparison() -> void {
    assertEqual([1, 2, 3], [1, 2, 3]);
    assertNotEqual([1, 2], [1, 2, 3]);
}
```

### Example 4: Option Tests

```atlas
fn test_option_some() -> void {
    let value = Some(42);
    let unwrapped = assertSome(value);
    assertEqual(unwrapped, 42);
}

fn test_option_none() -> void {
    let empty: Option<number> = None();
    assertNone(empty);
}
```

### Example 5: Result Tests

```atlas
fn divide(a: number, b: number) -> Result<number, string> {
    if (b == 0) {
        return Err("division by zero");
    }
    return Ok(a / b);
}

fn test_divide_success() -> void {
    let result = divide(10, 2);
    let value = assertOk(result);
    assertEqual(value, 5);
}

fn test_divide_error() -> void {
    let result = divide(10, 0);
    let error = assertErr(result);
    assertEqual(error, "division by zero");
}
```

### Example 6: Complex Test

```atlas
fn test_array_operations() -> void {
    let arr = [1, 2, 3];

    // Check initial state
    assertLength(arr, 3);
    assertContains(arr, 2);

    // Test filtering
    let evens = filter(arr, fn(x: number) { return x % 2 == 0; });
    assertLength(evens, 1);
    assertContains(evens, 2);

    // Test mapping
    let doubled = map(arr, fn(x: number) { return x * 2; });
    assertEqual(doubled, [2, 4, 6]);
}
```

---

## Design Philosophy

### Why This Design?

**Minimal stdlib:** Only assertion functions, no test framework
- Keeps language lean
- Easy to understand and maintain
- Aligns with Rust/Go patterns

**Separate discovery:** CLI handles `*.test.atl` files
- Explicit test organization
- Test code is isolated from production code
- Easy to exclude tests from production builds

**Simple conventions:** `test_*` function names
- No decorators or macros needed
- Functions are transparent to the language
- Easy for IDEs and tools to recognize

**Assertion primitives:** 13 focused functions
- Cover the most common assertion needs
- Each function has clear semantics
- Deep equality by default (arrays compared element-by-element)

---

## Implementation Details

### Deep Equality

`assertEqual` uses deep equality for all types:

- **Numbers, strings, bools, null:** Compared by value
- **Arrays:** Compared element-by-element (not by reference)
- **Options/Results:** Compared by their inner values

This matches user expectations: `assertEqual([1, 2], [1, 2])` returns true.

### Error Messages

All assertion failures produce clear, actionable error messages:

```
Assertion failed: values not equal
  Actual:   [1, 2, 3]
  Expected: [1, 2, 4]
```

### Parallelization

Tests are run in parallel by default. Individual tests should be independent:

- No shared mutable state
- Each test creates its own data
- Use `--sequential` flag if tests must run in order

---

## Best Practices

### ✅ Good Test Practices

```atlas
// Clear test names
fn test_positive_numbers_add_correctly() -> void {
    assertEqual(2 + 3, 5);
}

// One assertion per behavior
fn test_array_push_appends_element() -> void {
    let arr = [1, 2, 3];
    let result = push(arr, 4);
    assertLength(result, 4);
    assertContains(result, 4);
}

// Test both success and failure cases
fn test_divide_valid() -> void {
    assertEqual(divide(10, 2), Ok(5));
}

fn test_divide_invalid() -> void {
    assertEqual(divide(10, 0), Err("division by zero"));
}

// Independent tests (no shared state)
fn test_first_operation() -> void {
    let x = 1;
    assertEqual(x + 1, 2);
}

fn test_second_operation() -> void {
    let y = 1;  // Fresh variable, not dependent on test_first_operation
    assertEqual(y * 2, 2);
}
```

### ❌ Anti-Patterns

```atlas
// Avoid vague test names
fn test_stuff() -> void {
    assertEqual(1, 1);
}

// Avoid testing multiple behaviors
fn test_array_operations() -> void {
    let arr = [1, 2, 3];
    assertEqual(length(arr), 3);
    assertContains(arr, 2);
    let doubled = map(arr, fn(x: number) { return x * 2; });
    assertEqual(doubled, [2, 4, 6]);  // Too many behaviors in one test
}

// Avoid dependent tests
fn test_setup() -> void {
    global_state = 5;  // Sharing mutable state
}

fn test_depends_on_setup() -> void {
    assertEqual(global_state, 5);  // Depends on test_setup running first
}
```

---

## See Also

- `docs/specification/stdlib.md` - Full standard library reference
- `docs/specification/diagnostic-system.md` - Error and warning system
- `docs/specification/syntax.md` - Language syntax

---

**Summary:** Atlas testing uses simple assertion primitives from stdlib paired with automatic test discovery via CLI. Tests are regular Atlas functions in `*.test.atl` files with `test_*` names. This provides powerful testing without adding complexity to the language.
