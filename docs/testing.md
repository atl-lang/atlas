# Atlas Testing

Atlas has a built-in test framework following Go/Rust conventions.

## Quick Start

```atlas
// math.test.atl
fn test_addition(): void {
    test.equal(2 + 2, 4);
}

fn test_array_contains(): void {
    let arr = [1, 2, 3];
    test.contains(arr, 2);
}
```

```bash
atlas test              # run all tests
atlas test --verbose    # show each test name
```

## File Convention

| Pattern | Description |
|---------|-------------|
| `*.test.atl` | Test files (discovered automatically) |
| `test_*` | Test function prefix (required) |

Test functions must:
- Start with `test_`
- Take no parameters
- Return `void` (explicit annotation required)

## CLI Reference

```bash
atlas test                      # run all *.test.atl in current dir
atlas test <pattern>            # filter tests by name
atlas test --dir=tests/         # specific directory
atlas test --verbose            # show each test name
atlas test --sequential         # disable parallel execution
atlas test --json               # JSON output for CI
atlas test --no-color           # disable colored output
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | All tests passed |
| 1 | One or more tests failed |

## Assertions (`test.*` namespace)

### Basic

```atlas
test.assert(condition, message?)      // assert condition is true
test.equal(actual, expected, msg?)    // deep equality
test.notEqual(actual, expected, msg?) // not equal
```

### Result/Option

```atlas
test.ok(result)    // assert Result is Ok, return unwrapped value
test.err(result)   // assert Result is Err, return unwrapped error
```

### Collections

```atlas
test.contains(array, value)   // array contains value (deep equality)
test.empty(array)             // array is empty
```

### Numeric

```atlas
test.approx(a, b, epsilon)    // |a - b| <= epsilon
```

### Error Handling

```atlas
test.throws(fn, msg?)         // function throws/returns Err
test.noThrow(fn, msg?)        // function succeeds
```

## Example: Complete Test File

```atlas
// user.test.atl

fn test_create_user(): void {
    let user = createUser("alice", 30);
    test.equal(user.name, "alice");
    test.equal(user.age, 30);
}

fn test_invalid_age_returns_error(): void {
    let result = tryCreateUser("bob", -5);
    test.err(result);
}

fn test_users_collection(): void {
    let users = getActiveUsers();
    test.contains(users, "alice");
    test.assert(users.length() > 0, "should have users");
}

fn test_float_calculation(): void {
    let pi = calculatePi(1000);
    test.approx(pi, 3.14159, 0.001);
}
```

## JSON Output

```bash
atlas test --json
```

```json
{
  "tests": 4,
  "passed": 3,
  "failed": 1,
  "results": [
    {"name": "test_addition", "file": "./math.test.atl", "passed": true, "duration_ms": 1},
    {"name": "test_broken", "file": "./math.test.atl", "passed": false, "duration_ms": 2}
  ]
}
```

## Test Isolation

Each test runs in an isolated runtime:
- No shared state between tests
- Tests can run in parallel (default)
- Use `--sequential` if tests have external side effects

## Assertion Failure Output

```
FAIL test_user_age (1.23ms)

──────────────────────────────────────────────────
Test result: FAILED | 5 total, 4 passed, 1 failed
Time: 12.34ms

Failures:

  ● ./user.test.atl:15
    test_user_age
      test.equal: values not equal
        Actual:   29
        Expected: 30
```
