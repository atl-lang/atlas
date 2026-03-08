# Array Functions

Pure array functions for manipulation and search. Returns new arrays (Copy-on-Write semantics).

## arrayPush

```atlas
fn arrayPush(arr: T[], item: T) -> T[]
```

Appends element to end of array. Returns new array.

**Parameters:**
- `arr` - Array to modify
- `item` - Element to add

**Returns:** `T[]` - New array with element appended

**Note:** Returns new array (CoW). Use: `arr = arrayPush(arr, item);`

## arrayPop

```atlas
fn arrayPop(arr: T[]) -> any
```

Removes and returns last element. Returns array: [removed_element, new_array].

**Parameters:**
- `arr` - Array to modify

**Returns:** `[T, T[]]` - Two-element array with removed element and new array

**Errors:**
- Cannot pop from empty array

**Alias:** `pop`

## arrayShift

```atlas
fn arrayShift(arr: T[]) -> any
```

Removes and returns first element. Returns array: [removed_element, new_array].

**Parameters:**
- `arr` - Array to modify

**Returns:** `[T, T[]]` - Two-element array with removed element and new array

**Errors:**
- Cannot shift from empty array

**Alias:** `shift`

## arrayUnshift

```atlas
fn arrayUnshift(arr: T[], item: T) -> T[]
```

Prepends element to beginning of array. Returns new array.

**Parameters:**
- `arr` - Array to modify
- `item` - Element to add

**Returns:** `T[]` - New array with element prepended

**Note:** Returns new array (CoW). Use: `arr = arrayUnshift(arr, item);`

**Alias:** `unshift`

## arrayReverse

```atlas
fn arrayReverse(arr: T[]) -> T[]
```

Reverses array elements. Returns new array.

**Parameters:**
- `arr` - Array to reverse

**Returns:** `T[]` - New reversed array

**Note:** Returns new array (CoW). Use: `arr = arrayReverse(arr);`

**Alias:** `reverse`

## arraySort

```atlas
fn arraySort(arr: T[]) -> T[]
```

Sorts array by natural order. Numbers ascending, strings lexicographic. Returns new array.

**Parameters:**
- `arr` - Array to sort

**Returns:** `T[]` - New sorted array

**Note:** Returns new array (CoW). Use: `arr = arraySort(arr);`

## concat

```atlas
fn concat(arr1: T[], arr2: T[]) -> T[]
```

Concatenates two arrays. Returns new array.

**Parameters:**
- `arr1` - First array
- `arr2` - Second array

**Returns:** `T[]` - New array with all elements

## flatten

```atlas
fn flatten(arr: T[][]) -> T[]
```

Flattens array by one level. Returns new array.

**Parameters:**
- `arr` - Array of arrays

**Returns:** `T[]` - Flattened array

## arrayIndexOf

```atlas
fn arrayIndexOf(arr: T[], search: T) -> Option<number>
```

Finds first index of element. Returns None if not found.

**Parameters:**
- `arr` - Array to search
- `search` - Element to find

**Returns:** `Option<number>` - Index of first match or None

## arrayLastIndexOf

```atlas
fn arrayLastIndexOf(arr: T[], search: T) -> Option<number>
```

Finds last index of element. Returns None if not found.

**Parameters:**
- `arr` - Array to search
- `search` - Element to find

**Returns:** `Option<number>` - Index of last match or None

## arrayIsEmpty

```atlas
fn arrayIsEmpty(arr: T[]) -> bool
```

Returns true if the array has no elements.

**Parameters:**
- `arr` - Array to check

**Returns:** `bool` - True if length is zero, false otherwise

**Method form:** `arr.isEmpty()`

## arrayIncludes

```atlas
fn arrayIncludes(arr: T[], search: T) -> bool
```

Checks if array contains element.

**Parameters:**
- `arr` - Array to search
- `search` - Element to find

**Returns:** `bool` - True if found, false otherwise

## slice

```atlas
fn slice(arr: T[], start: number, end: number) -> T[]
```

Extracts slice from start (inclusive) to end (exclusive).

**Parameters:**
- `arr` - Array to slice
- `start` - Start index (integer, clamped to [0, len])
- `end` - End index (integer, clamped to [0, len])

**Returns:** `T[]` - Sliced array

**Errors:**
- Start > end

**Note:** Negative indices not supported in v0.2, clamped to 0

## Callback Intrinsics

### map

```atlas
fn map(arr: T[], callback: fn(T) -> U) -> U[]
```

Transforms each element using `callback` and returns a new array.

**Parameters:**
- `arr` - Array to transform
- `callback` - Function invoked for each element

**Returns:** `U[]` - New array of mapped values

**Example:**
```atlas
let doubled = map([1, 2, 3], fn(x) { return x * 2; });
```

### filter

```atlas
fn filter(arr: T[], predicate: fn(T) -> bool) -> T[]
```

Filters elements using a predicate.

**Parameters:**
- `arr` - Array to filter
- `predicate` - Function returning true to keep the element

**Returns:** `T[]` - New array with matching elements

**Example:**
```atlas
let evens = filter([1, 2, 3, 4], fn(x) { return x % 2 == 0; });
```

### reduce

```atlas
fn reduce(arr: T[], reducer: fn(any, T) -> any, initial: any) -> any
```

Accumulates values into a single result.

**Parameters:**
- `arr` - Array to reduce
- `reducer` - Function `(acc, elem) -> new_acc`
- `initial` - Initial accumulator value

**Returns:** Accumulated result.

**Example:**
```atlas
let sum = reduce([1, 2, 3], fn(acc, x) { return acc + x; }, 0);
```

### forEach

```atlas
fn forEach(arr: T[], callback: fn(T) -> any) -> null
```

Invokes a callback for each element (side effects only).

**Parameters:**
- `arr` - Array to iterate
- `callback` - Function invoked for each element

**Returns:** `null`.

**Example:**
```atlas
forEach([1, 2], fn(x) { print(x); });
```

### find

```atlas
fn find(arr: T[], predicate: fn(T) -> bool) -> Option<T>
```

Returns the first element that satisfies the predicate.

**Parameters:**
- `arr` - Array to search
- `predicate` - Function returning true for a match

**Returns:** `Option<T>` - Some(value) if found, otherwise None.

**Example:**
```atlas
let first = find([1, 2, 3], fn(x) { return x > 1; });
```

### findIndex

```atlas
fn findIndex(arr: T[], predicate: fn(T) -> bool) -> Option<number>
```

Returns the index of the first element that satisfies the predicate.

**Parameters:**
- `arr` - Array to search
- `predicate` - Function returning true for a match

**Returns:** `Option<number>` - Some(index) if found, otherwise None.

**Example:**
```atlas
let idx = findIndex([5, 6, 7], fn(x) { return x == 6; });
```

### flatMap

```atlas
fn flatMap(arr: T[], callback: fn(T) -> any) -> any[]
```

Maps each element and flattens one level if the callback returns arrays.

**Parameters:**
- `arr` - Array to transform
- `callback` - Function invoked for each element

**Returns:** Flattened array.

**Example:**
```atlas
let out = flatMap([1, 2], fn(x) { return [x, x * 10]; });
```

### some

```atlas
fn some(arr: T[], predicate: fn(T) -> bool) -> bool
```

Returns true if any element satisfies the predicate.

**Parameters:**
- `arr` - Array to test
- `predicate` - Function returning true for a match

**Returns:** `bool`.

**Example:**
```atlas
let hasEven = some([1, 3, 4], fn(x) { return x % 2 == 0; });
```

### every

```atlas
fn every(arr: T[], predicate: fn(T) -> bool) -> bool
```

Returns true if all elements satisfy the predicate.

**Parameters:**
- `arr` - Array to test
- `predicate` - Function returning true for a match

**Returns:** `bool`.

**Example:**
```atlas
let allEven = every([2, 4], fn(x) { return x % 2 == 0; });
```

### sort

```atlas
fn sort(arr: T[], comparator: fn(T, T) -> number) -> T[]
```

Sorts the array using a comparator function.

**Parameters:**
- `arr` - Array to sort
- `comparator` - Function returning negative, zero, or positive

**Returns:** `T[]` - Sorted array.

**Example:**
```atlas
let sorted = sort([3, 1, 2], fn(a, b) { return a - b; });
```

### sortBy

```atlas
fn sortBy(arr: T[], keyExtractor: fn(T) -> any) -> T[]
```

Sorts the array by keys extracted from each element.

**Parameters:**
- `arr` - Array to sort
- `keyExtractor` - Function returning a number or string key

**Returns:** `T[]` - Sorted array.

**Example:**
```atlas
let sorted = sortBy([3, 1, 2], fn(x) { return x; });
```
