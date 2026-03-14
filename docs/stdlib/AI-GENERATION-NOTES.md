# AI Generation Notes — Atlas Standard Library

This document is a guide for AI models generating Atlas stdlib calls. It covers import
patterns, the most common mistakes, idioms, type coercion rules, and naming conventions.

Read this before generating any Atlas code that calls stdlib functions.

---

## 1. No Imports for Stdlib

Atlas stdlib namespaces are globally available — **do not generate import statements for
them**.

```atlas
// WRONG — do not do this:
import { Json } from "stdlib";
import Json from "atlas:json";

// CORRECT — just call directly:
let result = Json.parse(text);
```

The only imports in Atlas are for user modules:

```atlas
import { myFn } from "./mymodule";
```

---

## 2. Namespace Casing Is Mandatory (D-049)

Namespace casing is enforced by the dispatch table. Wrong casing = function not found
= runtime error.

| Namespace | Casing | Example |
|-----------|--------|---------|
| `console` | lowercase | `console.log(...)` |
| `test` | lowercase | `test.assert(...)` |
| `io` | lowercase | `io.readLine()` |
| `file` | lowercase | `file.read(path)` |
| `task` | lowercase | `task.spawn(fn)` |
| `future` | lowercase | `future.resolve(val)` |
| `sync` | lowercase | `sync.channel()` |
| `process` | lowercase | `process.exec(cmd)` |
| `reflect` | lowercase | `reflect.typeOf(v)` |
| `sqlite` | lowercase | `sqlite.open(path)` |
| `Math` | PascalCase | `Math.sqrt(n)` |
| `Json` | PascalCase | `Json.parse(s)` |
| `Encoding` | PascalCase | `Encoding.base64Encode(s)` |
| `Regex` | PascalCase | `Regex.new(pattern)` |
| `DateTime` | PascalCase | `DateTime.now()` |
| `Path` | PascalCase | `Path.join(a, b)` |
| `Env` | PascalCase | `Env.get(key)` |
| `Http` | PascalCase | `Http.get(url)` |
| `Net` | PascalCase | `Net.tcpConnect(...)` |
| `Crypto` | PascalCase | `Crypto.sha256(data)` |
| `Gzip` | PascalCase | `Gzip.compress(data)` |
| `Tar` | PascalCase | `Tar.create(...)` |
| `Zip` | PascalCase | `Zip.create(...)` |

---

## 3. Result and Option Are Everywhere — Handle Them

Many stdlib functions return `Result<T, E>` or `Option<T>`. Never discard them silently.

### Pattern: match on Result

```atlas
match Json.parse(raw) {
    Ok(data) => {
        // use data
    }
    Err(e) => {
        console.error("failed: " + e);
    }
}
```

### Pattern: unwrap when you know it's safe

```atlas
let pattern = Regex.new("\\d+").unwrap();
```

### Pattern: unwrapOr for defaults

```atlas
let name = Json.getString(data, "name").unwrapOr("anonymous");
let count = Json.getNumber(data, "count").unwrapOr(0);
```

### Pattern: isSome / isNone guards

```atlas
let m = Regex.find(pattern, text);
if m.isSome() {
    let match = m.unwrap();
}
```

**Do not** treat `Result` or `Option` as the inner value without unwrapping:

```atlas
// WRONG — 'result' is Result<JsonValue, string>, not JsonValue:
let name = Json.getString(result, "name");

// CORRECT:
let name = Json.getString(result.unwrap(), "name");
```

---

## 4. JsonValue Is Not a Native Atlas Value

`Json.parse()` returns `JsonValue`, not `string`, `number`, etc. You must extract fields
explicitly.

```atlas
// WRONG — JsonValue is not directly usable as a string:
let name = Json.parse('{"name":"Atlas"}').unwrap();
console.log(name); // This is a JsonValue, not "Atlas"

// CORRECT — extract the field:
let data = Json.parse('{"name":"Atlas"}').unwrap();
let name = Json.getString(data, "name").unwrapOr("");
console.log(name); // "Atlas"
```

Use the typed getters — they return `Option<T>` (safe):
- `Json.getString(json, key): Option<string>`
- `Json.getNumber(json, key): Option<number>`
- `Json.getBool(json, key): Option<bool>`
- `Json.getArray(json, key): Option<JsonValue[]>`
- `Json.getObject(json, key): Option<JsonValue>`

---

## 5. Regex Must Be Compiled Before Use

You cannot pass a pattern string directly to matching functions. Always compile first
with `Regex.new()` or `Regex.newWithFlags()`.

```atlas
// WRONG:
let matched = Regex.isMatch("\\d+", "hello123");

// CORRECT:
let re = Regex.new("\\d+").unwrap();
let matched = Regex.isMatch(re, "hello123");
```

Exception: `Regex.test(pattern, text)` accepts raw pattern strings and is safe to use
for one-off checks — it returns `false` on compile error rather than panicking.

---

## 6. Escape Backslashes in Regex Patterns

Atlas string literals use `\` as an escape character. In regex patterns, `\d` must be
written as `"\\d"`.

```atlas
// WRONG — \d is not a valid Atlas string escape:
let re = Regex.new("\d+");

// CORRECT:
let re = Regex.new("\\d+").unwrap();
let re2 = Regex.new("\\w+\\s+\\w+").unwrap();
```

---

## 7. All DateTime Values Are UTC

`DateTime.now()`, `DateTime.fromTimestamp()`, and all other constructors return UTC.
The timezone getters always return `"UTC"` and offset `0`. Timezone conversion functions
produce UTC results (they do the math internally).

```atlas
let now = DateTime.now(); // UTC always
let ts = DateTime.toTimestamp(now);
let iso = DateTime.toIso(now); // ends with "+00:00"
```

---

## 8. Encoding Functions Take Strings, Return Strings

All `Encoding.*` functions take `string` and return `string`. They do not work with byte
arrays. If you need to encode binary data, you must first represent it as a string.

```atlas
let encoded = Encoding.base64Encode("Hello, Atlas!");
let decoded = Encoding.base64Decode(encoded);
```

---

## 9. Gzip Uses number[] for Bytes

Unlike `Encoding`, the `Gzip` namespace works with byte arrays (`number[]`, each element
0–255).

```atlas
// Compress a string → get bytes back
let bytes = Gzip.compress("hello world");  // number[]

// Decompress bytes → get string back
let text = Gzip.decompressString(bytes);

// Decompress bytes → get bytes back
let rawBytes = Gzip.decompress(bytes);
```

---

## 10. Duration Is a Map, Not a Type

`DateTime.durationFromSeconds()` etc. return a `HashMap`, not a special `Duration` type.
The keys are `"days"`, `"hours"`, `"minutes"`, `"seconds"`.

```atlas
let dur = DateTime.durationFromSeconds(3665);
// dur is a HashMap with:
//   dur.get("hours")   == 1
//   dur.get("minutes") == 1
//   dur.get("seconds") == 5

let label = DateTime.durationFormat(dur); // "1h 1m 5s"
```

---

## 11. reflect.fields() Returns String Keys Only

For `HashMap` and struct instances, `reflect.fields()` returns `string[]` of key names.
For all other types it returns `[]`.

```atlas
struct Foo { x: number, y: string }
let f = Foo { x: 1, y: "hi" };
let keys = reflect.fields(f); // ["x", "y"]

reflect.fields(42);    // []
reflect.fields("str"); // []
```

---

## 12. Semicolons Are Required for Expression Statements

Every expression statement requires a semicolon. Missing semicolons are a parse error.

```atlas
// WRONG:
console.log("hello")
let x = Regex.new("\\d+")

// CORRECT:
console.log("hello");
let x = Regex.new("\\d+").unwrap();
```

---

## 13. Method Chaining vs. Namespace Calls

Most stdlib values support both method syntax and namespace syntax. The method syntax
is idiomatic:

```atlas
// Namespace syntax (always valid):
let upper = toUpperCase(s);

// Method syntax (idiomatic):
let upper = s.toUpperCase();

// Both are correct for Option/Result:
let val = option.unwrap();
let val2 = unwrap(option);
```

For `Regex`, `DateTime`, and `JsonValue`, the method syntax delegates to the same
underlying implementation.

---

## 14. Arity Is Strict

Atlas stdlib functions check argument count at runtime. Passing too few or too many
arguments raises an `InvalidStdlibArgument` error immediately.

Common arity mistakes:
- `Json.prettify(text)` → WRONG (requires 2 args: `text, indent`)
- `DateTime.fromComponents(2024, 6)` → WRONG (requires 6 args)
- `Regex.replace(re, text)` → WRONG (requires 3 args: `re, text, replacement`)

---

## 15. Type Coercion Does Not Exist

Atlas does not implicitly coerce types. Pass the exact type expected.

```atlas
// WRONG — Math.sqrt expects number, not string:
Math.sqrt("4");

// CORRECT:
Math.sqrt(4.0);

// WRONG — DateTime.fromComponents expects numbers:
DateTime.fromComponents("2024", "01", "15", "10", "30", "00");

// CORRECT:
DateTime.fromComponents(2024, 1, 15, 10, 30, 0);
```

Use `str()`, `num()`, or `bool()` for explicit type conversion when you have a value of
the wrong type.

---

## Quick Function Lookup

| Goal | Call |
|------|------|
| Parse JSON | `Json.parse(text)` → `Result<JsonValue, string>` |
| Serialize to JSON | `Json.stringify(value)` → `string` |
| Encode base64 | `Encoding.base64Encode(text)` → `string` |
| Decode base64 | `Encoding.base64Decode(encoded)` → `string` |
| Encode hex | `Encoding.hexEncode(text)` → `string` |
| Compile regex | `Regex.new(pattern)` → `Result<Regex, string>` |
| Test regex (one-off) | `Regex.test(pattern, text)` → `bool` |
| Match regex | `Regex.isMatch(re, text)` → `bool` |
| Find first match | `Regex.find(re, text)` → `Option<map>` |
| Find all matches | `Regex.findAll(re, text)` → `map[]` |
| Replace first match | `Regex.replace(re, text, repl)` → `string` |
| Replace all matches | `Regex.replaceAll(re, text, repl)` → `string` |
| Current time | `DateTime.now()` → `DateTime` |
| Parse ISO date | `DateTime.parseIso(text)` → `DateTime` |
| Format date | `DateTime.format(dt, fmt)` → `string` |
| Unix timestamp | `DateTime.toTimestamp(dt)` → `number` |
| Add time | `DateTime.addDays(dt, n)` → `DateTime` |
| Get type name | `reflect.typeOf(v)` → `string` |
| Check type | `reflect.isPrimitive(v)` → `bool` |
| Get struct fields | `reflect.fields(v)` → `string[]` |
| Deep compare | `reflect.deepEquals(a, b)` → `bool` |
| Gzip compress | `Gzip.compress(data)` → `number[]` |
| Gzip decompress | `Gzip.decompressString(bytes)` → `string` |
