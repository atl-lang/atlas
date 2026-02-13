# Atlas Standard Library API Reference

**Purpose:** Complete API reference for Atlas standard library functions
**Audience:** AI agents and developers
**Status:** Growing - phases add functions as implemented

---

## Table of Contents
1. [Prelude (Built-in Functions)](#prelude-built-in-functions)
2. [String Functions](#string-functions)
3. [Array Functions](#array-functions)
4. [Math Functions](#math-functions)
5. [JSON Functions](#json-functions)
6. [File I/O Functions](#file-io-functions)
7. [Collection Functions](#collection-functions)
8. [Regex Functions](#regex-functions)
9. [DateTime Functions](#datetime-functions)
10. [Network Functions](#network-functions)

---

## Prelude (Built-in Functions)

**Note:** These functions are always in scope (no import needed).
**See also:** `docs/RUNTIME.md` for runtime behavior details.

### print
**Signature:** `print(value: string|number|bool|null) -> void`
**Behavior:** Writes value to stdout with newline
**Example:** `print("hello");` outputs `hello`
**Errors:** AT0102 if wrong type

### len
**Signature:** `len(value: string|T[]) -> number`
**Behavior:** Returns length (Unicode scalars for strings, element count for arrays)
**Example:** `len("🌟")` returns `1`
**Errors:** AT0102 if wrong type

### str
**Signature:** `str(value: number|bool|null) -> string`
**Behavior:** Converts value to string representation
**Example:** `str(42)` returns `"42"`
**Errors:** AT0102 if wrong type

---

## String Functions

**Implementation:** `crates/atlas-runtime/src/stdlib/string.rs`
**Phase:** phases/stdlib/phase-01-complete-string-api.md

### Core Operations

#### split
**Signature:** `split(s: string, separator: string) -> string[]`
**Behavior:** Divides string by separator, returns array of parts
**Example:** `split("a,b,c", ",")` returns `["a", "b", "c"]`
**Errors:** AT0102 if wrong types

#### join
**Signature:** `join(parts: string[], separator: string) -> string`
**Behavior:** Combines array of strings with separator
**Example:** `join(["a", "b"], ",")` returns `"a,b"`
**Errors:** AT0102 if wrong types

#### trim
**Signature:** `trim(s: string) -> string`
**Behavior:** Removes leading and trailing whitespace (Unicode-aware)
**Example:** `trim("  hello  ")` returns `"hello"`
**Errors:** AT0102 if wrong type

#### trimStart
**Signature:** `trimStart(s: string) -> string`
**Behavior:** Removes leading whitespace (Unicode-aware)
**Example:** `trimStart("  hello")` returns `"hello"`
**Errors:** AT0102 if wrong type

#### trimEnd
**Signature:** `trimEnd(s: string) -> string`
**Behavior:** Removes trailing whitespace (Unicode-aware)
**Example:** `trimEnd("hello  ")` returns `"hello"`
**Errors:** AT0102 if wrong type

### Search Operations

#### indexOf
**Signature:** `indexOf(s: string, search: string) -> number`
**Behavior:** Returns index of first occurrence, -1 if not found
**Example:** `indexOf("hello", "ll")` returns `2`
**Errors:** AT0102 if wrong types

#### lastIndexOf
**Signature:** `lastIndexOf(s: string, search: string) -> number`
**Behavior:** Returns index of last occurrence, -1 if not found
**Example:** `lastIndexOf("hello", "l")` returns `3`
**Errors:** AT0102 if wrong types

#### includes
**Signature:** `includes(s: string, search: string) -> bool`
**Behavior:** Returns true if search string is found
**Example:** `includes("hello", "ll")` returns `true`
**Errors:** AT0102 if wrong types

### Transformation

#### toUpperCase
**Signature:** `toUpperCase(s: string) -> string`
**Behavior:** Converts to uppercase (Unicode-aware)
**Example:** `toUpperCase("hello")` returns `"HELLO"`
**Errors:** AT0102 if wrong type

#### toLowerCase
**Signature:** `toLowerCase(s: string) -> string`
**Behavior:** Converts to lowercase (Unicode-aware)
**Example:** `toLowerCase("HELLO")` returns `"hello"`
**Errors:** AT0102 if wrong type

#### substring
**Signature:** `substring(s: string, start: number, end: number) -> string`
**Behavior:** Extracts substring from start to end (UTF-8 boundary safe)
**Example:** `substring("hello", 1, 4)` returns `"ell"`
**Errors:** AT0102 if wrong types, AT0103 if invalid indices

#### charAt
**Signature:** `charAt(s: string, index: number) -> string`
**Behavior:** Returns grapheme cluster at index (not byte)
**Example:** `charAt("hello", 0)` returns `"h"`
**Errors:** AT0102 if wrong types, AT0103 if out of bounds

#### repeat
**Signature:** `repeat(s: string, count: number) -> string`
**Behavior:** Repeats string count times (max count to prevent abuse)
**Example:** `repeat("ha", 3)` returns `"hahaha"`
**Errors:** AT0102 if wrong types, AT0104 if count too large

#### replace
**Signature:** `replace(s: string, search: string, replacement: string) -> string`
**Behavior:** Replaces first occurrence of search with replacement
**Example:** `replace("hello", "l", "L")` returns `"heLlo"`
**Errors:** AT0102 if wrong types

### Formatting

#### padStart
**Signature:** `padStart(s: string, length: number, fill: string) -> string`
**Behavior:** Pads start with fill to reach target length
**Example:** `padStart("5", 3, "0")` returns `"005"`
**Errors:** AT0102 if wrong types

#### padEnd
**Signature:** `padEnd(s: string, length: number, fill: string) -> string`
**Behavior:** Pads end with fill to reach target length
**Example:** `padEnd("5", 3, "0")` returns `"500"`
**Errors:** AT0102 if wrong types

#### startsWith
**Signature:** `startsWith(s: string, prefix: string) -> bool`
**Behavior:** Returns true if string starts with prefix
**Example:** `startsWith("hello", "he")` returns `true`
**Errors:** AT0102 if wrong types

#### endsWith
**Signature:** `endsWith(s: string, suffix: string) -> bool`
**Behavior:** Returns true if string ends with suffix
**Example:** `endsWith("hello", "lo")` returns `true`
**Errors:** AT0102 if wrong types

---

## Array Functions

**Implementation:** `crates/atlas-runtime/src/stdlib/array.rs`
**Phase:** phases/stdlib/phase-02-complete-array-api.md

_Phases will populate this section with array manipulation functions_

---

## Math Functions

**Implementation:** `crates/atlas-runtime/src/stdlib/math.rs`
**Phase:** phases/stdlib/phase-03-complete-math-api.md

_Phases will populate this section with math functions and constants_

---

## JSON Functions

**Implementation:** `crates/atlas-runtime/src/stdlib/json.rs`
**Phase:** phases/stdlib/phase-04-json-type-utilities.md

_Phases will populate this section with JSON parsing and serialization functions_

---

## File I/O Functions

**Implementation:** `crates/atlas-runtime/src/stdlib/io.rs`
**Phase:** phases/stdlib/phase-05-complete-file-io-api.md

_Phases will populate this section with file I/O functions_

---

## Collection Functions

**Implementation:** `crates/atlas-runtime/src/stdlib/collections.rs`
**Phase:** phases/stdlib/phase-07-collections.md

_Phases will populate this section with HashMap, HashSet, Queue, Stack APIs_

---

## Regex Functions

**Implementation:** `crates/atlas-runtime/src/stdlib/regex.rs`
**Phase:** phases/stdlib/phase-08-regex.md

_Phases will populate this section with regex functions_

---

## DateTime Functions

**Implementation:** `crates/atlas-runtime/src/stdlib/datetime.rs`
**Phase:** phases/stdlib/phase-09-datetime.md

_Phases will populate this section with date/time functions_

---

## Network Functions

**Implementation:** `crates/atlas-runtime/src/stdlib/network.rs`
**Phase:** phases/stdlib/phase-10-network-http.md

_Phases will populate this section with HTTP client functions_

---

## API Documentation Standards

**For AI agents adding new functions:**

1. **Format:** Follow the pattern above (Signature → Behavior → Example → Errors)
2. **Signatures:** Be explicit about types
3. **Behavior:** One sentence summary, key details below
4. **Examples:** Show common usage, not edge cases
5. **Errors:** List error codes, not full messages
6. **No code dumps:** This is API reference, not implementation
7. **Token efficiency:** Keep descriptions concise

**Example entry:**
```markdown
#### functionName
**Signature:** `functionName(arg: type) -> returnType`
**Behavior:** What it does in one sentence
**Example:** `functionName("input")` returns `"output"`
**Errors:** AT0102 if wrong type, AT0103 if invalid input
```
