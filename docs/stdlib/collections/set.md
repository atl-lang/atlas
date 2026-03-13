# Set\<T\>

A collection of unique values with O(1) average-case membership testing, insertion, and removal. Backed by Rust's `HashSet`. Duplicate insertions are silently ignored — each value appears at most once.

Only **hashable types** may be stored: `number`, `string`, `bool`, `null`. Using an unhashable type produces a runtime `UnhashableType` error.

All mutation methods follow the **CoW (Copy-on-Write)** pattern — assign results back to apply changes.

## Type Annotation

```atlas
let set: Set<string> = new Set<string>();
```

---

## Construction

### `new Set<T>()`

Creates a new empty Set.

```atlas
let set = new Set<string>();
```

---

### `Set.fromArray(arr)`

```atlas
Set.fromArray(arr: T[]): Set<T>
```

Creates a Set from an array of hashable values. Duplicate array elements are deduplicated automatically.

```atlas
let set = Set.fromArray(["apple", "banana", "apple", "cherry"]);
// set contains: "apple", "banana", "cherry"
```

---

## Methods

### `.add(value)`

```atlas
.add(value: T): Set<T>
```

Adds an element to the set. If the element already exists, the set is unchanged. Returns the updated set (CoW — assign back).

```atlas
let mut set = new Set<string>();
set = set.add("hello");
set = set.add("world");
set = set.add("hello");  // no effect, already present
```

---

### `.has(value)`

```atlas
.has(value: T): bool
```

Returns `true` if the element is in the set.

```atlas
if set.has("hello") {
    console.log("found it");
}
```

---

### `.remove(value)`

```atlas
.remove(value: T): Set<T>
```

Removes an element from the set. If the element does not exist, the set is unchanged. Returns the updated set (CoW — assign back).

```atlas
set = set.remove("world");
```

---

### `.size()`

```atlas
.size(): number
```

Returns the number of elements.

```atlas
let n = set.size();
```

---

### `.isEmpty()`

```atlas
.isEmpty(): bool
```

Returns `true` if the set has no elements.

```atlas
if set.isEmpty() {
    console.log("empty");
}
```

---

### `.clear()`

```atlas
.clear(): Set<T>
```

Removes all elements. Returns an empty set (CoW — assign back).

```atlas
set = set.clear();
```

---

### `.toArray()`

```atlas
.toArray(): T[]
```

Returns all elements as an array. Order is not guaranteed.

```atlas
let elements = set.toArray();
for item in elements {
    console.log(item);
}
```

---

### `.forEach(fn)`

```atlas
.forEach(fn: (T) -> void): void
```

Calls the provided function once for each element. Order is not guaranteed.

```atlas
set.forEach(fn(item): void {
    console.log(item);
});
```

---

## Full Example

```atlas
fn uniqueTags(borrow posts: string[][]): Set<string> {
    let mut all = new Set<string>();
    for tags in posts {
        for tag in tags {
            all = all.add(tag);
        }
    }
    return all;
}

fn main(): void {
    let posts = [
        ["atlas", "lang", "systems"],
        ["atlas", "async", "runtime"],
        ["lang", "types"],
    ];

    let tags = uniqueTags(posts);
    console.log("Unique tag count: " + tags.size().toString());

    if tags.has("atlas") {
        console.log("Has atlas tag");
    }

    let arr = tags.toArray();
    for tag in arr {
        console.log(tag);
    }
}
```
