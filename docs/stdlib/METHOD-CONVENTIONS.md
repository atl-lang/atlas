# Method Conventions — Atlas Standard Library

This document describes Atlas method naming conventions, which methods are available on
built-in types, and how chaining patterns work.

---

## Naming Conventions

### Casing

- All method names use **camelCase**: `push`, `indexOf`, `toUpperCase`, `addDays`, `isMatch`
- No underscores in public method names
- Acronyms follow normal camelCase rules: `toIso`, `toRfc3339`, `toUtc`, `isOk`

### Verb Prefixes

| Prefix | Meaning | Examples |
|--------|---------|---------|
| `is` / `has` | Boolean predicate | `isSome`, `isOk`, `isEmpty`, `isNone`, `hasField` |
| `to` | Convert to another type or format | `toString`, `toFixed`, `toIso`, `toTimestamp` |
| `from` | Constructor from data | `fromTimestamp`, `fromComponents` |
| `parse` | Parse from string | `parseIso`, `parseRfc3339` |
| `add` | Non-mutating arithmetic (returns new value) | `addDays`, `addHours`, `addSeconds` |
| `get` | Safe field access returning Option | `getString`, `getNumber`, `getArray` |
| `as` | Typed cast that panics on wrong type | `asString`, `asNumber`, `asBool` |
| `find` | Search returning Option or array | `find`, `findAll`, `findIndex` |
| `map` / `filter` / `reduce` | Higher-order collection transforms | standard |
| `unwrap` / `unwrapOr` | Extract from Option/Result | standard |

---

## Built-in Type Methods

### `number`

| Method | Signature | Description |
|--------|-----------|-------------|
| `.toString()` | `() -> string` | Convert to decimal string |
| `.toFixed(digits)` | `(number) -> string` | Format with fixed decimal places |
| `.toInt()` | `() -> number` | Truncate fractional part |

```atlas
let n = 3.14159;
n.toString();    // "3.14159"
n.toFixed(2);    // "3.14"
n.toInt();       // 3
```

---

### `bool`

| Method | Signature | Description |
|--------|-----------|-------------|
| `.toString()` | `() -> string` | `"true"` or `"false"` |

---

### `string`

| Method | Signature | Description |
|--------|-----------|-------------|
| `.length()` | `() -> number` | Number of Unicode characters |
| `.charAt(index)` | `(number) -> Option<string>` | Character at index |
| `.indexOf(sub)` | `(string) -> Option<number>` | First occurrence, or None |
| `.lastIndexOf(sub)` | `(string) -> Option<number>` | Last occurrence, or None |
| `.includes(sub)` | `(string) -> bool` | Whether substring is present |
| `.startsWith(prefix)` | `(string) -> bool` | Prefix check |
| `.endsWith(suffix)` | `(string) -> bool` | Suffix check |
| `.toUpperCase()` | `() -> string` | All uppercase |
| `.toLowerCase()` | `() -> string` | All lowercase |
| `.trim()` | `() -> string` | Strip leading and trailing whitespace |
| `.trimStart()` | `() -> string` | Strip leading whitespace |
| `.trimEnd()` | `() -> string` | Strip trailing whitespace |
| `.split(sep)` | `(string) -> string[]` | Split by separator |
| `.replace(old, new)` | `(string, string) -> string` | Replace first occurrence |
| `.replaceAll(old, new)` | `(string, string) -> string` | Replace all occurrences |
| `.repeat(count)` | `(number) -> string` | Repeat N times |
| `.padStart(len, pad)` | `(number, string) -> string` | Left-pad to length |
| `.padEnd(len, pad)` | `(number, string) -> string` | Right-pad to length |
| `.substring(start, end)` | `(number, number) -> string` | Extract slice |

```atlas
let s = "  Hello, Atlas!  ";
s.trim().toLowerCase().startsWith("hello"); // true
```

---

### `array`

All mutating array methods in Atlas are **non-mutating** — they return a new array via
CoW (copy-on-write). The original binding is updated automatically by the VM's CoW
write-back mechanism.

| Method | Signature | Description |
|--------|-----------|-------------|
| `.length()` | `() -> number` | Number of elements |
| `.push(value)` | `(T) -> T[]` | Append element |
| `.pop()` | `() -> T[]` | Remove last element |
| `.shift()` | `() -> T[]` | Remove first element |
| `.unshift(value)` | `(T) -> T[]` | Prepend element |
| `.slice(start, end)` | `(number, number) -> T[]` | Sub-array |
| `.concat(other)` | `(T[]) -> T[]` | Concatenate two arrays |
| `.indexOf(value)` | `(T) -> Option<number>` | Index of value, or None |
| `.lastIndexOf(value)` | `(T) -> Option<number>` | Last index of value, or None |
| `.includes(value)` | `(T) -> bool` | Whether value is present |
| `.reverse()` | `() -> T[]` | Reverse (new array) |
| `.sort()` | `() -> T[]` | Natural sort (new array) |
| `.join(sep)` | `(string) -> string` | Join elements with separator |
| `.map(fn)` | `((T) -> U) -> U[]` | Transform each element |
| `.filter(fn)` | `((T) -> bool) -> T[]` | Keep matching elements |
| `.reduce(fn, init)` | `((A, T) -> A, A) -> A` | Fold |
| `.forEach(fn)` | `((T) -> void) -> void` | Iterate with side effects |
| `.find(fn)` | `((T) -> bool) -> Option<T>` | First matching element |
| `.findIndex(fn)` | `((T) -> bool) -> Option<number>` | Index of first match |
| `.some(fn)` | `((T) -> bool) -> bool` | Any element matches |
| `.every(fn)` | `((T) -> bool) -> bool` | All elements match |
| `.flat()` | `() -> T[]` | Flatten one level |
| `.flatMap(fn)` | `((T) -> U[]) -> U[]` | Map then flatten |
| `.isEmpty()` | `() -> bool` | True if length is 0 |

```atlas
let nums = [3, 1, 4, 1, 5, 9];
let evens = nums.filter(fn(n): bool { return n % 2 == 0; });
let doubled = evens.map(fn(n): number { return n * 2; });
```

---

### `Map<K, V>`

| Method | Signature | Description |
|--------|-----------|-------------|
| `.get(key)` | `(K) -> Option<V>` | Get value by key (returns Option) |
| `.set(key, value)` | `(K, V) -> Map<K, V>` | Insert or update key-value pair |
| `.has(key)` | `(K) -> bool` | Key exists check |
| `.delete(key)` | `(K) -> Map<K, V>` | Remove key (unchanged if absent) |
| `.keys()` | `() -> K[]` | All keys (order not guaranteed) |
| `.values()` | `() -> V[]` | All values (order not guaranteed) |
| `.entries()` | `() -> [K, V][]` | Key-value pairs (order not guaranteed) |
| `.size()` | `() -> number` | Number of entries |
| `.isEmpty()` | `() -> bool` | True if empty |
| `.clear()` | `() -> Map<K, V>` | Empty the map |
| `.forEach(fn)` | `((K, V) -> void) -> void` | Iterate entries with side effects |
| `.map(fn)` | `((K, V) -> W) -> Map<K, W>` | Transform values |
| `.filter(fn)` | `((K, V) -> bool) -> Map<K, V>` | Keep matching entries |

```atlas
let mut m = new Map<string, number>();
m = m.set("x", 1);
m = m.set("y", 2);
let x = m.get("x");  // Some(1)
```

---

### `Set<T>`

| Method | Signature | Description |
|--------|-----------|-------------|
| `.add(value)` | `(T) -> Set<T>` | Add element (no-op if present) |
| `.has(value)` | `(T) -> bool` | Membership check |
| `.remove(value)` | `(T) -> Set<T>` | Remove element (unchanged if absent) |
| `.size()` | `() -> number` | Number of elements |
| `.isEmpty()` | `() -> bool` | True if empty |
| `.clear()` | `() -> Set<T>` | Empty the set |
| `.toArray()` | `() -> T[]` | All elements as array (order not guaranteed) |
| `.forEach(fn)` | `((T) -> void) -> void` | Iterate elements with side effects |

---

### `Queue<T>`

| Method | Signature | Description |
|--------|-----------|-------------|
| `.enqueue(value)` | `(T) -> Queue<T>` | Add to back |
| `.dequeue()` | `() -> [Option<T>, Queue<T>]` | Remove from front; returns `[value, updatedQueue]` |
| `.peek()` | `() -> Option<T>` | View front without removing |
| `.size()` | `() -> number` | Number of elements |
| `.isEmpty()` | `() -> bool` | True if empty |
| `.clear()` | `() -> Queue<T>` | Empty the queue |
| `.toArray()` | `() -> T[]` | All elements in FIFO order |

---

### `Stack<T>`

| Method | Signature | Description |
|--------|-----------|-------------|
| `.push(value)` | `(T) -> Stack<T>` | Push onto top |
| `.pop()` | `() -> [Option<T>, Stack<T>]` | Pop from top; returns `[value, updatedStack]` |
| `.peek()` | `() -> Option<T>` | View top without removing |
| `.size()` | `() -> number` | Number of elements |
| `.isEmpty()` | `() -> bool` | True if empty |
| `.clear()` | `() -> Stack<T>` | Empty the stack |
| `.toArray()` | `() -> T[]` | All elements in bottom-to-top order |

---

### `Option<T>`

| Method | Signature | Description |
|--------|-----------|-------------|
| `.isSome()` | `() -> bool` | True if has value |
| `.isNone()` | `() -> bool` | True if empty |
| `.unwrap()` | `() -> T` | Extract value (panics if None) |
| `.unwrapOr(default)` | `(T) -> T` | Extract value with fallback |
| `.map(fn)` | `((T) -> U) -> Option<U>` | Transform value if present |
| `.andThen(fn)` | `((T) -> Option<U>) -> Option<U>` | Chain optional operations |

```atlas
let opt: Option<number> = Some(42);
let val = opt.unwrapOr(0); // 42

let none: Option<number> = None;
let val2 = none.unwrapOr(0); // 0
```

---

### `Result<T, E>`

| Method | Signature | Description |
|--------|-----------|-------------|
| `.isOk()` | `() -> bool` | True if Ok |
| `.isErr()` | `() -> bool` | True if Err |
| `.unwrap()` | `() -> T` | Extract Ok value (panics if Err) |
| `.unwrapErr()` | `() -> E` | Extract Err value (panics if Ok) |
| `.unwrapOr(default)` | `(T) -> T` | Extract Ok value with fallback |
| `.map(fn)` | `((T) -> U) -> Result<U, E>` | Transform Ok value |
| `.mapErr(fn)` | `((E) -> F) -> Result<T, F>` | Transform Err value |
| `.andThen(fn)` | `((T) -> Result<U, E>) -> Result<U, E>` | Chain fallible operations |

```atlas
let result = Json.parse(raw);
let value = result.unwrapOr(None);
```

---

### `DateTime`

| Method | Signature | Description |
|--------|-----------|-------------|
| `.year()` | `() -> number` | Extract year |
| `.month()` | `() -> number` | Extract month (1–12) |
| `.day()` | `() -> number` | Extract day (1–31) |
| `.hour()` | `() -> number` | Extract hour (0–23) |
| `.minute()` | `() -> number` | Extract minute (0–59) |
| `.second()` | `() -> number` | Extract second (0–59) |
| `.timestamp()` | `() -> number` | Unix timestamp |
| `.format(fmt)` | `(string) -> string` | strftime formatting |
| `.addDays(n)` | `(number) -> DateTime` | Add/subtract days |
| `.addHours(n)` | `(number) -> DateTime` | Add/subtract hours |

```atlas
let now = DateTime.now();
let label = now.format("%Y-%m-%d");
let tomorrow = now.addDays(1);
```

---

### `Regex`

| Method | Signature | Description |
|--------|-----------|-------------|
| `.test(text)` | `(string) -> bool` | Match test |
| `.find(text)` | `(string) -> Option<map>` | First match |
| `.findAll(text)` | `(string) -> map[]` | All matches |
| `.replace(text, repl)` | `(string, string) -> string` | Replace first |
| `.split(text)` | `(string) -> string[]` | Split on matches |

```atlas
let re = Regex.new("\\d+").unwrap();
let found = re.find("abc 123 def");
```

---

### `Future<T>`

| Method | Signature | Description |
|--------|-----------|-------------|
| `.then(fn)` | `((T) -> U) -> Future<U>` | Chain on success |
| `.catch(fn)` | `((E) -> T) -> Future<T>` | Handle error |
| `.finally(fn)` | `(() -> void) -> Future<T>` | Run on settle |

---

### `ProcessOutput`

| Method | Signature | Description |
|--------|-----------|-------------|
| `.stdout()` | `() -> string` | Standard output text |
| `.stderr()` | `() -> string` | Standard error text |
| `.exitCode()` | `() -> number` | Exit code |
| `.success()` | `() -> bool` | True if exit code is 0 |

---

## Chaining Patterns

Atlas methods return new values (not void), enabling method chains:

### String chain

```atlas
let result = "  Hello World  ".trim().toLowerCase().replace("world", "atlas");
// "hello atlas"
```

### Array chain

```atlas
let result = [1, 2, 3, 4, 5, 6]
    .filter(fn(n): bool { return n % 2 == 0; })
    .map(fn(n): number { return n * n; })
    .reverse();
// [36, 16, 4]
```

### Option chain

```atlas
let result = Json.getString(data, "name")
    .map(fn(s): string { return s.trim().toUpperCase(); })
    .unwrapOr("UNKNOWN");
```

### Result chain

```atlas
let count = Json.parse(raw)
    .map(fn(data): number {
        return Json.getNumber(data, "count").unwrapOr(0);
    })
    .unwrapOr(0);
```

---

## CoW Mutation Pattern

Collection methods (`push`, `pop`, `set`, `delete`, etc.) return updated collections.
The VM automatically writes the result back to the binding. You do not need to reassign
explicitly in most cases, but the mutation only takes effect if the variable is `let mut`
or `var`.

```atlas
var arr = [1, 2, 3];
arr.push(4);   // arr is now [1, 2, 3, 4] — VM CoW write-back
arr.pop();     // arr is now [1, 2, 3]

var m = new Map<string, number>();
m.set("x", 1); // m now has key "x"
```

This is CoW — each operation creates a new internal value, but the variable is updated.
The original is only copied if there are multiple references.

---

## What NOT to Do

```atlas
// WRONG — method names must be camelCase:
let s = "hello";
s.ToUpperCase();    // does not exist
s.to_upper_case();  // does not exist

// WRONG — bare function names for namespace methods:
parseJSON(text);    // removed in B23 — use Json.parse(text)
base64Encode(s);    // removed in B27 — use Encoding.base64Encode(s)

// WRONG — chaining on void return:
console.log("a").toLowerCase(); // console.log returns void

// WRONG — mutating an immutable binding:
let arr = [1, 2];
arr.push(3); // TypeError — arr is not mutable; use 'var arr' or 'let mut arr'
```
