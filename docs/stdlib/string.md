# String

String methods are available as instance methods on any `string` value using dot syntax, and also as global functions that take the string as the first argument. All operations are Unicode-aware unless noted otherwise.

---

## Instance Methods

### `.length -> number`

The number of Unicode characters in the string. Accessed as a property, not a method call.

```atlas
let n = "hello".length;
// n == 5

let n2 = "".length;
// n2 == 0
```

---

### `.split(separator: string) -> string[]`

Splits the string by `separator`. If `separator` is empty, returns an array of individual characters.

```atlas
let parts = "a,b,c".split(",");
// parts == ["a", "b", "c"]

let chars = "hi".split("");
// chars == ["h", "i"]
```

---

### `.trim() -> string`

Removes leading and trailing whitespace (Unicode-aware).

```atlas
let s = "  hello  ".trim();
// s == "hello"
```

---

### `.trimStart() -> string`

Removes only leading whitespace.

```atlas
let s = "  hello  ".trimStart();
// s == "hello  "
```

---

### `.trimEnd() -> string`

Removes only trailing whitespace.

```atlas
let s = "  hello  ".trimEnd();
// s == "  hello"
```

---

### `.toUpperCase() -> string`

Converts the string to uppercase (Unicode-aware).

```atlas
let s = "hello".toUpperCase();
// s == "HELLO"
```

---

### `.toLowerCase() -> string`

Converts the string to lowercase (Unicode-aware).

```atlas
let s = "HELLO".toLowerCase();
// s == "hello"
```

---

### `.indexOf(needle: string) -> Option<number>`

Returns `Some(index)` of the first occurrence of `needle`, or `None` if not found. An empty `needle` always returns `Some(0)`.

```atlas
let idx = "hello world".indexOf("world");
// idx == Some(6)

let missing = "hello".indexOf("xyz");
// missing == None
```

---

### `.lastIndexOf(needle: string) -> Option<number>`

Returns `Some(index)` of the last occurrence of `needle`, or `None` if not found. An empty `needle` returns `Some(length)`.

```atlas
let idx = "abcabc".lastIndexOf("a");
// idx == Some(3)
```

---

### `.includes(needle: string) -> bool`

Returns `true` if the string contains `needle`.

```atlas
let found = "hello world".includes("world");
// found == true
```

---

### `.startsWith(prefix: string) -> bool`

Returns `true` if the string starts with `prefix`.

```atlas
let ok = "hello".startsWith("hel");
// ok == true
```

---

### `.endsWith(suffix: string) -> bool`

Returns `true` if the string ends with `suffix`.

```atlas
let ok = "hello".endsWith("llo");
// ok == true
```

---

### `.substring(start: number, end: number) -> string`

Returns the substring from `start` (inclusive) to `end` (exclusive). Indices must be integers on UTF-8 character boundaries. Errors if `start > end` or indices are out of bounds.

```atlas
let sub = "hello world".substring(6, 11);
// sub == "world"
```

---

### `.charAt(index: number) -> Option<string>`

Returns `Some(character)` at the given index (Unicode character, not byte), or `None` if out of bounds. Index must be an integer.

```atlas
let ch = "hello".charAt(1);
// ch == Some("e")

let ch2 = "hello".charAt(99);
// ch2 == None
```

---

### `.repeat(count: number) -> string`

Returns the string repeated `count` times. `count` must be a non-negative integer. Maximum count is 1,000,000.

```atlas
let s = "ab".repeat(3);
// s == "ababab"

let s2 = "x".repeat(0);
// s2 == ""
```

---

### `.replace(search: string, replacement: string) -> string`

Replaces the **first** occurrence of `search` with `replacement`. If `search` is empty, the original string is returned unchanged.

```atlas
let s = "foo foo foo".replace("foo", "bar");
// s == "bar foo foo"
```

---

### `.replaceAll(search: string, replacement: string) -> string`

Replaces **all** occurrences of `search` with `replacement`. If `search` is empty, the original string is returned unchanged.

```atlas
let s = "foo foo foo".replaceAll("foo", "bar");
// s == "bar bar bar"
```

---

### `.padStart(length: number, fill: string) -> string`

Pads the start of the string with `fill` until the total length is `length`. If the string is already at or beyond `length`, it is returned unchanged. The fill string is cycled as needed.

```atlas
let s = "5".padStart(3, "0");
// s == "005"

let s2 = "hello".padStart(3, "x");
// s2 == "hello"  (already long enough)
```

---

### `.padEnd(length: number, fill: string) -> string`

Pads the end of the string with `fill` until the total length is `length`.

```atlas
let s = "hi".padEnd(5, ".");
// s == "hi..."
```

---

## Global String Functions

These are available without any import.

### `charCodeAt(str: string, index: number) -> number`

Returns the Unicode code point of the character at `index`. Errors if index is out of bounds.

```atlas
let code = charCodeAt("A", 0);
// code == 65
```

### `fromCharCode(...codes: number) -> string`

Converts one or more Unicode code points to a string.

```atlas
let s = fromCharCode(72, 101, 108, 108, 111);
// s == "Hello"
```

### `split(str: string, sep: string) -> string[]`

Global form of `.split()`.

```atlas
let parts = split("a-b-c", "-");
// parts == ["a", "b", "c"]
```

### `join(arr: string[], sep: string) -> string`

Joins an array of strings with `sep`. All elements must be strings.

```atlas
let s = join(["x", "y", "z"], " | ");
// s == "x | y | z"
```

### `trim(str: string) -> string`

Global form of `.trim()`.

### `trimStart(str: string) -> string`

Global form of `.trimStart()`.

### `trimEnd(str: string) -> string`

Global form of `.trimEnd()`.

### `indexOf(str: string, sub: string) -> Option<number>`

Global form of `.indexOf()`.

### `lastIndexOf(str: string, sub: string) -> Option<number>`

Global form of `.lastIndexOf()`.

### `includes(str: string, sub: string) -> bool`

Global form of `.includes()`.

### `startsWith(str: string, prefix: string) -> bool`

Global form of `.startsWith()`.

### `endsWith(str: string, suffix: string) -> bool`

Global form of `.endsWith()`.

### `toUpperCase(str: string) -> string`

Global form of `.toUpperCase()`.

### `toLowerCase(str: string) -> string`

Global form of `.toLowerCase()`.

### `substring(str: string, start: number, end: number) -> string`

Global form of `.substring()`.

### `repeat(str: string, count: number) -> string`

Global form of `.repeat()`.

### `replace(str: string, old: string, new: string) -> string`

Global form of `.replace()` (first occurrence only).

### `padStart(str: string, len: number, pad: string) -> string`

Global form of `.padStart()`.

### `padEnd(str: string, len: number, pad: string) -> string`

Global form of `.padEnd()`.
