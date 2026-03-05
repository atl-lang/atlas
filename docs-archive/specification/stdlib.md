# Atlas Standard Library Reference

**Version:** 0.3
**Status:** Comprehensive API reference

This document describes the Atlas standard library, organized by category.

---

## ⚠️ Current Reality vs Future Direction

**This spec describes the ASPIRATIONAL API.** The current implementation differs:

### What Works Today (v0.3)

```atlas
// Arrays - use prefixed global functions
let arr = [1, 2, 3];
let arr2 = arrayPush(arr, 4);      // NOT push(arr, 4)
let n = len(arr);                   // NOT length(arr) or arr.length()

// HashMaps - use prefixed global functions
let m = hashMapNew();
hashMapPut(m, "key", value);        // NOT m.put("key", value)
let v = hashMapGet(m, "key");       // NOT m.get("key")

// JSON
let obj = parseJSON(str);           // NOT JSON.parse(str)
let s = stringifyJSON(obj);         // NOT JSON.stringify(obj)
```

### Future Direction (H-065)

Atlas will adopt professional language patterns:

1. **Method syntax:** `arr.push(x)`, `map.get(key)`, `str.trim()`
2. **Namespaced imports:** `import { HashMap } from "std/collections"`
3. **Clean function names:** No type prefixes

**Timeline:** Targeted for v0.4+. See tracking issue H-065.

---

For a detailed guide to the testing system, see [Testing System](testing.md).

---

## Table of Contents

1. [Testing Primitives](#testing-primitives)
2. [Type System](#type-system)
3. [Array Operations](#array-operations)
4. [String Operations](#string-operations)
5. [Math Functions](#math-functions)
6. [I/O Operations](#io-operations)
7. [JSON Operations](#json-operations)
8. [Process Management](#process-management)
9. [Filesystem Watching](#filesystem-watching)

---

## Testing Primitives

Atlas provides 13 assertion functions for test code. See [Testing System](testing.md) for comprehensive test documentation.

### Basic Assertions (2 functions)

| Function | Signature | Returns |
|----------|-----------|---------|
| `assert` | `(condition: bool, message: string) -> void` | void |
| `assertFalse` | `(condition: bool, message: string) -> void` | void |

### Equality Assertions (2 functions)

| Function | Signature | Returns |
|----------|-----------|---------|
| `assertEqual` | `(actual: T, expected: T) -> void` | void |
| `assertNotEqual` | `(actual: T, expected: T) -> void` | void |

Uses deep equality: arrays are compared element-by-element (not by reference).

### Result Assertions (2 functions)

| Function | Signature | Returns |
|----------|-----------|---------|
| `assertOk` | `(result: Result<T, E>) -> T` | unwrapped Ok value |
| `assertErr` | `(result: Result<T, E>) -> E` | unwrapped Err value |

### Option Assertions (2 functions)

| Function | Signature | Returns |
|----------|-----------|---------|
| `assertSome` | `(option: Option<T>) -> T` | unwrapped Some value |
| `assertNone` | `(option: Option<T>) -> void` | void |

### Collection Assertions (3 functions)

| Function | Signature | Returns |
|----------|-----------|---------|
| `assertContains` | `(array: array, value: T) -> void` | void |
| `assertEmpty` | `(array: array) -> void` | void |
| `assertLength` | `(array: array, expected: number) -> void` | void |

### Error Assertions (2 functions)

| Function | Signature | Notes |
|----------|-----------|-------|
| `assertThrows` | `(fn: NativeFunction) -> void` | Works with Rust closures only |
| `assertNoThrow` | `(fn: NativeFunction) -> void` | Works with Rust closures only |

---

## Type System

### Type Checking (8 functions)

| Function | Signature | Description |
|----------|-----------|-------------|
| `type` | `(value: T) -> string` | Get type name as string |
| `isString` | `(value: T) -> bool` | Check if value is string |
| `isNumber` | `(value: T) -> bool` | Check if value is number |
| `isBool` | `(value: T) -> bool` | Check if value is bool |
| `isNull` | `(value: T) -> bool` | Check if value is null |
| `isArray` | `(value: T) -> bool` | Check if value is array |
| `isFunction` | `(value: T) -> bool` | Check if value is function |
| `isObject` | `(value: T) -> bool` | Check if value is object/record |

### Type Conversion (7 functions)

| Function | Signature | Description |
|----------|-----------|-------------|
| `str` | `(value: T) -> string` | Convert to string |
| `num` | `(value: T) -> number` | Convert to number |
| `bool` | `(value: T) -> bool` | Convert to bool |
| `parseInt` | `(string: string) -> Option<number>` | Parse integer from string |
| `parseFloat` | `(string: string) -> Option<number>` | Parse float from string |
| `toString` | `(value: T) -> string` | Convert to string (alias for str) |
| `toNumber` | `(value: T) -> number` | Convert to number (alias for num) |

### Option Operations (4 functions)

| Function | Signature | Description |
|----------|-----------|-------------|
| `Some` | `(value: T) -> Option<T>` | Wrap value in Some |
| `None` | `() -> Option<T>` | Create empty Option |
| `unwrap` | `(option: Option<T>) -> T` | Extract value (panics if None) |
| `unwrapOr` | `(option: Option<T>, default: T) -> T` | Extract value or return default |

### Result Operations (4 functions)

| Function | Signature | Description |
|----------|-----------|-------------|
| `Ok` | `(value: T) -> Result<T, E>` | Wrap value in Ok |
| `Err` | `(error: E) -> Result<T, E>` | Wrap error in Err |
| `unwrap` | `(result: Result<T, E>) -> T` | Extract value (panics if Err) |
| `unwrapOr` | `(result: Result<T, E>, default: T) -> T` | Extract value or return default |

---

## Array Operations

### Core Operations (7 functions)

| Function | Signature | Description |
|----------|-----------|-------------|
| `length` | `(array: array) -> number` | Get array length |
| `push` | `(array: array, element: T) -> array` | Append element |
| `pop` | `(array: array) -> [T, array]` | Remove last element |
| `shift` | `(array: array) -> [T, array]` | Remove first element |
| `unshift` | `(array: array, element: T) -> array` | Prepend element |
| `reverse` | `(array: array) -> array` | Reverse elements |
| `sort` | `(array: array) -> array` | Sort by natural order |

### Search Operations (3 functions)

| Function | Signature | Description |
|----------|-----------|-------------|
| `indexOf` | `(array: array, search: T) -> Option<number>` | Find first index of element |
| `lastIndexOf` | `(array: array, search: T) -> Option<number>` | Find last index of element |
| `includes` | `(array: array, search: T) -> bool` | Check if array contains element |

### Slicing & Flattening (2 functions)

| Function | Signature | Description |
|----------|-----------|-------------|
| `slice` | `(array: array, start: number, end: number) -> array` | Extract subarray |
| `flatten` | `(array: array) -> array` | Flatten one level |

### Higher-Order Operations (3 functions)

| Function | Signature | Description |
|----------|-----------|-------------|
| `map` | `(array: array, fn: (T) -> U) -> array` | Transform each element |
| `filter` | `(array: array, fn: (T) -> bool) -> array` | Keep matching elements |
| `reduce` | `(array: array, fn: (A, T) -> A, initial: A) -> A` | Fold array to single value |

### Concatenation (1 function)

| Function | Signature | Description |
|----------|-----------|-------------|
| `concat` | `(array1: array, array2: array) -> array` | Concatenate two arrays |

**Note:** Array concatenation is also supported via `+` operator: `[1, 2] + [3, 4]` → `[1, 2, 3, 4]`

---

## String Operations

### String Inspection (3 functions)

| Function | Signature | Description |
|----------|-----------|-------------|
| `length` | `(string: string) -> number` | Get string length (Unicode scalar count) |
| `charAt` | `(string: string, index: number) -> string` | Get character at index |
| `charCodeAt` | `(string: string, index: number) -> number` | Get Unicode code point |

### String Search (4 functions)

| Function | Signature | Description |
|----------|-----------|-------------|
| `indexOf` | `(string: string, search: string) -> Option<number>` | Find first index of substring |
| `lastIndexOf` | `(string: string, search: string) -> Option<number>` | Find last index of substring |
| `includes` | `(string: string, search: string) -> bool` | Check if contains substring |
| `startsWith` | `(string: string, prefix: string) -> bool` | Check if starts with prefix |

### String Transformation (6 functions)

| Function | Signature | Description |
|----------|-----------|-------------|
| `substring` | `(string: string, start: number, end: number) -> string` | Extract substring |
| `slice` | `(string: string, start: number, end: number) -> string` | Extract slice |
| `toUpperCase` | `(string: string) -> string` | Convert to uppercase |
| `toLowerCase` | `(string: string) -> string` | Convert to lowercase |
| `trim` | `(string: string) -> string` | Remove leading/trailing whitespace |
| `split` | `(string: string, separator: string) -> array` | Split into array |

### String Building (1 function)

| Function | Signature | Description |
|----------|-----------|-------------|
| `repeat` | `(string: string, count: number) -> string` | Repeat string N times |

---

## Math Functions

### Constants (3 values)

| Name | Value | Description |
|------|-------|-------------|
| `Math.PI` | 3.14159... | π (pi) |
| `Math.E` | 2.71828... | e (Euler's number) |
| `Math.TAU` | 6.28318... | 2π (tau) |

### Basic Math (6 functions)

| Function | Signature | Description |
|----------|-----------|-------------|
| `abs` | `(x: number) -> number` | Absolute value |
| `min` | `(x: number, y: number) -> number` | Minimum of two numbers |
| `max` | `(x: number, y: number) -> number` | Maximum of two numbers |
| `floor` | `(x: number) -> number` | Floor (round down) |
| `ceil` | `(x: number) -> number` | Ceiling (round up) |
| `round` | `(x: number) -> number` | Round to nearest integer |

### Trigonometry (6 functions)

| Function | Signature | Description |
|----------|-----------|-------------|
| `sin` | `(x: number) -> number` | Sine (radians) |
| `cos` | `(x: number) -> number` | Cosine (radians) |
| `tan` | `(x: number) -> number` | Tangent (radians) |
| `asin` | `(x: number) -> number` | Arcsine |
| `acos` | `(x: number) -> number` | Arccosine |
| `atan` | `(x: number) -> number` | Arctangent |

### Exponential & Logarithm (4 functions)

| Function | Signature | Description |
|----------|-----------|-------------|
| `sqrt` | `(x: number) -> number` | Square root |
| `pow` | `(x: number, y: number) -> number` | x to power y |
| `exp` | `(x: number) -> number` | e to power x |
| `log` | `(x: number) -> number` | Natural logarithm |

### Random (2 functions)

| Function | Signature | Description |
|----------|-----------|-------------|
| `random` | `() -> number` | Random float [0, 1) |
| `randomInt` | `(max: number) -> number` | Random integer [0, max) |

---

## I/O Operations

### Console Output (1 function)

| Function | Signature | Description |
|----------|-----------|-------------|
| `print` | `(value: T) -> void` | Print to stdout |

### File Operations (8 functions)

| Function | Signature | Description |
|----------|-----------|-------------|
| `readFile` | `(path: string) -> Result<string, string>` | Read file contents |
| `writeFile` | `(path: string, content: string) -> Result<null, string>` | Write file (overwrites) |
| `appendFile` | `(path: string, content: string) -> Result<null, string>` | Append to file |
| `fileExists` | `(path: string) -> bool` | Check if file exists |
| `readDir` | `(path: string) -> Result<array, string>` | List directory contents |
| `createDir` | `(path: string) -> Result<null, string>` | Create directory |
| `removeFile` | `(path: string) -> Result<null, string>` | Delete file |
| `removeDir` | `(path: string) -> Result<null, string>` | Delete directory |

### Path Operations (2 functions)

| Function | Signature | Description |
|----------|-----------|-------------|
| `pathJoin` | `(parts: array) -> string` | Join path components |
| `fileInfo` | `(path: string) -> Result<object, string>` | Get file metadata |

---

## JSON Operations

### JSON Parsing (2 functions)

| Function | Signature | Description |
|----------|-----------|-------------|
| `parseJson` | `(text: string) -> Result<json, string>` | Parse JSON string |
| `stringifyJson` | `(value: json) -> Result<string, string>` | Stringify to JSON |

### JSON Access (2 functions)

| Function | Signature | Description |
|----------|-----------|-------------|
| `jsonGet` | `(json: json, key: string) -> json` | Safe get with null default |
| `jsonHas` | `(json: json, key: string) -> bool` | Check if key exists |

---

## Process Management

### Process Spawning (3 functions)

| Function | Signature | Description |
|----------|-----------|-------------|
| `spawnProcess` | `(command: array) -> ProcessHandle` | Spawn child process |
| `exec` | `(command: string) -> Result<string, string>` | Execute command synchronously |
| `shell` | `(command: string) -> Result<string, string>` | Execute shell command |

### Process I/O (3 functions)

| Function | Signature | Description |
|----------|-----------|-------------|
| `processStdin` | `(handle: ProcessHandle, data: string) -> void` | Write to process stdin |
| `processStdout` | `(handle: ProcessHandle) -> Promise<string>` | Read from stdout |
| `processStderr` | `(handle: ProcessHandle) -> Promise<string>` | Read from stderr |

### Process Control (3 functions)

| Function | Signature | Description |
|----------|-----------|-------------|
| `processWait` | `(handle: ProcessHandle) -> Promise<number>` | Wait for process exit |
| `processKill` | `(handle: ProcessHandle) -> void` | Terminate process |
| `processIsRunning` | `(handle: ProcessHandle) -> bool` | Check if process is alive |

### Environment (3 functions)

| Function | Signature | Description |
|----------|-----------|-------------|
| `getEnv` | `(name: string) -> Option<string>` | Get environment variable |
| `setEnv` | `(name: string, value: string) -> void` | Set environment variable |
| `getPid` | `() -> number` | Get current process ID |

---

## Filesystem Watching

### File Watching (1 function)

| Function | Signature | Description |
|----------|-----------|-------------|
| `watchPath` | `(path: string) -> Promise<object>` | Watch file/directory for changes |

**Returns:** A Promise that yields change events with `kind` and `path` fields.

**Example:**
```atlas
let watcher = watchPath("/path/to/file");
let event = awaitAsync(watcher);  // Yields: { kind: "modify", path: "..." }
```

---

## HashMap Operations

### HashMap Creation & Access (4 functions)

| Function | Signature | Description |
|----------|-----------|-------------|
| `hashMapNew` | `() -> HashMap<string, T>` | Create empty HashMap |
| `hashMapGet` | `(map: HashMap, key: string) -> Option<T>` | Get value by key |
| `hashMapSet` | `(map: HashMap, key: string, value: T) -> HashMap` | Set key-value pair |
| `hashMapDelete` | `(map: HashMap, key: string) -> HashMap` | Remove key |

### HashMap Inspection (3 functions)

| Function | Signature | Description |
|----------|-----------|-------------|
| `hashMapKeys` | `(map: HashMap) -> array` | Get all keys |
| `hashMapValues` | `(map: HashMap) -> array` | Get all values |
| `hashMapSize` | `(map: HashMap) -> number` | Get number of entries |

**Note:** HashMap mutations return the updated map (copy-on-write semantics).

---

## Summary by Category

| Category | Function Count |
|----------|----------------|
| Testing Primitives | 13 |
| Type System | 19 |
| Array Operations | 15 |
| String Operations | 13 |
| Math Functions | 18 |
| I/O Operations | 10 |
| JSON Operations | 4 |
| Process Management | 9 |
| Filesystem Watching | 1 |
| HashMap Operations | 7 |
| **Total** | **109** |

---

## See Also

- [Testing System](testing.md) - Comprehensive test guide
- [Language Semantics](language-semantics.md) - Type rules and execution model
- [Diagnostic System](diagnostic-system.md) - Error codes and messages
