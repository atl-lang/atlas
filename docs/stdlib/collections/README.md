# collections

The `collections` module provides four general-purpose data structures built into the Atlas standard library. All collection types follow the **Copy-on-Write (CoW)** pattern: mutation methods return a new (or updated) collection value rather than mutating in place. Assign the return value back to apply the change.

All four types are first-class Atlas values. They are backed by Rust's standard library data structures for proven performance and correctness.

---

## Collection Types

| Type | Semantics | Backing | Key operations |
|------|-----------|---------|----------------|
| [Map](map.md) | Key-value pairs, O(1) avg | `HashMap<HashKey, Value>` | `.set()`, `.get()`, `.delete()`, `.has()` |
| [Set](set.md) | Unique values, O(1) avg | `HashSet<HashKey>` | `.add()`, `.remove()`, `.has()` |
| [Queue](queue.md) | FIFO ordered | `VecDeque<Value>` | `.enqueue()`, `.dequeue()`, `.peek()` |
| [Stack](stack.md) | LIFO ordered | `Vec<Value>` | `.push()`, `.pop()`, `.peek()` |

---

## Hashable Types

`Map` keys and `Set` elements must be **hashable**. Only the following Atlas types are hashable:

- `number` — IEEE 754; NaN values all hash identically
- `string`
- `bool`
- `null`

Arrays, functions, `Option`, `Result`, and other compound types cannot be used as keys. Attempting to do so produces a runtime `UnhashableType` error.

---

## Construction

| Collection | Constructor | From existing data |
|------------|-------------|-------------------|
| Map | `new Map<K, V>()` | `Map.fromEntries(entries)` |
| Set | `new Set<T>()` | `Set.fromArray(arr)` |
| Queue | `new Queue<T>()` | — |
| Stack | `new Stack<T>()` | — |

---

## Type Annotations

```atlas
let map:   Map<string, number> = new Map<string, number>();
let set:   Set<string>         = new Set<string>();
let queue: Queue<number>       = new Queue<number>();
let stack: Stack<number>       = new Stack<number>();
```

---

## CoW Write-Back Pattern

All mutation operations return the updated collection. You must assign the result back:

```atlas
let mut map = new Map<string, number>();
map = map.set("key", 42);        // write-back required

let mut stack = new Stack<string>();
stack = stack.push("item");      // write-back required
```

The VM's CoW write-back mechanism handles this automatically when calling methods on named variables that are declared `let mut` or `var`.

---

## Detailed Documentation

- [Map](map.md) — key-value collection
- [Set](set.md) — unique-value collection
- [Queue](queue.md) — FIFO queue
- [Stack](stack.md) — LIFO stack
