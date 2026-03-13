# Queue\<T\>

A FIFO (First-In, First-Out) queue backed by a circular buffer (`VecDeque`). Elements enqueued first are dequeued first. Both `enqueue` and `dequeue` operate in O(1) amortized time.

Accepts values of any type — no hashability requirement.

All mutation methods follow the **CoW (Copy-on-Write)** pattern. Assign results back to apply changes.

## Type Annotation

```atlas
let q: Queue<string> = new Queue<string>();
```

---

## Construction

### `new Queue<T>()`

Creates a new empty queue.

```atlas
let q = new Queue<string>();
```

---

## Methods

### `.enqueue(value)`

```atlas
.enqueue(value: T): Queue<T>
```

Adds an element to the back of the queue. Returns the updated queue (CoW — assign back).

```atlas
let mut q = new Queue<string>();
q = q.enqueue("first");
q = q.enqueue("second");
q = q.enqueue("third");
```

---

### `.dequeue()`

```atlas
.dequeue(): [Option<T>, Queue<T>]
```

Removes and returns the element from the front of the queue. Returns a two-element array:
- `[0]` — `Some(element)` if the queue was non-empty, `None` if empty
- `[1]` — the updated queue after removal (CoW — assign back)

```atlas
let result  = q.dequeue();
let element = result[0];   // Some("first") or None
q           = result[1];   // updated queue

match element {
    Some(v) => console.log("Dequeued: " + v),
    None    => console.log("Queue was empty"),
}
```

Destructuring syntax:

```atlas
let [item, remaining] = q.dequeue();
q = remaining;
```

---

### `.peek()`

```atlas
.peek(): Option<T>
```

Returns the front element without removing it. Returns `Some(element)` if non-empty, `None` if empty.

```atlas
let front = q.peek();
match front {
    Some(v) => console.log("Front: " + v),
    None    => console.log("Empty"),
}
```

---

### `.size()`

```atlas
.size(): number
```

Returns the number of elements in the queue.

```atlas
let n = q.size();
console.log("Queue has " + n.toString() + " items");
```

---

### `.isEmpty()`

```atlas
.isEmpty(): bool
```

Returns `true` if the queue has no elements.

```atlas
while !q.isEmpty() {
    let [item, remaining] = q.dequeue();
    q = remaining;
    match item {
        Some(v) => console.log(v),
        None    => {},
    }
}
```

---

### `.clear()`

```atlas
.clear(): Queue<T>
```

Removes all elements. Returns an empty queue (CoW — assign back).

```atlas
q = q.clear();
```

---

### `.toArray()`

```atlas
.toArray(): T[]
```

Returns all elements as an array in FIFO order (front of queue at index 0).

```atlas
let arr = q.toArray();
for item in arr {
    console.log(item);
}
```

---

## Full Example

```atlas
fn processQueue(): void {
    let mut tasks = new Queue<string>();

    // Enqueue tasks
    tasks = tasks.enqueue("parse");
    tasks = tasks.enqueue("typecheck");
    tasks = tasks.enqueue("compile");
    tasks = tasks.enqueue("link");

    console.log("Tasks queued: " + tasks.size().toString());

    // Process in FIFO order
    while !tasks.isEmpty() {
        let [task, remaining] = tasks.dequeue();
        tasks = remaining;
        match task {
            Some(t) => console.log("Running: " + t),
            None    => {},
        }
    }
    // Output:
    // Running: parse
    // Running: typecheck
    // Running: compile
    // Running: link
}
```
