# Json — JSON Parsing and Serialization

Namespace: `Json` (PascalCase, D-049)

The `Json` namespace provides JSON parsing, serialization, validation, prettification, and
typed field extraction. All functions use `Json.method()` call syntax.

---

## Overview

Atlas represents parsed JSON as a distinct `JsonValue` type, separate from Atlas primitives.
`Json.parse()` returns a `Result<JsonValue, string>`, not a raw Atlas value. You must use
extraction methods (`.asString()`, `.asNumber()`, `Json.getString()`, etc.) to convert JSON
fields to Atlas native types.

**Import:** No import required. `Json` is a built-in namespace.

---

## Namespace Functions

### `Json.parse(text: string): Result<JsonValue, string>`

Parse a JSON string into a `JsonValue`. Returns `Ok(JsonValue)` on success, or
`Err("Invalid JSON: <detail>")` on malformed input.

**Type mapping:**

| JSON type | Atlas JsonValue |
|-----------|----------------|
| `null` | `JsonValue::Null` |
| `true` / `false` | `JsonValue::Bool` |
| number | `JsonValue::Number` |
| string | `JsonValue::String` |
| array | `JsonValue::Array` |
| object | `JsonValue::Object` |

```atlas
let result = Json.parse('{"name":"Atlas","version":3}');
match result {
    Ok(data) => console.log(Json.getString(data, "name").unwrapOr("?")),
    Err(e) => console.error("Parse failed: " + e),
}
```

---

### `Json.stringify(value: any): string`

Serialize any Atlas value to a compact JSON string. Returns the JSON with no extra
whitespace. Detects circular references. Functions cannot be serialized and cause a
runtime error.

```atlas
let json = Json.stringify(42);          // "42"
let json2 = Json.stringify("hello");    // "\"hello\""
let json3 = Json.stringify([1, 2, 3]); // "[1,2,3]"
```

---

### `Json.isValid(text: string): bool`

Check if a string is valid JSON without allocating a full parse result. More efficient
than `Json.parse()` when you only need a validity check.

```atlas
let ok = Json.isValid('{"x":1}');    // true
let bad = Json.isValid("{bad json}"); // false
```

---

### `Json.prettify(text: string, indent: number): string`

Format a JSON string with indentation. `indent` is the number of spaces per level.
Input must be valid JSON.

```atlas
let raw = '{"a":1,"b":{"c":2}}';
let pretty = Json.prettify(raw, 2);
// {
//   "a": 1,
//   "b": {
//     "c": 2
//   }
// }
```

---

### `Json.minify(text: string): string`

Remove all extraneous whitespace from a JSON string, producing the most compact
valid representation.

```atlas
let spaced = '{ "a" : 1 , "b" : 2 }';
let compact = Json.minify(spaced); // '{"a":1,"b":2}'
```

---

### `Json.keys(json: JsonValue): string[]`

Return the top-level key names from a JSON object value. Returns an empty array if the
`JsonValue` is not an object.

```atlas
let data = Json.parse('{"x":1,"y":2}').unwrap();
let keys = Json.keys(data); // ["x", "y"]
```

---

### `Json.getString(json: JsonValue, key: string): Option<string>`

Safely extract a string field from a JSON object. Returns `None` if the key does not
exist or if the value at that key is not a JSON string.

```atlas
let data = Json.parse('{"name":"Atlas"}').unwrap();
let name = Json.getString(data, "name").unwrapOr("unknown"); // "Atlas"
let missing = Json.getString(data, "nope").unwrapOr("default"); // "default"
```

---

### `Json.getNumber(json: JsonValue, key: string): Option<number>`

Safely extract a number field from a JSON object. Returns `None` if the key does not
exist or if the value is not a JSON number.

```atlas
let data = Json.parse('{"count":42}').unwrap();
let count = Json.getNumber(data, "count").unwrapOr(0); // 42
```

---

### `Json.getBool(json: JsonValue, key: string): Option<bool>`

Safely extract a boolean field from a JSON object. Returns `None` if the key does not
exist or if the value is not a JSON boolean.

```atlas
let data = Json.parse('{"active":true}').unwrap();
let active = Json.getBool(data, "active").unwrapOr(false); // true
```

---

### `Json.getArray(json: JsonValue, key: string): Option<JsonValue[]>`

Safely extract an array field from a JSON object. Returns `None` if the key does not
exist or if the value is not a JSON array. Each element of the returned array is a
`JsonValue` — extract further with the appropriate methods.

```atlas
let data = Json.parse('{"tags":["a","b","c"]}').unwrap();
let tags = Json.getArray(data, "tags"); // Some([JsonValue, JsonValue, JsonValue])
```

---

### `Json.getObject(json: JsonValue, key: string): Option<JsonValue>`

Safely extract a nested JSON object field. Returns `None` if the key does not exist or
if the value is not a JSON object.

```atlas
let data = Json.parse('{"meta":{"version":1}}').unwrap();
let meta = Json.getObject(data, "meta"); // Some(JsonValue::Object)
if meta.isSome() {
    let ver = Json.getNumber(meta.unwrap(), "version").unwrapOr(0);
}
```

---

### `Json.isNull(json: JsonValue, key: string): bool`

Check if a specific key in a JSON object holds a null value.

```atlas
let data = Json.parse('{"x":null,"y":1}').unwrap();
let nullish = Json.isNull(data, "x"); // true
let normal = Json.isNull(data, "y");  // false
```

---

## JsonValue Instance Methods

After parsing, a `JsonValue` exposes these instance methods via method dispatch:

| Method | Signature | Description |
|--------|-----------|-------------|
| `.asString()` | `(): string` | Extract string — panics if wrong type |
| `.asNumber()` | `(): number` | Extract number — panics if wrong type |
| `.asBool()` | `(): bool` | Extract bool — panics if wrong type |

Prefer the `Json.getString()` / `Json.getNumber()` / `Json.getBool()` namespace functions
for safe extraction that returns `Option<T>` instead of panicking.

---

## Type Reference

| Type | Description |
|------|-------------|
| `JsonValue` | Opaque JSON value — use extraction methods to get Atlas native types |
| `Result<JsonValue, string>` | Return type of `Json.parse()` |
| `Option<T>` | Return type of `Json.getString()`, `Json.getNumber()`, etc. |

---

## Common Patterns

### Parse, extract, and use fields

```atlas
let raw = '{"id":1,"label":"hello","enabled":true}';
match Json.parse(raw) {
    Ok(data) => {
        let id = Json.getNumber(data, "id").unwrapOr(0);
        let label = Json.getString(data, "label").unwrapOr("");
        let enabled = Json.getBool(data, "enabled").unwrapOr(false);
        console.log(id.toString() + ": " + label);
    }
    Err(e) => console.error("JSON error: " + e),
}
```

### Validate before processing

```atlas
fn processInput(raw: string): void {
    if !Json.isValid(raw) {
        console.error("Rejecting malformed JSON");
        return;
    }
    let data = Json.parse(raw).unwrap();
    // ...
}
```

### Nested object extraction

```atlas
let raw = '{"user":{"name":"Alice","age":30}}';
let root = Json.parse(raw).unwrap();
let user = Json.getObject(root, "user").unwrap();
let name = Json.getString(user, "name").unwrapOr("?");
let age = Json.getNumber(user, "age").unwrapOr(0);
```

### Prettify for logging / debugging

```atlas
let raw = '{"a":1,"b":{"c":3}}';
console.log(Json.prettify(raw, 2));
```

---

## Error Behavior

| Situation | Result |
|-----------|--------|
| Malformed JSON in `Json.parse()` | `Err("Invalid JSON: <serde detail>")` |
| Wrong type in `.asString()` etc. | Runtime `TypeError` — panics |
| Missing key in `Json.getString()` etc. | `None` — safe |
| Circular reference in `Json.stringify()` | Runtime error |
| Serializing a function value | Runtime error |
| Invalid `indent` in `Json.prettify()` | RuntimeError (must be non-negative integer) |
