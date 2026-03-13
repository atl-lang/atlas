# Stack\<T\>

A LIFO (Last-In, First-Out) stack backed by a `Vec`. Elements pushed last are popped first. Both `push` and `pop` operate in O(1) amortized time.

Accepts values of any type — no hashability requirement.

All mutation methods follow the **CoW (Copy-on-Write)** pattern. Assign results back to apply changes.

## Type Annotation

```atlas
let s: Stack<string> = new Stack<string>();
```

---

## Construction

### `new Stack<T>()`

Creates a new empty stack.

```atlas
let s = new Stack<string>();
```

---

## Methods

### `.push(value)`

```atlas
.push(value: T): Stack<T>
```

Pushes an element onto the top of the stack. Returns the updated stack (CoW — assign back).

```atlas
let mut s = new Stack<string>();
s = s.push("a");
s = s.push("b");
s = s.push("c");
// top: "c"
```

---

### `.pop()`

```atlas
.pop(): [Option<T>, Stack<T>]
```

Removes and returns the element from the top of the stack. Returns a two-element array:
- `[0]` — `Some(element)` if the stack was non-empty, `None` if empty
- `[1]` — the updated stack after removal (CoW — assign back)

```atlas
let result  = s.pop();
let element = result[0];  // Some("c") or None
s           = result[1];  // updated stack

match element {
    Some(v) => console.log("Popped: " + v),
    None    => console.log("Stack was empty"),
}
```

Destructuring syntax:

```atlas
let [top, remaining] = s.pop();
s = remaining;
```

---

### `.peek()`

```atlas
.peek(): Option<T>
```

Returns the top element without removing it. Returns `Some(element)` if non-empty, `None` if empty.

```atlas
let top = s.peek();
match top {
    Some(v) => console.log("Top: " + v),
    None    => console.log("Empty"),
}
```

---

### `.size()`

```atlas
.size(): number
```

Returns the number of elements in the stack.

```atlas
let n = s.size();
console.log("Stack depth: " + n.toString());
```

---

### `.isEmpty()`

```atlas
.isEmpty(): bool
```

Returns `true` if the stack has no elements.

```atlas
if s.isEmpty() {
    console.log("nothing on the stack");
}
```

---

### `.clear()`

```atlas
.clear(): Stack<T>
```

Removes all elements. Returns an empty stack (CoW — assign back).

```atlas
s = s.clear();
```

---

### `.toArray()`

```atlas
.toArray(): T[]
```

Returns all elements as an array in bottom-to-top order (bottom of stack at index 0, top at last index).

```atlas
let arr = s.toArray();
// if stack is [bottom -> "a", "b", "c" <- top]
// arr is ["a", "b", "c"]
```

---

## Full Example

```atlas
fn reverseArray(borrow items: string[]): string[] {
    let mut s = new Stack<string>();

    // Push all items
    for item in items {
        s = s.push(item);
    }

    // Pop in reverse order
    let mut result: string[] = [];
    while !s.isEmpty() {
        let [item, remaining] = s.pop();
        s = remaining;
        match item {
            Some(v) => result = result + [v],
            None    => {},
        }
    }
    return result;
}

fn main(): void {
    let words    = ["one", "two", "three", "four"];
    let reversed = reverseArray(words);
    for word in reversed {
        console.log(word);
    }
    // Output:
    // four
    // three
    // two
    // one
}
```

---

## Typical Use Cases

**Expression evaluation:** Push operands onto the stack; operators pop and push results.

**Undo history:** Push state snapshots; pop to undo.

**Depth-first traversal:** Push children in reverse order; pop next node to visit.

```atlas
fn depthFirst(borrow root: string): void {
    let mut s = new Stack<string>();
    s = s.push(root);

    while !s.isEmpty() {
        let [node, remaining] = s.pop();
        s = remaining;
        match node {
            Some(v) => console.log("Visit: " + v),
            None    => {},
        }
    }
}
```
