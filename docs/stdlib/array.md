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
fn arrayIndexOf(arr: T[], search: T) -> number?
```

Finds first index of element. Returns None if not found.

**Parameters:**
- `arr` - Array to search
- `search` - Element to find

**Returns:** `number?` - Index of first match or None

## arrayLastIndexOf

```atlas
fn arrayLastIndexOf(arr: T[], search: T) -> number?
```

Finds last index of element. Returns None if not found.

**Parameters:**
- `arr` - Array to search
- `search` - Element to find

**Returns:** `number?` - Index of last match or None

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
