# Encoding — Base64, Hex, and URL Encoding

Namespace: `Encoding` (PascalCase, D-049)

The `Encoding` namespace provides encoding and decoding for base64 (standard and URL-safe
variants), hexadecimal, and percent-encoded URLs. All functions use `Encoding.method()`
call syntax.

**Note:** `urlEncode` and `urlDecode` are only available when the runtime is built with the
`http` feature flag. `base64*` and `hex*` are always available.

---

## Import

No import required. `Encoding` is a built-in namespace.

---

## Base64 Functions

### `Encoding.base64Encode(data: string) -> string`

Encode a UTF-8 string to standard Base64 (RFC 4648, using `+` and `/`, with `=` padding).

```atlas
let encoded = Encoding.base64Encode("Hello, Atlas!");
// "SGVsbG8sIEF0bGFzIQ=="
```

---

### `Encoding.base64Decode(encoded: string) -> string`

Decode a standard Base64 string back to a UTF-8 string. Panics (runtime error) if the
input is not valid Base64 or if the decoded bytes are not valid UTF-8.

```atlas
let decoded = Encoding.base64Decode("SGVsbG8sIEF0bGFzIQ==");
// "Hello, Atlas!"
```

**Errors:**
- `InvalidStdlibArgument` if the input is not valid base64
- `InvalidStdlibArgument` if the decoded bytes are not valid UTF-8

---

### `Encoding.base64UrlEncode(data: string) -> string`

Encode a UTF-8 string to URL-safe Base64 (RFC 4648, using `-` and `_` instead of `+`
and `/`). Suitable for embedding in URLs and filenames.

```atlas
let token = Encoding.base64UrlEncode("user:pass?key=1&v=2");
```

---

### `Encoding.base64UrlDecode(encoded: string) -> string`

Decode a URL-safe Base64 string back to a UTF-8 string. Errors on invalid input or
non-UTF-8 decoded bytes.

```atlas
let raw = Encoding.base64UrlDecode(token);
```

---

## Hex Functions

### `Encoding.hexEncode(data: string) -> string`

Encode the raw bytes of a UTF-8 string as lowercase hexadecimal. Each byte becomes two
hex characters.

```atlas
let hex = Encoding.hexEncode("ABC");
// "414243"
```

---

### `Encoding.hexDecode(hex: string) -> string`

Decode a hexadecimal string back to a UTF-8 string. The input must be a valid hex string
with an even number of characters. Errors if input is not valid hex or decoded bytes are
not valid UTF-8.

```atlas
let text = Encoding.hexDecode("414243");
// "ABC"
```

**Errors:**
- `InvalidStdlibArgument` if the input is not valid hex
- `InvalidStdlibArgument` if the decoded bytes are not valid UTF-8

---

## URL Encoding Functions

These functions require the `http` feature flag.

### `Encoding.urlEncode(data: string) -> string`

Percent-encode a string for use in URLs. Encodes characters outside the URL-safe set.

```atlas
let encoded = Encoding.urlEncode("hello world & more");
// "hello%20world%20%26%20more"
```

---

### `Encoding.urlDecode(encoded: string) -> string`

Decode a percent-encoded URL string back to a plain string. Errors if the input contains
invalid percent-encoding sequences.

```atlas
let plain = Encoding.urlDecode("hello%20world");
// "hello world"
```

---

## Choosing the Right Variant

| Use case | Function |
|----------|----------|
| General binary-to-text encoding | `Encoding.base64Encode` / `Encoding.base64Decode` |
| Token/JWT signing fields, filenames | `Encoding.base64UrlEncode` / `Encoding.base64UrlDecode` |
| Hex digests, byte inspection | `Encoding.hexEncode` / `Encoding.hexDecode` |
| URL query string parameters | `Encoding.urlEncode` / `Encoding.urlDecode` |

---

## Common Patterns

### Round-trip encode/decode

```atlas
let original = "Hello, Atlas!";
let encoded = Encoding.base64Encode(original);
let decoded = Encoding.base64Decode(encoded);
// decoded == original
```

### Encoding a token for an HTTP header

```atlas
let credentials = "user:secret";
let token = "Basic " + Encoding.base64Encode(credentials);
// Use token in Authorization header
```

### Hex representation of arbitrary bytes

```atlas
let raw = "binary\x00data";
let hex = Encoding.hexEncode(raw);
console.log(hex);
```

### Building a URL query string

```atlas
let query = "search=" + Encoding.urlEncode("hello world + things")
          + "&page=" + Encoding.urlEncode("1");
```

---

## Error Behavior

| Function | Error condition | Result |
|----------|-----------------|--------|
| `base64Decode` | Invalid base64 input | `InvalidStdlibArgument` panic |
| `base64Decode` | Non-UTF-8 decoded bytes | `InvalidStdlibArgument` panic |
| `base64UrlDecode` | Invalid URL-safe base64 | `InvalidStdlibArgument` panic |
| `base64UrlDecode` | Non-UTF-8 decoded bytes | `InvalidStdlibArgument` panic |
| `hexDecode` | Invalid hex input | `InvalidStdlibArgument` panic |
| `hexDecode` | Non-UTF-8 decoded bytes | `InvalidStdlibArgument` panic |
| `urlDecode` | Invalid percent-encoding | `InvalidStdlibArgument` panic |
| `urlEncode` / `urlDecode` | `http` feature not compiled | Function not registered — call fails |

All encoding functions take exactly one string argument. Passing wrong arity or wrong type
raises an `InvalidStdlibArgument` error.
