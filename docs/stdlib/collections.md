# Collections

Atlas provides four typed collection types: `Map<K, V>`, `Set<T>`, `Queue<T>`, and `Stack<T>`. All collections are **copy-on-write (CoW)** — mutating operations return the updated collection and the VM writes it back to the variable automatically.

Only hashable types can be used as Map keys or Set elements: `number`, `string`, `bool`, and `null`.

---

## Map\<K, V\>

A key-value map with O(1) average-case get, set, and delete. Keys must be hashable (`number`, `string`, `bool`, `null`).

### `new Map<K, V>() -> Map<K, V>`

Creates a new empty Map.

```atlas
let map: Map<string, number> = new Map<string, number>();
```

### `Map.fromEntries(entries: [K, V][]) -> Map<K, V>`

Creates a Map from an array of `[key, value]` pairs.

```atlas
let map = Map.fromEntries([
    ["a", 1],
    ["b", 2],
    ["c", 3],
]);
```

---

### `.set(key: K, value: V) -> Map<K, V>`

Inserts or updates a key-value pair. Returns the updated map.

```atlas
let mut map = new Map<string, number>();
map = map.set("name", "Atlas");
map = map.set("version", 3);
```

### `.get(key: K) -> Option<V>`

Returns `Some(value)` if the key exists, or `None` if not.

```atlas
let val = map.get("name");
match val {
    Some(v) => console.log(v),
    None    => console.log("not found"),
}
```

### `.has(key: K) -> bool`

Returns `true` if the key exists in the map.

```atlas
if map.has("name") {
    console.log("has name");
}
```

### `.delete(key: K) -> Map<K, V>`

Removes a key. Returns the updated map (unchanged if key was absent).

```atlas
map = map.delete("name");
```

### `.keys() -> K[]`

Returns all keys as an array. Order is not guaranteed.

```atlas
let k = map.keys();
```

### `.values() -> V[]`

Returns all values as an array. Order is not guaranteed.

```atlas
let v = map.values();
```

### `.entries() -> [K, V][]`

Returns all key-value pairs as an array of `[key, value]` arrays. Order is not guaranteed.

```atlas
for entry in map.entries() {
    console.log(entry[0] + " => " + entry[1].toString());
}
```

### `.size() -> number`

Returns the number of entries in the map.

```atlas
let n = map.size();
```

### `.isEmpty() -> bool`

Returns `true` if the map contains no entries.

```atlas
if map.isEmpty() {
    console.log("empty");
}
```

### `.clear() -> Map<K, V>`

Returns an empty map (all entries removed).

```atlas
map = map.clear();
```

### `.forEach(fn: (K, V) -> void) -> void`

Calls the provided function once for each key-value pair.

```atlas
map.forEach(fn(k, v): void {
    console.log(k + ": " + v.toString());
});
```

### `.map(fn: (K, V) -> W) -> Map<K, W>`

Returns a new map with each value transformed by `fn`. Keys are preserved.

```atlas
let doubled = scores.map(fn(k, v): number { return v * 2; });
```

### `.filter(fn: (K, V) -> bool) -> Map<K, V>`

Returns a new map containing only the entries for which `fn` returns `true`.

```atlas
let passing = scores.filter(fn(k, v): bool { return v >= 60; });
```

---

### Full Example

```atlas
let mut scores: Map<string, number> = new Map<string, number>();
scores = scores.set("Alice", 95);
scores = scores.set("Bob", 87);
scores = scores.set("Charlie", 92);

console.log(scores.size().toString());  // 3

let alice = scores.get("Alice");
match alice {
    Some(n) => console.log("Alice: " + n.toString()),
    None    => console.log("not found"),
}

for entry in scores.entries() {
    console.log(entry[0] + ": " + entry[1].toString());
}
```

---

## Set\<T\>

A set of unique hashable values with O(1) average-case membership testing. Automatically deduplicates on insert.

### `new Set<T>() -> Set<T>`

Creates a new empty Set.

```atlas
let set: Set<string> = new Set<string>();
```

### `Set.fromArray(arr: T[]) -> Set<T>`

Creates a Set from an array, deduplicating automatically.

```atlas
let set = Set.fromArray([1, 2, 2, 3, 3]);
// set contains {1, 2, 3}
```

---

### `.add(value: T) -> Set<T>`

Adds an element. If already present, the set is unchanged. Returns the updated set.

```atlas
let mut set = new Set<string>();
set = set.add("apple");
set = set.add("banana");
set = set.add("apple");  // duplicate, no effect
// set contains {"apple", "banana"}
```

### `.has(value: T) -> bool`

Returns `true` if the value is in the set.

```atlas
set.has("apple")   // true
set.has("cherry")  // false
```

### `.remove(value: T) -> Set<T>`

Removes an element. If not present, the set is unchanged. Returns the updated set.

```atlas
set = set.remove("apple");
```

### `.size() -> number`

Returns the number of elements.

```atlas
let n = set.size();
```

### `.isEmpty() -> bool`

Returns `true` if the set is empty.

```atlas
set.isEmpty()  // false
```

### `.clear() -> Set<T>`

Returns an empty set.

```atlas
set = set.clear();
```

### `.toArray() -> T[]`

Returns all elements as an array. Order is not guaranteed.

```atlas
let arr = set.toArray();
```

### `.forEach(fn: (T) -> void) -> void`

Calls the provided function once for each element.

```atlas
set.forEach(fn(item): void {
    console.log(item);
});
```

---

### Full Example

```atlas
let mut visited: Set<string> = new Set<string>();

fn visit(borrow url: string): void {
    if visited.has(url) {
        console.log("Already visited: " + url);
        return;
    }
    visited = visited.add(url);
    console.log("Visiting: " + url);
}

visit("https://example.com");
visit("https://other.com");
visit("https://example.com");  // already visited
```

---

## Queue\<T\>

A FIFO (First-In-First-Out) queue backed by a circular buffer (`VecDeque`) for O(1) enqueue and dequeue.

### `new Queue<T>() -> Queue<T>`

Creates a new empty queue.

```atlas
let q: Queue<number> = new Queue<number>();
```

---

### `.enqueue(value: T) -> Queue<T>`

Adds an element to the back of the queue. Returns the updated queue.

```atlas
let mut q = new Queue<string>();
q = q.enqueue("task1");
q = q.enqueue("task2");
q = q.enqueue("task3");
```

### `.dequeue() -> [Option<T>, Queue<T>]`

Removes and returns the front element. Returns `[Option<element>, updatedQueue]`. Returns `None` if the queue is empty.

```atlas
let [item, remaining] = q.dequeue();
q = remaining;
match item {
    Some(v) => console.log("got: " + v),
    None    => console.log("queue was empty"),
}
```

### `.peek() -> Option<T>`

Views the front element without removing it. Returns `None` if empty.

```atlas
let front = q.peek();
```

### `.size() -> number`

Returns the number of elements in the queue.

```atlas
let n = q.size();
```

### `.isEmpty() -> bool`

Returns `true` if the queue has no elements.

```atlas
while !q.isEmpty() {
    let [item, remaining] = q.dequeue();
    q = remaining;
    console.log(item.unwrap());
}
```

### `.clear() -> Queue<T>`

Returns an empty queue.

```atlas
q = q.clear();
```

### `.toArray() -> T[]`

Returns elements as an array in FIFO order (front first).

```atlas
let arr = q.toArray();
```

---

### Full Example

```atlas
let mut tasks: Queue<string> = new Queue<string>();
tasks = tasks.enqueue("task1");
tasks = tasks.enqueue("task2");
tasks = tasks.enqueue("task3");

while !tasks.isEmpty() {
    let [task, remaining] = tasks.dequeue();
    tasks = remaining;
    console.log("Processing: " + task.unwrap());
}
// Processing: task1
// Processing: task2
// Processing: task3
```

---

## Stack\<T\>

A LIFO (Last-In-First-Out) stack backed by a `Vec` for O(1) push and pop.

### `new Stack<T>() -> Stack<T>`

Creates a new empty stack.

```atlas
let s: Stack<number> = new Stack<number>();
```

---

### `.push(value: T) -> Stack<T>`

Pushes an element onto the top of the stack. Returns the updated stack.

```atlas
let mut s = new Stack<number>();
s = s.push(1);
s = s.push(2);
s = s.push(3);
```

### `.pop() -> [Option<T>, Stack<T>]`

Removes and returns the top element. Returns `[Option<element>, updatedStack]`. Returns `None` if the stack is empty.

```atlas
let [top, remaining] = s.pop();
s = remaining;
match top {
    Some(v) => console.log("popped: " + v.toString()),
    None    => console.log("stack was empty"),
}
```

### `.peek() -> Option<T>`

Views the top element without removing it. Returns `None` if empty.

```atlas
let top = s.peek();
```

### `.size() -> number`

Returns the number of elements in the stack.

```atlas
let n = s.size();
```

### `.isEmpty() -> bool`

Returns `true` if the stack has no elements.

```atlas
s.isEmpty()  // false after pushes
```

### `.clear() -> Stack<T>`

Returns an empty stack.

```atlas
s = s.clear();
```

### `.toArray() -> T[]`

Returns elements as an array in bottom-to-top order (bottom of stack is first element).

```atlas
let arr = s.toArray();
// arr == [1, 2, 3] if 1 was pushed first
```

---

### Full Example

```atlas
// Expression parser using a stack for operator precedence
let mut operands: Stack<number> = new Stack<number>();
operands = operands.push(10);
operands = operands.push(5);
operands = operands.push(3);

let [top, s2] = operands.pop();
operands = s2;
let [next, s3] = operands.pop();
operands = s3;

let result = top.unwrap() + next.unwrap();
console.log(result.toString());  // 8
```

---

## CoW Mutation Pattern

All four collection types use copy-on-write semantics. The VM handles the write-back automatically when you reassign to a `let mut` or `var` binding:

```atlas
let mut map = new Map<string, number>();
map = map.set("x", 1);   // CoW: returns new map, VM writes back to 'map'

let mut set = new Set<string>();
set = set.add("hello");  // same pattern

let mut q = new Queue<number>();
q = q.enqueue(42);

let mut s = new Stack<number>();
s = s.push(99);
```

This means all bindings (both `let mut` and `var`) can hold mutable collections — mutation is content-level, not binding-level.
