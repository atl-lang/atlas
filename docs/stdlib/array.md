# Array

Array methods are available as instance methods on any `array` value using dot syntax. All array operations are **copy-on-write (CoW)** — they return a new array and never mutate the original.

Callback-based operations (`map`, `filter`, `reduce`, `forEach`, `find`, `findIndex`, `some`, `every`, `flatMap`) are VM intrinsics and accept closures directly.

---

## Instance Methods

### `.push(element: T) -> T[]`

Returns a new array with `element` appended at the end.

```atlas
let nums = [1, 2, 3];
let nums2 = nums.push(4);
// nums2 == [1, 2, 3, 4]
```

---

### `.pop() -> [T, T[]]`

Removes and returns the last element. Returns a two-element array `[removedElement, newArray]`. Errors on empty array.

```atlas
let [last, rest] = [1, 2, 3].pop();
// last == 3, rest == [1, 2]
```

---

### `.shift() -> [T, T[]]`

Removes and returns the first element. Returns a two-element array `[removedElement, newArray]`. Errors on empty array.

```atlas
let [first, rest] = [1, 2, 3].shift();
// first == 1, rest == [2, 3]
```

---

### `.unshift(element: T) -> T[]`

Returns a new array with `element` prepended at the beginning.

```atlas
let nums = [2, 3, 4];
let nums2 = nums.unshift(1);
// nums2 == [1, 2, 3, 4]
```

---

### `.reverse() -> T[]`

Returns a new array with elements in reverse order.

```atlas
let reversed = [1, 2, 3].reverse();
// reversed == [3, 2, 1]
```

---

### `.concat(other: T[]) -> T[]`

Returns a new array containing all elements from both arrays.

```atlas
let a = [1, 2];
let b = [3, 4];
let c = a.concat(b);
// c == [1, 2, 3, 4]
```

---

### `.slice(start: number, end: number) -> T[]`

Returns a new array with elements from `start` (inclusive) to `end` (exclusive). Indices are clamped to array bounds. Errors if `start > end`.

```atlas
let sub = [10, 20, 30, 40, 50].slice(1, 4);
// sub == [20, 30, 40]
```

---

### `.indexOf(value: T) -> Option<number>`

Returns `Some(index)` of the first occurrence, or `None` if not found. Arrays and objects are compared by reference identity, not deep equality.

```atlas
let idx = ["a", "b", "c"].indexOf("b");
// idx == Some(1)

let missing = ["a", "b"].indexOf("z");
// missing == None
```

---

### `.lastIndexOf(value: T) -> Option<number>`

Returns `Some(index)` of the last occurrence, or `None` if not found.

```atlas
let idx = [1, 2, 1, 3].lastIndexOf(1);
// idx == Some(2)
```

---

### `.includes(value: T) -> bool`

Returns `true` if the array contains the given value.

```atlas
let found = ["x", "y", "z"].includes("y");
// found == true
```

---

### `.sort() -> T[]`

Returns a new sorted array. Numbers sort ascending by value; strings sort lexicographically. Original is not modified.

```atlas
let sorted = [3, 1, 4, 1, 5].sort();
// sorted == [1, 1, 3, 4, 5]

let words = ["banana", "apple", "cherry"].sort();
// words == ["apple", "banana", "cherry"]
```

---

### `.flat() -> T[]`

Flattens one level of nesting. Nested arrays are expanded in-place; non-array elements are kept as-is.

```atlas
let flat = [[1, 2], [3, 4], 5].flat();
// flat == [1, 2, 3, 4, 5]
```

---

### `.join(separator: string) -> string`

Joins all string elements into a single string separated by `separator`. All elements must be strings.

```atlas
let s = ["hello", "world"].join(", ");
// s == "hello, world"

let csv = ["a", "b", "c"].join(",");
// csv == "a,b,c"
```

---

### `.length -> number`

The number of elements in the array. Accessed as a property, not a method call.

```atlas
let n = [1, 2, 3].length;
// n == 3
```

---

## Callback-Based Methods (VM Intrinsics)

These methods accept a function or closure argument and are executed by the VM directly.

### `.map(fn: (T) -> U) -> U[]`

Transforms each element by applying `fn`.

```atlas
let doubled = [1, 2, 3].map(fn(borrow x: number): number { return x * 2; });
// doubled == [2, 4, 6]
```

### `.filter(fn: (T) -> bool) -> T[]`

Keeps only elements for which `fn` returns `true`.

```atlas
let evens = [1, 2, 3, 4].filter(fn(borrow x: number): bool { return x % 2 == 0; });
// evens == [2, 4]
```

### `.reduce(fn: (U, T) -> U, initial: U) -> U`

Accumulates elements into a single value starting from `initial`.

```atlas
let sum = [1, 2, 3, 4].reduce(
    fn(borrow acc: number, borrow x: number): number { return acc + x; },
    0
);
// sum == 10
```

### `.forEach(fn: (T) -> void) -> void`

Calls `fn` for each element. Returns `void`.

```atlas
[1, 2, 3].forEach(fn(borrow x: number): void {
    console.log(x.toString());
});
```

### `.find(fn: (T) -> bool) -> Option<T>`

Returns `Some(element)` for the first element where `fn` returns `true`, or `None`.

```atlas
let found = [1, 2, 3, 4].find(fn(borrow x: number): bool { return x > 2; });
// found == Some(3)
```

### `.findIndex(fn: (T) -> bool) -> Option<number>`

Returns `Some(index)` of the first element where `fn` returns `true`, or `None`.

```atlas
let idx = [10, 20, 30].findIndex(fn(borrow x: number): bool { return x == 20; });
// idx == Some(1)
```

### `.some(fn: (T) -> bool) -> bool`

Returns `true` if any element satisfies `fn`.

```atlas
let anyBig = [1, 2, 100].some(fn(borrow x: number): bool { return x > 50; });
// anyBig == true
```

### `.every(fn: (T) -> bool) -> bool`

Returns `true` if all elements satisfy `fn`.

```atlas
let allPos = [1, 2, 3].every(fn(borrow x: number): bool { return x > 0; });
// allPos == true
```

### `.flatMap(fn: (T) -> U[]) -> U[]`

Maps each element to an array and flattens one level.

```atlas
let result = [1, 2, 3].flatMap(fn(borrow x: number): number[] { return [x, x * 10]; });
// result == [1, 10, 2, 20, 3, 30]
```

---

## Global Array Utility Functions

### `arrayEnumerate(arr: T[]) -> (number, T)[]`

Returns an array of `(index, value)` tuples. Useful for indexed iteration without a counter variable.

```atlas
for pair in arrayEnumerate(["a", "b", "c"]) {
    let (i, v) = pair;
    console.log(i.toString() + ": " + v);
}
// 0: a
// 1: b
// 2: c
```

### `arrayFill(arr: T[], value: T, start: number, end: number) -> T[]`

Returns a new array with elements from `start` (inclusive) to `end` (exclusive) replaced with `value`.

```atlas
let filled = arrayFill([1, 2, 3, 4, 5], 0, 1, 3);
// filled == [1, 0, 0, 4, 5]
```
