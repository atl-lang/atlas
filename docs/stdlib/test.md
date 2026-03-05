# Testing Functions

Unit testing assertions and test utilities.

## Basic Assertions

### assert

```atlas
fn assert(condition: bool) -> Result<Null, string>
```

Asserts that condition is true.

**Parameters:**
- `condition` - Boolean expression

**Returns:**
- `Ok(Null)` if true
- `Err(string)` with failure message if false

### assertEqual

```atlas
fn assertEqual(actual: any, expected: any) -> Result<Null, string>
```

Asserts two values are equal.

**Parameters:**
- `actual` - Actual value
- `expected` - Expected value

**Returns:**
- `Ok(Null)` if equal
- `Err(string)` with diff if not equal

### assertNotEqual

```atlas
fn assertNotEqual(actual: any, expected: any) -> Result<Null, string>
```

Asserts two values are not equal.

**Parameters:**
- `actual` - Actual value
- `expected` - Unexpected value

**Returns:**
- `Ok(Null)` if not equal
- `Err(string)` if equal

## Boolean Assertions

### assertTrue

```atlas
fn assertTrue(condition: bool) -> Result<Null, string>
```

Asserts condition is true (alias for assert).

**Parameters:**
- `condition` - Boolean expression

**Returns:**
- `Ok(Null)` if true
- `Err(string)` if false

### assertFalse

```atlas
fn assertFalse(condition: bool) -> Result<Null, string>
```

Asserts condition is false.

**Parameters:**
- `condition` - Boolean expression

**Returns:**
- `Ok(Null)` if false
- `Err(string)` if true

## Collection Assertions

### assertEmpty

```atlas
fn assertEmpty(value: any) -> Result<Null, string>
```

Asserts collection is empty.

**Parameters:**
- `value` - Array, string, or map

**Returns:**
- `Ok(Null)` if empty
- `Err(string)` if not empty

### assertLength

```atlas
fn assertLength(value: any, length: number) -> Result<Null, string>
```

Asserts collection has specific length.

**Parameters:**
- `value` - Array, string, or map
- `length` - Expected length (integer)

**Returns:**
- `Ok(Null)` if matches
- `Err(string)` if doesn't match

### assertContains

```atlas
fn assertContains(collection: any, item: any) -> Result<Null, string>
```

Asserts collection contains item.

**Parameters:**
- `collection` - Array, string, or map
- `item` - Item to check for

**Returns:**
- `Ok(Null)` if found
- `Err(string)` if not found

## Option/Result Assertions

### assertSome

```atlas
fn assertSome(opt: Option<T>) -> Result<T, string>
```

Asserts Option is Some and returns value.

**Parameters:**
- `opt` - Option value

**Returns:**
- `Ok(T)` with unwrapped value
- `Err(string)` if None

### assertNone

```atlas
fn assertNone(opt: Option<T>) -> Result<Null, string>
```

Asserts Option is None.

**Parameters:**
- `opt` - Option value

**Returns:**
- `Ok(Null)` if None
- `Err(string)` if Some

### assertOk

```atlas
fn assertOk(res: Result<T, E>) -> Result<T, string>
```

Asserts Result is Ok and returns value.

**Parameters:**
- `res` - Result value

**Returns:**
- `Ok(T)` with success value
- `Err(string)` if Err

### assertErr

```atlas
fn assertErr(res: Result<T, E>) -> Result<E, string>
```

Asserts Result is Err and returns error.

**Parameters:**
- `res` - Result value

**Returns:**
- `Ok(E)` with error value
- `Err(string)` if Ok

## Exception Assertions

### assertThrows

```atlas
fn assertThrows(fn_call: fn() -> any) -> Result<string, string>
```

Asserts function throws an exception.

**Parameters:**
- `fn_call` - Function that should throw

**Returns:**
- `Ok(string)` with error message if throws
- `Err(string)` if no exception thrown

### assertNoThrow

```atlas
fn assertNoThrow(fn_call: fn() -> any) -> Result<Null, string>
```

Asserts function does not throw.

**Parameters:**
- `fn_call` - Function that shouldn't throw

**Returns:**
- `Ok(Null)` if no exception
- `Err(string)` with exception message if throws

## Example Usage

```atlas
// Basic assertions
assert(true)?;
assertEqual(2 + 2, 4)?;
assertNotEqual(5, 10)?;

// Boolean
assertTrue(x > 0)?;
assertFalse(x < 0)?;

// Collections
let arr = [1, 2, 3];
assertLength(arr, 3)?;
assertContains(arr, 2)?;

// Options
let some_val = Some(42);
let val = assertSome(some_val)?;
print(val); // 42

// Results
let ok_val = Ok(10);
let val = assertOk(ok_val)?;
print(val); // 10

// Exceptions
let throws = || { throw "error"; };
let msg = assertThrows(throws)?;
print(msg); // "error"

// Combine with test framework
test("addition", || {
  assertEqual(2 + 2, 4)?;
  assertEqual(1 + 1, 2)?;
  Ok(Null)
})?;
```

## Testing Patterns

### Basic Test

```atlas
fn my_test() -> Result<Null, string> {
  assertEqual(add(2, 3), 5)?;
  assertEqual(add(-1, 1), 0)?;
  Ok(Null)
}
```

### Multiple Assertions

```atlas
fn test_array_ops() -> Result<Null, string> {
  let arr = [1, 2, 3];
  assertLength(arr, 3)?;
  assertContains(arr, 2)?;
  assertEqual(arrayIndexOf(arr, 2), Some(1))?;
  Ok(Null)
}
```

### Option Testing

```atlas
fn test_option_handling() -> Result<Null, string> {
  let opt = Some(42);
  assertSome(opt)?;

  let none = None;
  assertNone(none)?;

  Ok(Null)
}
```

### Result Testing

```atlas
fn test_error_handling() -> Result<Null, string> {
  let ok = Ok(10);
  let val = assertOk(ok)?;
  assertEqual(val, 10)?;

  let err = Err("failure");
  assertErr(err)?;

  Ok(Null)
}
```
