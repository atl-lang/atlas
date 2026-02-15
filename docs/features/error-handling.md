# Error Handling in Atlas

**Version:** v0.2
**Status:** Implemented
**Related:** Result<T, E> type, ? operator

---

## Overview

Atlas uses Rust-style explicit error handling with `Result<T, E>` types and the `?` operator for error propagation. This eliminates runtime exceptions and makes error handling visible in function signatures.

**Philosophy:**
- Errors are values, not exceptions
- Error handling must be explicit
- Function signatures show what can fail
- Type system prevents ignoring errors

---

## Result<T, E> Type

The `Result` type represents either success (`Ok`) or failure (`Err`):

```atlas
// Success: Result<number, string>
let success: Result<number, string> = Ok(42);

// Failure: Result<number, string>
let failure: Result<number, string> = Err("something went wrong");
```

### Type Parameters

- `T` - The success value type
- `E` - The error value type

Both can be any Atlas type (number, string, custom types, etc.).

---

## Constructing Results

### Ok Constructor

Creates a successful Result:

```atlas
fn divide(a: number, b: number) -> Result<number, string> {
    if (b == 0) {
        return Err("division by zero");
    }
    return Ok(a / b);
}
```

### Err Constructor

Creates a failed Result:

```atlas
fn validate_age(age: number) -> Result<number, string> {
    if (age < 0) {
        return Err("age cannot be negative");
    }
    if (age > 150) {
        return Err("age too large");
    }
    return Ok(age);
}
```

---

## Pattern Matching on Results

Use `match` to handle both success and failure cases:

```atlas
let result = divide(10, 2);

match result {
    Ok(value) => print("Success: " + str(value)),
    Err(error) => print("Error: " + error)
}
```

### Exhaustiveness

Match expressions on Result **must** handle both variants:

```atlas
// ✅ Valid - handles both cases
match result {
    Ok(v) => v,
    Err(e) => 0
}

// ❌ Error - missing Err case
match result {
    Ok(v) => v
}
```

---

## Error Propagation Operator (?)

The `?` operator unwraps `Ok` values or returns `Err` early:

```atlas
fn process() -> Result<number, string> {
    let x = divide(100, 10)?;  // Unwraps to 10, or returns Err
    let y = divide(x, 2)?;      // Unwraps to 5, or returns Err
    return Ok(y * 2);           // Returns Ok(10)
}
```

### How It Works

The `?` operator desugars to:

```atlas
// This code:
let value = result?;

// Is equivalent to:
let value = match result {
    Ok(v) => v,
    Err(e) => return Err(e)
};
```

### Requirements

The `?` operator can only be used:

1. On `Result<T, E>` types
2. Inside functions that return `Result<T', E'>`
3. Where `E` matches `E'` (error types must be compatible)

### Error Type Compatibility

The error type must match the function's return type:

```atlas
// ✅ Valid - both use string errors
fn calculate() -> Result<number, string> {
    let x = divide(10, 2)?;  // Result<number, string>
    return Ok(x);
}

// ❌ Error - incompatible error types
fn mismatch() -> Result<number, bool> {
    let x = divide(10, 2)?;  // Error: expects Result<_, bool>, got Result<_, string>
    return Ok(x);
}
```

### Multiple Propagations

Chain multiple `?` operations:

```atlas
fn complex_calculation() -> Result<number, string> {
    let a = divide(100, 10)?;   // 10
    let b = divide(a, 2)?;       // 5
    let c = divide(b, 5)?;       // 1
    return Ok(c * 100);          // 100
}
```

### Early Return

When `?` encounters an `Err`, it immediately returns from the function:

```atlas
fn process() -> Result<number, string> {
    let x = divide(100, 10)?;   // Ok(10)
    let y = divide(x, 0)?;       // Err("division by zero") - returns here
    let z = divide(y, 2)?;       // Never executed
    return Ok(z);                // Never executed
}
```

---

## Result Methods

Atlas provides rich methods for working with Results.

### Checking Variants

**is_ok(result: Result<T, E>) -> bool**

Returns `true` if Result is `Ok`:

```atlas
let result = Ok(42);
print(str(is_ok(result)));  // true

let error = Err("failed");
print(str(is_ok(error)));   // false
```

**is_err(result: Result<T, E>) -> bool**

Returns `true` if Result is `Err`:

```atlas
let result = Err("failed");
print(str(is_err(result)));  // true
```

### Extracting Values

**unwrap(result: Result<T, E>) -> T**

Extracts the `Ok` value, panics on `Err`:

```atlas
let result = Ok(42);
let value = unwrap(result);  // 42

let error = Err("failed");
let value = unwrap(error);   // Runtime panic!
```

⚠️ **Warning:** Only use `unwrap()` when you're certain the Result is `Ok`.

**expect(result: Result<T, E>, message: string) -> T**

Like `unwrap()` but with a custom error message:

```atlas
let result = parse_number("invalid");
let value = expect(result, "number must be valid");  // Panics with custom message
```

**unwrap_or(result: Result<T, E>, default: T) -> T**

Returns `Ok` value or a default:

```atlas
let result = Err("failed");
let value = unwrap_or(result, 0);  // 0
```

**unwrap_or_else(result: Result<T, E>, fn: (E) -> T) -> T**

Returns `Ok` value or calls a function on the error:

```atlas
fn handle_error(err: string) -> number {
    print("Error: " + err);
    return 0;
}

let result = Err("failed");
let value = unwrap_or_else(result, handle_error);  // 0, prints error
```

### Transforming Results

**result_map(result: Result<T, E>, fn: (T) -> U) -> Result<U, E>**

Transforms the `Ok` value, preserves `Err`:

```atlas
fn double(x: number) -> number {
    return x * 2;
}

let result = Ok(21);
let doubled = result_map(result, double);  // Ok(42)

let error = Err("failed");
let mapped = result_map(error, double);    // Err("failed")
```

**result_map_err(result: Result<T, E>, fn: (E) -> F) -> Result<T, F>**

Transforms the `Err` value, preserves `Ok`:

```atlas
fn format_error(e: string) -> string {
    return "Error: " + e;
}

let error = Err("failed");
let formatted = result_map_err(error, format_error);  // Err("Error: failed")

let success = Ok(42);
let mapped = result_map_err(success, format_error);   // Ok(42)
```

### Chaining Operations

**result_and_then(result: Result<T, E>, fn: (T) -> Result<U, E>) -> Result<U, E>**

Chains Result-returning operations:

```atlas
fn safe_divide(x: number) -> Result<number, string> {
    if (x == 0) {
        return Err("division by zero");
    }
    return Ok(100 / x);
}

let result = Ok(10);
let chained = result_and_then(result, safe_divide);  // Ok(10)

let zero = Ok(0);
let failed = result_and_then(zero, safe_divide);     // Err("division by zero")
```

**result_or_else(result: Result<T, E>, fn: (E) -> Result<T, E>) -> Result<T, E>**

Recovers from errors:

```atlas
fn retry(_err: string) -> Result<number, string> {
    return Ok(0);
}

let error = Err("failed");
let recovered = result_or_else(error, retry);  // Ok(0)

let success = Ok(42);
let unchanged = result_or_else(success, retry); // Ok(42)
```

### Converting to Option

**result_ok(result: Result<T, E>) -> Option<T>**

Converts `Ok` to `Some`, `Err` to `None`:

```atlas
let success = Ok(42);
let opt = result_ok(success);  // Some(42)

let error = Err("failed");
let none = result_ok(error);   // None
```

**result_err(result: Result<T, E>) -> Option<E>**

Converts `Err` to `Some`, `Ok` to `None`:

```atlas
let error = Err("failed");
let opt = result_err(error);   // Some("failed")

let success = Ok(42);
let none = result_err(success); // None
```

---

## Common Patterns

### Safe Division

```atlas
fn divide(a: number, b: number) -> Result<number, string> {
    if (b == 0) {
        return Err("division by zero");
    }
    return Ok(a / b);
}
```

### Validation

```atlas
fn validate_username(name: string) -> Result<string, string> {
    if (len(name) < 3) {
        return Err("username too short");
    }
    if (len(name) > 20) {
        return Err("username too long");
    }
    return Ok(name);
}
```

### Chaining Validations

```atlas
fn create_user(name: string, age: number) -> Result<string, string> {
    let valid_name = validate_username(name)?;
    let valid_age = validate_age(age)?;
    return Ok("User: " + valid_name + ", Age: " + str(valid_age));
}
```

### Error Recovery Pipeline

```atlas
fn process_with_fallback(input: string) -> Result<number, string> {
    let result = parse_number(input);
    let recovered = result_or_else(result, fn(_e: string) -> Result<number, string> {
        return Ok(0);  // Default value on parse error
    });
    return recovered;
}
```

### Combining Multiple Results

```atlas
fn sum_all(a: Result<number, string>, b: Result<number, string>) -> Result<number, string> {
    let x = a?;
    let y = b?;
    return Ok(x + y);
}
```

---

## Type Checking

The type checker enforces several rules for Result usage:

### 1. Result Type Required

The `?` operator only works on `Result` types:

```atlas
// ❌ Error AT3027: Not a Result type
let value = Some(42)?;  // Option is not Result
```

### 2. Function Context Required

`?` can only be used inside functions:

```atlas
// ❌ Error AT3028: Not in a function
let x = divide(10, 2)?;
```

### 3. Result-Returning Function

The function must return a `Result` type:

```atlas
// ❌ Error AT3030: Function doesn't return Result
fn process() -> number {
    let x = divide(10, 2)?;
    return x;
}

// ✅ Valid
fn process() -> Result<number, string> {
    let x = divide(10, 2)?;
    return Ok(x);
}
```

### 4. Error Type Compatibility

Error types must match:

```atlas
// ❌ Error AT3029: Error type mismatch
fn mismatch() -> Result<number, bool> {
    let x = divide(10, 2)?;  // Returns Result<_, string>
    return Ok(x);
}
```

---

## Error Codes

| Code | Description |
|------|-------------|
| `AT3027` | ? operator requires Result<T, E> type |
| `AT3028` | ? operator can only be used inside functions |
| `AT3029` | ? operator error type mismatch |
| `AT3030` | Function must return Result<T, E> to use ? |

---

## Comparison with Other Languages

### Rust

Atlas's Result type is inspired by Rust:

```rust
// Rust
fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        return Err("division by zero".to_string());
    }
    Ok(a / b)
}

// Atlas
fn divide(a: number, b: number) -> Result<number, string> {
    if (b == 0) {
        return Err("division by zero");
    }
    return Ok(a / b);
}
```

### Go

Unlike Go's multiple return values, Atlas makes errors explicit in types:

```go
// Go
func divide(a, b int) (int, error) {
    if b == 0 {
        return 0, errors.New("division by zero")
    }
    return a / b, nil
}

// Atlas
fn divide(a: number, b: number) -> Result<number, string> {
    if (b == 0) {
        return Err("division by zero");
    }
    return Ok(a / b);
}
```

### TypeScript

TypeScript uses exceptions, Atlas uses explicit Result types:

```typescript
// TypeScript
function divide(a: number, b: number): number {
    if (b === 0) {
        throw new Error("division by zero");
    }
    return a / b;
}

// Atlas
fn divide(a: number, b: number) -> Result<number, string> {
    if (b == 0) {
        return Err("division by zero");
    }
    return Ok(a / b);
}
```

---

## Best Practices

### 1. Use Descriptive Error Types

Prefer string messages for simple errors:

```atlas
fn parse_age(input: string) -> Result<number, string> {
    let num = parse_number(input)?;
    if (num < 0) {
        return Err("age cannot be negative");
    }
    return Ok(num);
}
```

### 2. Use ? for Propagation

Prefer `?` over manual matching:

```atlas
// ✅ Good
fn calculate() -> Result<number, string> {
    let x = divide(100, 10)?;
    return Ok(x * 2);
}

// ❌ Verbose
fn calculate() -> Result<number, string> {
    let result = divide(100, 10);
    match result {
        Ok(x) => return Ok(x * 2),
        Err(e) => return Err(e)
    }
}
```

### 3. Provide Context

Use `expect()` with meaningful messages:

```atlas
let config = load_config();
let value = expect(config, "config file must exist");
```

### 4. Handle Errors at Boundaries

Don't propagate errors to the top level - handle them at appropriate boundaries:

```atlas
fn main() {
    let result = process_data();
    match result {
        Ok(data) => print("Success: " + str(data)),
        Err(err) => print("Error: " + err)
    }
}
```

### 5. Avoid Unwrap in Production

Only use `unwrap()` when failure is truly impossible:

```atlas
// ⚠️ Acceptable - we just created it
let result = Ok(42);
let value = unwrap(result);

// ❌ Dangerous - this can fail
let user_input = get_input();
let value = unwrap(parse_number(user_input));  // BAD!
```

---

## Implementation Details

### Interpreter

Results are represented as `Value::Result(Result<Box<Value>, Box<Value>>)` using Rust's built-in Result type.

The `?` operator:
1. Evaluates the expression
2. Checks if it's a Result
3. If `Ok(v)`: unwraps to `v`
4. If `Err(e)`: sets control flow to early return with `Err(e)`

### VM/Compiler

The `?` operator compiles to bytecode:

```
Dup                    # Duplicate result for checking
IsResultOk             # Check if Ok variant
JumpIfFalse err_label  # Jump to error handling if Err
ExtractResultValue     # Extract Ok value
Jump done_label        # Skip error handling
err_label:
  Return               # Early return with Err
done_label:
  # Continue with Ok value
```

### Type Checking

The type checker:
1. Verifies the expression has type `Result<T, E>`
2. Extracts `T` and `E` types
3. Checks current function returns `Result<T', E'>`
4. Validates `E` == `E'` (error type compatibility)
5. Returns type `T` (the success type)

---

## Future Enhancements

Potential future additions (v0.3+):

1. **Custom Error Types**: User-defined error enums
2. **Error Context**: Stack traces and error chains
3. **Try Blocks**: `try { ... }` for grouping fallible operations
4. **Automatic Conversions**: `From` trait for error type conversions
5. **? in Match Arms**: Use `?` inside match expressions

---

## See Also

- [Type System Specification](../specification/types.md)
- [Pattern Matching](../specification/types.md#pattern-matching-types-v02)
- [Standard Library API](../api/stdlib.md#result-methods)
- [Option Type](./generics.md#optionless-than-tgreater-than)
