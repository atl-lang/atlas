# Atlas Standard Library

This document describes the Atlas standard library API.

---

## Testing Primitives

Atlas provides built-in assertion functions for writing tests. These are the
building blocks used by the `atlas test` CLI command (CLI/phase-02).

### Design Philosophy

Following the Rust/Go model:
- `stdlib` provides assertion **primitives** only
- `atlas test` CLI handles **discovery, execution, and reporting**

### Assertions

#### Basic

| Function | Signature | Description |
|----------|-----------|-------------|
| `assert` | `(condition: bool, message: string) -> void` | Assert condition is true |
| `assertFalse` | `(condition: bool, message: string) -> void` | Assert condition is false |

#### Equality

| Function | Signature | Description |
|----------|-----------|-------------|
| `assertEqual` | `(actual: T, expected: T) -> void` | Assert values are deeply equal |
| `assertNotEqual` | `(actual: T, expected: T) -> void` | Assert values are not equal |

Deep equality: arrays are compared element-by-element (not by reference).

#### Result

| Function | Signature | Description |
|----------|-----------|-------------|
| `assertOk` | `(result: Result<T, E>) -> T` | Assert `Result` is `Ok`; return unwrapped value |
| `assertErr` | `(result: Result<T, E>) -> E` | Assert `Result` is `Err`; return unwrapped error |

#### Option

| Function | Signature | Description |
|----------|-----------|-------------|
| `assertSome` | `(option: Option<T>) -> T` | Assert `Option` is `Some`; return unwrapped value |
| `assertNone` | `(option: Option<T>) -> void` | Assert `Option` is `None` |

#### Collections

| Function | Signature | Description |
|----------|-----------|-------------|
| `assertContains` | `(array: array, value: T) -> void` | Assert array contains value (deep equality) |
| `assertEmpty` | `(array: array) -> void` | Assert array has zero elements |
| `assertLength` | `(array: array, expected: number) -> void` | Assert array has exactly `expected` elements |

#### Error

| Function | Signature | Description |
|----------|-----------|-------------|
| `assertThrows` | `(fn: NativeFunction) -> void` | Assert function throws (returns Err) |
| `assertNoThrow` | `(fn: NativeFunction) -> void` | Assert function does not throw |

> **Note:** `assertThrows` and `assertNoThrow` work with `NativeFunction` values
> (Rust closures passed via the Atlas embedding API). Test discovery and Atlas-code
> test execution are handled by `atlas test` (CLI/phase-02).

### Example

```atlas
fn divide(a: number, b: number) -> Result<number, string> {
    if (b == 0) { return Err("division by zero"); }
    return Ok(a / b);
}

fn test_divide() -> void {
    // Test successful division
    let result = divide(10, 2);
    let value = assertOk(result);
    assertEqual(value, 5);

    // Test division by zero
    let err_result = divide(5, 0);
    let err_msg = assertErr(err_result);
    assertEqual(err_msg, "division by zero");
}

test_divide();
```

```atlas
fn test_collections() -> void {
    let nums = [1, 2, 3];
    assertLength(nums, 3);
    assertContains(nums, 2);
    assertNotEqual(nums[0], nums[2]);

    let empty = [];
    assertEmpty(empty);
}

test_collections();
```

### Running Tests

Tests are discovered and executed via the CLI (CLI/phase-02):

```sh
atlas test            # run all test_* functions
atlas test --filter divide  # run matching tests
```

### What Stdlib Does NOT Provide

- Test discovery (CLI/phase-02)
- Test execution (CLI/phase-02)
- Parallel test execution (CLI/phase-02)
- Test reporting and output formatting (CLI/phase-02)
