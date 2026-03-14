# Types

The `types` module covers type checking predicates, type conversion utilities, and the built-in `Option<T>` and `Result<T, E>` wrapper types including their methods.

All functions are available globally without an import.

---

## Option\<T\>

`Option<T>` represents a value that may or may not be present. Constructed with `Some(value)` or `None`.

### Construction

```atlas
let opt: Option<number> = Some(42);
let empty: Option<number> = None;
```

### `.isSome() -> bool`

Returns `true` if the Option contains a value.

```atlas
Some(42).isSome()   // true
None.isSome()       // false
```

### `.isNone() -> bool`

Returns `true` if the Option is `None`.

```atlas
None.isNone()       // true
Some(1).isNone()    // false
```

### `.unwrap() -> T`

Returns the contained value. Panics if the Option is `None`.

```atlas
let n = Some(42).unwrap();
// n == 42

None.unwrap();
// runtime panic: "unwrap() called on None"
```

### `.unwrapOr(default: T) -> T`

Returns the contained value, or `default` if `None`.

```atlas
let n = Some(42).unwrapOr(0);
// n == 42

let n2 = None.unwrapOr(0);
// n2 == 0
```

### `.expect(message: string) -> T`

Returns the contained value. Panics with `message` if the Option is `None`.

```atlas
let n = Some(42).expect("must have value");
// n == 42

None.expect("expected a number");
// runtime panic: "expected a number"
```

### `.map(fn: (T) -> U) -> Option<U>`

Transforms the contained value using `fn`. If `None`, returns `None` unchanged. This is a VM intrinsic.

```atlas
let doubled = Some(21).map(fn(borrow x: number): number { return x * 2; });
// doubled == Some(42)

let still_none = None.map(fn(borrow x: number): number { return x * 2; });
// still_none == None
```

### `.andThen(fn: (T) -> Option<U>) -> Option<U>`

Chains Option operations. If `Some`, applies `fn` to the value and returns the result. If `None`, returns `None`. This is a VM intrinsic.

```atlas
fn safeDivide(borrow n: number): Option<number> {
    if n == 0 { return None; }
    return Some(100 / n);
}

let result = Some(5).andThen(safeDivide);
// result == Some(20)

let result2 = Some(0).andThen(safeDivide);
// result2 == None
```

---

## Result\<T, E\>

`Result<T, E>` represents either a successful value (`Ok`) or an error (`Err`). Constructed with `Ok(value)` or `Err(error)`.

### Construction

```atlas
let ok: Result<number, string> = Ok(42);
let bad: Result<number, string> = Err("something failed");
```

### `.isOk() -> bool`

Returns `true` if the Result is `Ok`.

```atlas
Ok(42).isOk()    // true
Err("x").isOk()  // false
```

### `.isErr() -> bool`

Returns `true` if the Result is `Err`.

```atlas
Err("x").isErr()  // true
Ok(42).isErr()    // false
```

### `.unwrap() -> T`

Returns the contained value. Panics with the error value if the Result is `Err`.

```atlas
let n = Ok(42).unwrap();
// n == 42

Err("failed").unwrap();
// runtime panic: "unwrap() called on Err(failed)"
```

### `.unwrapErr() -> E`

Returns the error value. Panics if the Result is `Ok`. This is a VM intrinsic.

```atlas
let e = Err("oops").unwrapErr();
// e == "oops"
```

### `.unwrapOr(default: T) -> T`

Returns the contained value, or `default` if `Err`.

```atlas
let n = Ok(42).unwrapOr(0);
// n == 42

let n2 = Err("x").unwrapOr(0);
// n2 == 0
```

### `.expect(message: string) -> T`

Returns the contained value. Panics with `message: errValue` if `Err`.

```atlas
let n = Ok(42).expect("must succeed");
// n == 42

Err("bad").expect("parsing failed");
// runtime panic: "parsing failed: bad"
```

### `.context(message: string) -> Result<T, string>`

Wraps an `Err` with additional context. If `Ok`, passes through unchanged. The new error is formatted as `"message: originalError"`.

```atlas
let r = Err("file not found").context("reading config");
// r == Err("reading config: file not found")

let r2 = Ok(42).context("ignored");
// r2 == Ok(42)
```

### `.ok() -> Option<T>`

Converts a `Result` to an `Option`, discarding the error. `Ok(v) -> Some(v)`, `Err(_) -> None`.

```atlas
let opt = Ok(42).ok();
// opt == Some(42)

let opt2 = Err("x").ok();
// opt2 == None
```

### `.err() -> Option<E>`

Converts a `Result` to an `Option` containing the error. `Ok(_) -> None`, `Err(e) -> Some(e)`.

```atlas
let opt = Err("failed").err();
// opt == Some("failed")

let opt2 = Ok(42).err();
// opt2 == None
```

### `.map(fn: (T) -> U) -> Result<U, E>`

Transforms the `Ok` value using `fn`. `Err` passes through unchanged. This is a VM intrinsic.

```atlas
let r = Ok(21).map(fn(borrow x: number): number { return x * 2; });
// r == Ok(42)

let r2 = Err("x").map(fn(borrow x: number): number { return x * 2; });
// r2 == Err("x")
```

### `.mapErr(fn: (E) -> F) -> Result<T, F>`

Transforms the `Err` value using `fn`. `Ok` passes through unchanged. This is a VM intrinsic.

```atlas
let r = Err("raw error").mapErr(fn(borrow e: string): string {
    return "Wrapped: " + e;
});
// r == Err("Wrapped: raw error")
```

### `.andThen(fn: (T) -> Result<U, E>) -> Result<U, E>`

Chains Result operations (flat-map). If `Ok`, applies `fn`. If `Err`, returns `Err` unchanged. This is a VM intrinsic.

```atlas
fn parseNum(borrow s: string): Result<number, string> {
    return s.toNumber();
}

let r = Ok("42").andThen(parseNum);
// r == Ok(42)

let r2 = Err("skip").andThen(parseNum);
// r2 == Err("skip")
```

---

## Type Checking Predicates

### `isString(value: any) -> bool`

```atlas
isString("hello")   // true
isString(42)        // false
```

### `isNumber(value: any) -> bool`

Returns `true` for any `number`, including `NaN` (NaN is still typed as number).

```atlas
isNumber(42)        // true
isNumber(0)         // true
isNumber("42")      // false
```

### `isBool(value: any) -> bool`

```atlas
isBool(true)        // true
isBool(1)           // false
```

### `isNull(value: any) -> bool`

```atlas
isNull(null)        // true
isNull(0)           // false
```

### `isArray(value: any) -> bool`

```atlas
isArray([1, 2, 3])  // true
isArray("abc")      // false
```

### `isFunction(value: any) -> bool`

Returns `true` for user-defined functions and builtins.

```atlas
isFunction(console.log)             // true
isFunction(fn(borrow x: number): number { return x; })  // true
isFunction(42)                      // false
```

### `isObject(value: any) -> bool`

Returns `true` only for JSON objects (records), not Atlas structs or maps.

```atlas
let obj = Json.parse("{}").unwrap();
isObject(obj)   // true
isObject([])    // false
```

### `isType(value: any, typeName: string) -> bool`

Checks the runtime type name of `value` against `typeName`. The type name string matches the values returned by `type()`.

```atlas
isType(42, "number")       // true
isType("hi", "string")     // true
isType([1, 2], "array")    // true
isType(None, "option")     // true
```

### `hasField(value: any, field: string) -> bool`

Returns `true` if `value` is a JSON object or HashMap containing the given key.

```atlas
let obj = Json.parse("{\"x\": 1}").unwrap();
hasField(obj, "x")    // true
hasField(obj, "y")    // false
```

### `hasTag(value: any, tag: string) -> bool`

Returns `true` if `value` is a JSON object or HashMap with a `"tag"` key matching `tag`. Useful for discriminated union patterns.

```atlas
let event = Json.parse("{\"tag\": \"click\", \"x\": 10}").unwrap();
hasTag(event, "click")   // true
hasTag(event, "hover")   // false
```

---

## Type Conversion

### `toNumber(value: any) -> Result<number, string>`

Safely converts a value to number. Returns `Result` so invalid conversions are handled rather than panicked.

| Input | Result |
|-------|--------|
| `number` | `Ok(n)` |
| `true` | `Ok(1)` |
| `false` | `Ok(0)` |
| `"42"` | `Ok(42)` |
| `"3.14"` | `Ok(3.14)` |
| `""` (empty) | `Err(...)` |
| `"abc"` | `Err(...)` |
| `null`, `array`, etc. | `Err(...)` |

```atlas
let r = toNumber("42");
// r == Ok(42)

let r2 = toNumber("not a number");
// r2 == Err("Cannot parse 'not a number' as number")

match toNumber(userInput) {
    Ok(n) => console.log("got: " + n.toString()),
    Err(e) => console.error("invalid: " + e),
}
```

### `toString(value: any) -> string`

Converts any value to its string representation. Equivalent to the global `str()` function.

```atlas
let s = toString(42);
// s == "42"

let s2 = toString(Some("hi"));
// s2 == "Some(\"hi\")"
```

### `toBool(value: any) -> bool`

Converts a value to boolean using JavaScript-like truthiness rules.

- `false` → `false`
- `0`, `NaN` → `false`
- `""` (empty string) → `false`
- `null` → `false`
- Everything else → `true`

```atlas
let b = toBool(0);       // false
let b2 = toBool("hi");   // true
let b3 = toBool([]);     // true  (array is always truthy)
```

### String-to-number parsing

Use method syntax on strings (D-045 — no bare global `parseInt`/`parseFloat`):

- **`.toInt(radix?: number) -> Result<number, string>`** — parse integer, default radix 10
- **`.toNumber() -> Result<number, string>`** — parse float

```atlas
let r = "42".toInt();
// r == Ok(42)

let r2 = "FF".toInt(16);
// r2 == Ok(255)

let r3 = "1010".toInt(2);
// r3 == Ok(10)

let r4 = "xyz".toInt();
// r4 == Err("...")

let rf = "3.14".toNumber();
// rf == Ok(3.14)

let rf2 = "abc".toNumber();
// rf2 == Err("...")
```

---

## number Instance Methods

### `.toString() -> string`

Returns the canonical string representation of a number.

```atlas
let s = (42).toString();
// s == "42"

let s2 = (3.14).toString();
// s2 == "3.14"

let s3 = (0 / 0).toString();
// s3 == "NaN"
```

### `.toFixed(digits: number) -> string`

Formats the number with exactly `digits` decimal places.

```atlas
let s = (3.14159).toFixed(2);
// s == "3.14"

let s2 = (1).toFixed(3);
// s2 == "1.000"
```

### `.toInt() -> number`

Truncates the number toward zero (equivalent to `Math.trunc`).

```atlas
let n = (3.9).toInt();
// n == 3

let n2 = (-3.9).toInt();
// n2 == -3
```

---

## bool Instance Methods

### `.toString() -> string`

Returns `"true"` or `"false"`.

```atlas
let s = true.toString();
// s == "true"
```
