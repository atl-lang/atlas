# Core

Core globals are available in every Atlas program without any import. They cover output, type conversion, introspection, and program control.

---

## console namespace

The `console` namespace provides output functions. `log` and `println` write to stdout with a newline. `print` writes to stdout without a newline. `error`, `warn`, and `debug` write to stderr.

All `console` functions accept any number of arguments of any type. Multiple arguments are joined with a space.

### `console.log(...args: any) -> void`

Prints arguments to stdout followed by a newline.

```atlas
console.log("Hello, world!");
console.log("x =", 42, "done");
// stdout: Hello, world!
// stdout: x = 42 done
```

### `console.println(...args: any) -> void`

Alias for `console.log`. Identical behavior.

```atlas
console.println("Same as log");
```

### `console.print(...args: any) -> void`

Prints arguments to stdout **without** a trailing newline.

```atlas
console.print("Enter name: ");
```

### `console.error(...args: any) -> void`

Prints arguments to stderr followed by a newline.

```atlas
console.error("Something went wrong:", errMsg);
```

### `console.warn(...args: any) -> void`

Prints arguments to stderr with a `WARN: ` prefix.

```atlas
console.warn("Deprecated function used");
// stderr: WARN: Deprecated function used
```

### `console.debug(...args: any) -> void`

Prints arguments to stderr with a `DEBUG: ` prefix.

```atlas
console.debug("state =", value);
// stderr: DEBUG: state = 42
```

---

## Type Conversion

### `str(value: any) -> string`

Converts any value to its string representation.

| Input | Output |
|-------|--------|
| `null` | `"null"` |
| `true` / `false` | `"true"` / `"false"` |
| `42` | `"42"` |
| `3.14` | `"3.14"` |
| `NaN` | `"NaN"` |
| `Infinity` | `"Infinity"` |
| `Some(x)` | `"Some(x)"` |
| `None` | `"None"` |
| `Ok(x)` | `"Ok(x)"` |
| `Err(e)` | `"Err(e)"` |

```atlas
let s = str(42);
// s == "42"

let s2 = str(true);
// s2 == "true"

let s3 = str(None);
// s3 == "None"
```

### `num(value: string) -> number`

Parses a string as a number. Trims whitespace before parsing. Panics if the string is not a valid number.

> For safe conversion that returns a `Result`, use `toNumber()` from the `types` module.

```atlas
let n = num("42");
// n == 42

let f = num("3.14");
// f == 3.14
```

### `bool(value: any) -> bool`

Converts a value to boolean using JavaScript-like truthiness:

- `false`, `0`, `NaN`, `""` (empty string), `null` → `false`
- Everything else → `true`

```atlas
let b = bool(0);
// b == false

let b2 = bool("hello");
// b2 == true

let b3 = bool(null);
// b3 == false
```

---

## Length

### `len(value: string | any[]) -> number`

Returns the length of a string (number of Unicode characters) or an array (number of elements).

```atlas
let n = len("hello");
// n == 5

let n2 = len([1, 2, 3]);
// n2 == 3
```

---

## Type Introspection

### `type(value: any) -> string`

Returns the runtime type name of `value` as a string.

| Value | Returns |
|-------|---------|
| `null` | `"null"` |
| `true` / `false` | `"boolean"` |
| `42` | `"number"` |
| `"hello"` | `"string"` |
| `[1, 2]` | `"array"` |
| `fn() {}` | `"function"` |
| `Some(x)` | `"option"` |
| `HashMap` | `"map"` |
| `HashSet` | `"set"` |
| `(1, 2)` | `"tuple"` |

```atlas
let t = type(42);
// t == "number"

let t2 = type([1, 2, 3]);
// t2 == "array"
```

---

## Error Handling

### `panic(message: string) -> void`

Terminates the program immediately with an error message. Use for unrecoverable errors that indicate a programming bug.

```atlas
fn divide(borrow a: number, borrow b: number): number {
    if b == 0 {
        panic("divide by zero");
    }
    return a / b;
}
```

### `assertEq(actual: any, expected: any) -> void`

Asserts that `actual` equals `expected`. Panics with a descriptive message if they differ.

```atlas
assertEq(1 + 1, 2);
assertEq("hello".length(), 5);
```

---

## test namespace

The `test` namespace is used for writing test assertions.

### `test.assert(condition: bool, message?: string) -> void`

Asserts that `condition` is `true`. Panics with `message` (or a default message) if `false`.

```atlas
test.assert(1 + 1 == 2, "basic arithmetic");
test.assert("hello".length() == 5);
```

### `test.assertEq(actual: any, expected: any) -> void`

Asserts that `actual` equals `expected`. Panics with a message showing both values if they differ.

```atlas
test.assertEq(Math.abs(-5), 5);
test.assertEq([1, 2].length(), 2);
```
