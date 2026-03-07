# Atlas Stdlib — AI Generation Notes

This file documents stdlib functions whose return types differ from naive expectations.
**Read this before generating Atlas code that calls stdlib functions.**

---

## Functions That Return `Result` or `Option` (Not the Raw Value)

These are the most common sources of first-pass generation errors:

| Function | Naive (wrong) | Actual return type | Correct pattern |
|----------|---------------|-------------------|-----------------|
| `sqrt(x)` | `number` | `Result<number, string>` | `let x = unwrap(sqrt(n));` |
| `parseJSON(s)` | `json` | `Result<json, string>` | `let data = unwrap(parseJSON(s));` |
| `indexOf(s, q)` | `number` | `Option<number>` | `match indexOf(s, q) { Some(n) => n, None => -1 }` |
| `log(x)` | `number` | `Result<number, string>` | `let v = unwrap(log(x));` |
| `asin(x)` | `number` | `Result<number, string>` | `let v = unwrap(asin(x));` |
| `acos(x)` | `number` | `Result<number, string>` | `let v = unwrap(acos(x));` |
| `clamp(v,lo,hi)` | `number` | `Result<number, string>` | `let v = unwrap(clamp(x, 0, 100));` |
| `lastIndexOf(s,q)` | `number` | `Option<number>` | `match lastIndexOf(s, q) { Some(n) => n, None => -1 }` |
| `charAt(s, i)` | `string` | `Option<string>` | `match charAt(s, i) { Some(c) => c, None => "" }` |

---

## Correct `unwrap` Usage

`unwrap(value)` extracts the inner value from `Option<T>` or `Result<T,E>`. Panics if `None` or `Err`.

```atlas
// Safe for known-valid inputs:
let root = unwrap(sqrt(16));        // 4.0
let data = unwrap(parseJSON(raw));  // json value

// Safe match for uncertain inputs:
let idx = match indexOf(str, "x") {
    Some(n) => n,
    None    => -1
};

let parsed = match parseJSON(input) {
    Ok(data) => data,
    Err(e)   => { print("parse error: " + e); null }
};
```

---

## Why These Return `Result`/`Option`

These functions can fail on valid inputs:
- `sqrt(-1)` → domain error
- `parseJSON("not json")` → malformed input
- `indexOf("abc", "z")` → not found

Atlas makes failure explicit rather than returning `NaN`, `-1`, or throwing. This is correct behavior — but it means you must always unwrap.

---

## Arrays, Collections — CoW Rebind Pattern

Array/collection mutation functions return a **new value**. Always rebind:

```atlas
// WRONG:
arrayPush(arr, 4);         // result discarded
hashMapPut(map, "k", "v"); // result discarded

// CORRECT:
arr = arrayPush(arr, 4);
map = hashMapPut(map, "k", "v");
```

See `docs/stdlib/array.md` and `docs/stdlib/collections.md` for full details.
