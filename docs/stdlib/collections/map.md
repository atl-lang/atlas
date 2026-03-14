# Map\<K, V\>

A key-value collection with O(1) average-case lookup, insertion, and deletion. Backed by Rust's `HashMap` with deterministic hashing.

Only **hashable types** may be used as keys: `number`, `string`, `bool`, `null`. Using an unhashable type as a key produces a runtime `UnhashableType` error.

All mutation methods follow the **CoW (Copy-on-Write)** pattern — they return an updated copy of the map. Assign the result back to apply the change.

## Type Annotation

```atlas
let map: Map<string, number> = new Map<string, number>();
```

---

## Construction

### `new Map<K, V>()`

Creates a new empty Map.

```atlas
let map = new Map<string, number>();
```

---

### `Map.fromEntries(entries)`

```atlas
Map.fromEntries(entries: [K, V][]): Map<K, V>
```

Creates a Map from an array of `[key, value]` pairs. Each entry must be a two-element array. Duplicate keys are overwritten by the last occurrence.

**Parameters:**
- `entries` — array of `[key, value]` arrays

```atlas
let map = Map.fromEntries([
    ["a", 1],
    ["b", 2],
    ["c", 3],
]);
```

---

## Methods

### `.set(key, value)`

```atlas
.set(key: K, value: V): Map<K, V>
```

Inserts or updates a key-value pair. Returns the updated map (CoW — assign back).

```atlas
let mut map = new Map<string, number>();
map = map.set("name", "Atlas");
map = map.set("version", 3);
```

---

### `.get(key)`

```atlas
.get(key: K): Option<V>
```

Returns `Some(value)` if the key exists, `None` otherwise.

```atlas
let result = map.get("name");
match result {
    Some(v) => console.log("Found: " + v),
    None    => console.log("Not found"),
}
```

---

### `.has(key)`

```atlas
.has(key: K): bool
```

Returns `true` if the key exists in the map.

```atlas
if map.has("name") {
    console.log("has name");
}
```

---

### `.delete(key)`

```atlas
.delete(key: K): Map<K, V>
```

Removes a key-value pair and returns the updated map (CoW — assign back). If the key does not exist, the map is unchanged.

```atlas
map = map.delete("name");
```

---

### `.keys()`

```atlas
.keys(): K[]
```

Returns all keys as an array. Order is not guaranteed.

```atlas
let keys = map.keys();
for key in keys {
    console.log(key);
}
```

---

### `.values()`

```atlas
.values(): V[]
```

Returns all values as an array. Order is not guaranteed.

```atlas
let vals = map.values();
for v in vals {
    console.log(v);
}
```

---

### `.entries()`

```atlas
.entries(): [K, V][]
```

Returns all key-value pairs as an array of `[key, value]` arrays. Order is not guaranteed.

```atlas
let pairs = map.entries();
for pair in pairs {
    console.log(pair[0].toString() + " => " + pair[1].toString());
}
```

---

### `.size()`

```atlas
.size(): number
```

Returns the number of entries.

```atlas
let n = map.size();
console.log("Entries: " + n.toString());
```

---

### `.isEmpty()`

```atlas
.isEmpty(): bool
```

Returns `true` if the map has no entries.

```atlas
if map.isEmpty() {
    console.log("map is empty");
}
```

---

### `.clear()`

```atlas
.clear(): Map<K, V>
```

Removes all entries. Returns an empty map (CoW — assign back).

```atlas
map = map.clear();
```

---

### `.forEach(fn)`

```atlas
.forEach(fn: (K, V): void): void
```

Calls the provided function once for each key-value pair. Order is not guaranteed.

```atlas
map.forEach(fn(k, v): void {
    console.log(k + " => " + v.toString());
});
```

---

### `.map(fn)`

```atlas
.map(fn: (K, V): W): Map<K, W>
```

Returns a new map with each value transformed by `fn`. Keys are preserved.

```atlas
let doubled = scores.map(fn(k, v): number { return v * 2; });
```

---

### `.filter(fn)`

```atlas
.filter(fn: (K, V): bool): Map<K, V>
```

Returns a new map containing only the entries for which `fn` returns `true`.

```atlas
let passing = scores.filter(fn(k, v): bool { return v >= 60; });
```

---

## Full Example

```atlas
fn wordCount(borrow words: string[]): Map<string, number> {
    let mut counts = new Map<string, number>();
    for word in words {
        let current = counts.get(word);
        let n = match current {
            Some(v) => v + 1,
            None    => 1,
        };
        counts = counts.set(word, n);
    }
    return counts;
}

fn main(): void {
    let words  = ["the", "cat", "sat", "on", "the", "mat", "the"];
    let counts = wordCount(words);

    let pairs = counts.entries();
    for pair in pairs {
        console.log(pair[0] + ": " + pair[1].toString());
    }
    // Output (order varies):
    // the: 3
    // cat: 1
    // sat: 1
    // on: 1
    // mat: 1
}
```
