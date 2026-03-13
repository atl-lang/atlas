# Regex — Regular Expressions

Namespace: `Regex` (PascalCase, D-049)

The `Regex` namespace provides compiled regular expression patterns with matching,
searching, capture group extraction, replacement, and splitting. Patterns are compiled
once and stored as a `Regex` value type for efficient reuse.

---

## Import

No import required. `Regex` is a built-in namespace.

---

## Pattern Compilation

### `Regex.new(pattern: string) -> Result<Regex, string>`

Compile a regular expression pattern. Returns `Ok(Regex)` if the pattern is valid, or
`Err(string)` with an error description if the pattern is invalid.

Patterns follow the Rust `regex` crate syntax, which is a subset of Perl-compatible
regular expressions with Unicode support. Lookaheads and backreferences are **not**
supported.

```atlas
let result = Regex.new("\\d+");
match result {
    Ok(pattern) => console.log("compiled"),
    Err(e) => console.error("bad pattern: " + e),
}

// Common idiom — unwrap when pattern is known valid:
let pattern = Regex.new("\\d+").unwrap();
```

---

### `Regex.newWithFlags(pattern: string, flags: string) -> Result<Regex, string>`

Compile a regular expression with modifier flags. The `flags` string may contain any
combination of:

| Flag | Behavior |
|------|----------|
| `i` | Case insensitive matching |
| `m` | Multi-line: `^` and `$` match line boundaries |
| `s` | Dot-all: `.` matches newline characters |
| `x` | Extended: ignore whitespace and allow `#` comments in pattern |

Returns `Err(string)` if any flag character is not recognized.

```atlas
let pattern = Regex.newWithFlags("hello", "i").unwrap();
// matches "HELLO", "Hello", "hello", etc.

let multi = Regex.newWithFlags("^line", "m").unwrap();
// ^ matches at the start of each line
```

---

### `Regex.escape(text: string) -> string`

Escape all regex metacharacters in a string so it can be safely used as a literal pattern.
Escaped characters: `. * + ? ^ $ ( ) [ ] { } | \`

```atlas
let literal = Regex.escape("hello.world");
// Returns "hello\\.world"
let pattern = Regex.new(literal).unwrap();
// Matches the literal string "hello.world"
```

---

### `Regex.test(pattern: string, text: string) -> bool`

Convenience function: compile a pattern and test a string in one call. Returns `false` on
compile error instead of propagating an error. Useful for quick one-off checks.

```atlas
let matches = Regex.test("\\d+", "hello123"); // true
let bad = Regex.test("[invalid", "test");       // false (compile error)
```

---

## Matching Functions

### `Regex.isMatch(regex: Regex, text: string) -> bool`

Test whether the pattern matches anywhere in the text. Returns `true` if there is at least
one match.

```atlas
let pattern = Regex.new("\\d+").unwrap();
let found = Regex.isMatch(pattern, "hello 42 world"); // true
let none = Regex.isMatch(pattern, "no digits here");  // false
```

---

### `Regex.find(regex: Regex, text: string) -> Option<{text: string, start: number, end: number}>`

Find the first match in the text. Returns `Some(map)` with the match details, or `None`
if no match is found.

The returned map contains:
- `text`: the matched substring
- `start`: byte offset of the start of the match
- `end`: byte offset of the end of the match (exclusive)

```atlas
let pattern = Regex.new("\\d+").unwrap();
let m = Regex.find(pattern, "hello 42 world");
if m.isSome() {
    let data = m.unwrap();
    let matched = data.get("text");   // "42"
    let start = data.get("start");    // 6
    let end = data.get("end");        // 8
}
```

---

### `Regex.findAll(regex: Regex, text: string) -> {text: string, start: number, end: number}[]`

Find all non-overlapping matches in the text. Returns an array of match maps. Returns an
empty array if no matches are found.

```atlas
let pattern = Regex.new("\\d+").unwrap();
let matches = Regex.findAll(pattern, "a1 b22 c333");
// [
//   {text: "1",   start: 1, end: 2},
//   {text: "22",  start: 4, end: 6},
//   {text: "333", start: 8, end: 11}
// ]
```

---

### `Regex.matchIndices(regex: Regex, text: string) -> number[][]`

Get the byte-index positions of all matches as `[start, end]` pairs. Returns an array of
two-element arrays.

```atlas
let pattern = Regex.new("\\d+").unwrap();
let indices = Regex.matchIndices(pattern, "a1b22c");
// [[1, 2], [3, 5]]
```

---

## Capture Groups

### `Regex.captures(regex: Regex, text: string) -> Option<string[]>`

Extract indexed capture groups from the first match. Returns `Some(array)` where:
- Index `0` is the full match
- Index `1`, `2`, etc. are the capture groups in order
- Optional groups that did not participate in the match are `null`

Returns `None` if there is no match.

```atlas
let pattern = Regex.new("(\\d+)-(\\w+)").unwrap();
let groups = Regex.captures(pattern, "123-abc");
if groups.isSome() {
    let g = groups.unwrap();
    // g[0] == "123-abc"  (full match)
    // g[1] == "123"      (group 1)
    // g[2] == "abc"      (group 2)
}
```

---

### `Regex.capturesNamed(regex: Regex, text: string) -> Option<HashMap<string, string>>`

Extract named capture groups from the first match. Named groups use the syntax
`(?P<name>pattern)`. Returns `Some(map)` with group name → matched string, or `None` if
there is no match. Groups that did not participate are mapped to `null`.

```atlas
let pattern = Regex.new("(?P<year>\\d{4})-(?P<month>\\d{2})-(?P<day>\\d{2})").unwrap();
let groups = Regex.capturesNamed(pattern, "Date: 2024-06-15");
if groups.isSome() {
    let g = groups.unwrap();
    let year = g.get("year");   // "2024"
    let month = g.get("month"); // "06"
    let day = g.get("day");     // "15"
}
```

---

## Replacement Functions

### `Regex.replace(regex: Regex, text: string, replacement: string) -> string`

Replace the **first** match with a replacement string. The replacement may reference
capture groups:

| Reference | Expands to |
|-----------|-----------|
| `$1`, `$2`, ... | Numbered capture group |
| `$&` | The entire match |
| `` $` `` | Text before the match |
| `$'` | Text after the match |

```atlas
let pattern = Regex.new("(\\d+)").unwrap();
let result = Regex.replace(pattern, "a1b2c3", "[$1]");
// "a[1]b2c3"  — only first match replaced
```

---

### `Regex.replaceAll(regex: Regex, text: string, replacement: string) -> string`

Replace **all** matches with a replacement string. Uses the same capture group reference
syntax as `Regex.replace()`.

```atlas
let pattern = Regex.new("(\\d+)").unwrap();
let result = Regex.replaceAll(pattern, "a1b2c3", "[$1]");
// "a[1]b[2]c[3]"
```

---

## Splitting Functions

### `Regex.split(regex: Regex, text: string) -> string[]`

Split a string at all positions where the pattern matches. Empty strings between adjacent
matches are preserved.

```atlas
let pattern = Regex.new(",").unwrap();
let parts = Regex.split(pattern, "a,b,,c");
// ["a", "b", "", "c"]
```

---

### `Regex.splitN(regex: Regex, text: string, limit: number) -> string[]`

Split a string at pattern matches with a maximum split count. `limit` is the maximum
number of splits — the resulting array has at most `limit + 1` elements. Passing `limit`
of `0` returns an empty array.

```atlas
let pattern = Regex.new(",").unwrap();
let parts = Regex.splitN(pattern, "a,b,c,d", 2);
// ["a", "b", "c,d"]  — split at most twice
```

---

## Regex Instance Methods

After compilation, a `Regex` value supports instance method syntax:

| Method | Equivalent namespace call |
|--------|--------------------------|
| `.test(text)` | `Regex.isMatch(re, text)` |
| `.find(text)` | `Regex.find(re, text)` |
| `.findAll(text)` | `Regex.findAll(re, text)` |
| `.replace(text, repl)` | `Regex.replace(re, text, repl)` |
| `.split(text)` | `Regex.split(re, text)` |

---

## Pattern Syntax Notes

Atlas regex uses the Rust `regex` crate. Key points:

- **Unicode-aware by default.** `.` matches any Unicode scalar value (except newline
  unless `s` flag is set).
- **No backreferences.** `\1` is not supported.
- **No lookaheads or lookbehinds.** `(?=...)` is not supported.
- **Named groups** use `(?P<name>...)` syntax.
- **Character classes** follow standard regex: `\d`, `\w`, `\s`, `[a-z]`, etc.
- **Anchors:** `^` (start of string/line), `$` (end of string/line), `\b` (word boundary).

---

## Common Patterns

### Validate an email address (basic)

```atlas
let re = Regex.new("^[^@]+@[^@]+\\.[^@]+$").unwrap();
let valid = Regex.isMatch(re, "user@example.com"); // true
```

### Extract all numbers from a string

```atlas
let re = Regex.new("\\d+").unwrap();
let matches = Regex.findAll(re, "3 cats and 12 dogs");
// [{text:"3",...}, {text:"12",...}]
```

### Parse a date string

```atlas
let re = Regex.new("(?P<y>\\d{4})-(?P<m>\\d{2})-(?P<d>\\d{2})").unwrap();
let groups = Regex.capturesNamed(re, "2024-06-15");
let year = groups.unwrap().get("y"); // "2024"
```

### Replace all whitespace runs with a single space

```atlas
let re = Regex.new("\\s+").unwrap();
let clean = Regex.replaceAll(re, "  hello   world  ", " ");
// " hello world "
```

### Case-insensitive word search

```atlas
let re = Regex.newWithFlags("atlas", "i").unwrap();
let found = Regex.isMatch(re, "Building with ATLAS today");
```

---

## Error Behavior

| Condition | Result |
|-----------|--------|
| Invalid pattern in `Regex.new()` | `Err(string)` — safe |
| Invalid pattern in `Regex.test()` | `false` — silent |
| Unknown flag in `Regex.newWithFlags()` | `Err("Invalid regex flag: '<c>'")` |
| Using Regex value where string expected | `TypeError` |
| No match in `Regex.find()` | `None` |
| No match in `Regex.captures()` | `None` |
| `limit` of `0` in `Regex.splitN()` | empty array `[]` |
