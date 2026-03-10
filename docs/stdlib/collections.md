# Collections Functions

HashMap, HashSet, Queue, and Stack data structures.

## HashMap Functions

### hashMapNew

```atlas
fn hashMapNew() : HashMap<string, any>
```

Creates a new empty HashMap.

**Returns:** `HashMap<string, any>` - Empty map

### hashMapFromEntries

```atlas
fn hashMapFromEntries(entries: [string, any][]) : HashMap<string, any>
```

Creates HashMap from array of [key, value] pairs.

**Parameters:**
- `entries` - Array of [key, value] tuples

**Returns:** `HashMap<string, any>`

### hashMapPut

```atlas
fn hashMapPut(map: HashMap<K, V>, key: K, value: V) : HashMap<K, V>
```

Adds or updates key-value pair. Returns new map (CoW — always rebind the result).

**Parameters:**
- `map` - HashMap
- `key` - Key (usually string)
- `value` - Value

**Returns:** `HashMap<K, V>` - Modified map

**Note:** Uses Copy-on-Write semantics (Arc::make_mut internally). Always rebind: `map = hashMapPut(map, k, v)`

### hashMapCopy

```atlas
fn hashMapCopy(map: HashMap<K, V>) : HashMap<K, V>
```

Creates a shallow copy of HashMap.

**Parameters:**
- `map` - HashMap to copy

**Returns:** `HashMap<K, V>` - Copied map

### hashMapGet

```atlas
fn hashMapGet(map: HashMap<K, V>, key: K) : Option<V>
```

Gets value for key. Returns None if not found.

**Parameters:**
- `map` - HashMap
- `key` - Key to lookup

**Returns:** `Option<V>` - Value or None

### hashMapRemove

```atlas
fn hashMapRemove(map: HashMap<K, V>, key: K) : HashMap<K, V>
```

Removes key from map. Returns new map.

**Parameters:**
- `map` - HashMap
- `key` - Key to remove

**Returns:** `HashMap<K, V>` - Map without key

### hashMapHas

```atlas
fn hashMapHas(map: HashMap<K, V>, key: K) : bool
```

Checks if key exists in map.

**Parameters:**
- `map` - HashMap
- `key` - Key to check

**Returns:** `bool`

### hashMapSize

```atlas
fn hashMapSize(map: HashMap<K, V>) : number
```

Returns number of entries in map.

**Parameters:**
- `map` - HashMap

**Returns:** `number` - Entry count

### hashMapIsEmpty

```atlas
fn hashMapIsEmpty(map: HashMap<K, V>) : bool
```

Checks if map is empty.

**Parameters:**
- `map` - HashMap

**Returns:** `bool`

### hashMapClear

```atlas
fn hashMapClear(map: HashMap<K, V>) : HashMap<K, V>
```

Removes all entries. Returns empty map.

**Parameters:**
- `map` - HashMap

**Returns:** `HashMap<K, V>` - Empty map

### hashMapKeys

```atlas
fn hashMapKeys(map: HashMap<K, V>) : K[]
```

Gets all keys as array.

**Parameters:**
- `map` - HashMap

**Returns:** `K[]` - Array of keys

### hashMapValues

```atlas
fn hashMapValues(map: HashMap<K, V>) : V[]
```

Gets all values as array.

**Parameters:**
- `map` - HashMap

**Returns:** `V[]` - Array of values

### hashMapEntries

```atlas
fn hashMapEntries(map: HashMap<K, V>) : [K, V][]
```

Gets all entries as array of [key, value] tuples.

**Parameters:**
- `map` - HashMap

**Returns:** `[K, V][]` - Array of entries

### hashMapForEach

```atlas
fn hashMapForEach(map: HashMap<K, V>, callback: fn(V, K) : any) : null
```

Invokes a callback for each key/value pair.

**Parameters:**
- `map` - HashMap
- `callback` - Function invoked as `(value, key)`

**Returns:** `null`

**Example:**
```atlas
hashMapForEach(map, fn(value, key) { print(key); });
```

### hashMapMap

```atlas
fn hashMapMap(map: HashMap<K, V>, callback: fn(V, K) : U) : HashMap<K, U>
```

Transforms values using a callback and returns a new map.

**Parameters:**
- `map` - HashMap
- `callback` - Function invoked as `(value, key)`

**Returns:** `HashMap<K, U>` - New map with transformed values

**Example:**
```atlas
let doubled = hashMapMap(map, fn(value, key) { return value * 2; });
```

### hashMapFilter

```atlas
fn hashMapFilter(map: HashMap<K, V>, predicate: fn(V, K) : bool) : HashMap<K, V>
```

Filters entries using a predicate and returns a new map.

**Parameters:**
- `map` - HashMap
- `predicate` - Function invoked as `(value, key)`

**Returns:** `HashMap<K, V>` - Filtered map

**Example:**
```atlas
let filtered = hashMapFilter(map, fn(value, key) { return value > 10; });
```

## HashSet Functions

### hashSetNew

```atlas
fn hashSetNew() : HashSet<T>
```

Creates a new empty HashSet.

**Returns:** `HashSet<T>` - Empty set

### hashSetFromArray

```atlas
fn hashSetFromArray(arr: T[]) : HashSet<T>
```

Creates HashSet from array.

**Parameters:**
- `arr` - Array of values

**Returns:** `HashSet<T>` - Set with array elements

### hashSetAdd

```atlas
fn hashSetAdd(set: HashSet<T>, value: T) : HashSet<T>
```

Adds element to set. Returns new set (CoW — always rebind the result).

**Parameters:**
- `set` - HashSet
- `value` - Value to add

**Returns:** `HashSet<T>`

### hashSetRemove

```atlas
fn hashSetRemove(set: HashSet<T>, value: T) : HashSet<T>
```

Removes element from set. Returns new set.

**Parameters:**
- `set` - HashSet
- `value` - Value to remove

**Returns:** `HashSet<T>`

### hashSetHas

```atlas
fn hashSetHas(set: HashSet<T>, value: T) : bool
```

Checks if element is in set.

**Parameters:**
- `set` - HashSet
- `value` - Value to check

**Returns:** `bool`

### hashSetSize

```atlas
fn hashSetSize(set: HashSet<T>) : number
```

Returns number of elements in set.

**Parameters:**
- `set` - HashSet

**Returns:** `number`

### hashSetIsEmpty

```atlas
fn hashSetIsEmpty(set: HashSet<T>) : bool
```

Checks if set is empty.

**Parameters:**
- `set` - HashSet

**Returns:** `bool`

### hashSetClear

```atlas
fn hashSetClear(set: HashSet<T>) : HashSet<T>
```

Removes all elements. Returns empty set.

**Parameters:**
- `set` - HashSet

**Returns:** `HashSet<T>` - Empty set

### hashSetUnion

```atlas
fn hashSetUnion(set1: HashSet<T>, set2: HashSet<T>) : HashSet<T>
```

Returns union of two sets (all elements from both).

**Parameters:**
- `set1` - First set
- `set2` - Second set

**Returns:** `HashSet<T>` - Union

### hashSetIntersection

```atlas
fn hashSetIntersection(set1: HashSet<T>, set2: HashSet<T>) : HashSet<T>
```

Returns intersection of two sets (elements in both).

**Parameters:**
- `set1` - First set
- `set2` - Second set

**Returns:** `HashSet<T>` - Intersection

### hashSetDifference

```atlas
fn hashSetDifference(set1: HashSet<T>, set2: HashSet<T>) : HashSet<T>
```

Returns difference: elements in set1 but not in set2.

**Parameters:**
- `set1` - First set
- `set2` - Second set

**Returns:** `HashSet<T>` - Difference

### hashSetSymmetricDifference

```atlas
fn hashSetSymmetricDifference(set1: HashSet<T>, set2: HashSet<T>) : HashSet<T>
```

Returns symmetric difference: elements in either but not both.

**Parameters:**
- `set1` - First set
- `set2` - Second set

**Returns:** `HashSet<T>` - Symmetric difference

### hashSetIsSubset

```atlas
fn hashSetIsSubset(set1: HashSet<T>, set2: HashSet<T>) : bool
```

Checks if set1 is subset of set2 (all elements of set1 in set2).

**Parameters:**
- `set1` - Potential subset
- `set2` - Potential superset

**Returns:** `bool`

### hashSetIsSuperset

```atlas
fn hashSetIsSuperset(set1: HashSet<T>, set2: HashSet<T>) : bool
```

Checks if set1 is superset of set2 (all elements of set2 in set1).

**Parameters:**
- `set1` - Potential superset
- `set2` - Potential subset

**Returns:** `bool`

### hashSetToArray

```atlas
fn hashSetToArray(set: HashSet<T>) : T[]
```

Converts set to array.

**Parameters:**
- `set` - HashSet

**Returns:** `T[]` - Array of elements

### hashSetForEach

```atlas
fn hashSetForEach(set: HashSet<T>, callback: fn(T) : any) : null
```

Invokes a callback for each element in the set.

**Parameters:**
- `set` - HashSet
- `callback` - Function invoked for each element

**Returns:** `null`

**Example:**
```atlas
hashSetForEach(set, fn(value) { print(value); });
```

### hashSetMap

```atlas
fn hashSetMap(set: HashSet<T>, callback: fn(T) : U) : U[]
```

Maps set elements to an array of results.

**Parameters:**
- `set` - HashSet
- `callback` - Function invoked for each element

**Returns:** `U[]` - Array of mapped values

**Example:**
```atlas
let values = hashSetMap(set, fn(value) { return value * 2; });
```

### hashSetFilter

```atlas
fn hashSetFilter(set: HashSet<T>, predicate: fn(T) : bool) : HashSet<T>
```

Filters set elements using a predicate.

**Parameters:**
- `set` - HashSet
- `predicate` - Function invoked for each element

**Returns:** `HashSet<T>` - Filtered set

**Example:**
```atlas
let filtered = hashSetFilter(set, fn(value) { return value > 0; });
```

## Queue Functions

### queueNew

```atlas
fn queueNew() : Queue<T>
```

Creates new empty queue.

**Returns:** `Queue<T>` - FIFO queue

### queueEnqueue

```atlas
fn queueEnqueue(queue: Queue<T>, value: T) : Queue<T>
```

Adds element to back of queue.

**Parameters:**
- `queue` - Queue
- `value` - Value to add

**Returns:** `Queue<T>`

### queueDequeue

```atlas
fn queueDequeue(queue: Queue<T>) : Option<T>
```

Removes and returns element from front of queue.

**Parameters:**
- `queue` - Queue

**Returns:** `Option<T>` - Front element or None if empty

### queuePeek

```atlas
fn queuePeek(queue: Queue<T>) : Option<T>
```

Returns front element without removing.

**Parameters:**
- `queue` - Queue

**Returns:** `Option<T>` - Front element or None if empty

### queueSize

```atlas
fn queueSize(queue: Queue<T>) : number
```

Returns number of elements in queue.

**Parameters:**
- `queue` - Queue

**Returns:** `number`

### queueIsEmpty

```atlas
fn queueIsEmpty(queue: Queue<T>) : bool
```

Checks if queue is empty.

**Parameters:**
- `queue` - Queue

**Returns:** `bool`

### queueClear

```atlas
fn queueClear(queue: Queue<T>) : Queue<T>
```

Removes all elements.

**Parameters:**
- `queue` - Queue

**Returns:** `Queue<T>` - Empty queue

### queueToArray

```atlas
fn queueToArray(queue: Queue<T>) : T[]
```

Converts queue to array.

**Parameters:**
- `queue` - Queue

**Returns:** `T[]` - Array in FIFO order

## Stack Functions

### stackNew

```atlas
fn stackNew() : Stack<T>
```

Creates new empty stack.

**Returns:** `Stack<T>` - LIFO stack

### stackPush

```atlas
fn stackPush(stack: Stack<T>, value: T) : Stack<T>
```

Adds element to top of stack.

**Parameters:**
- `stack` - Stack
- `value` - Value to push

**Returns:** `Stack<T>`

### stackPop

```atlas
fn stackPop(stack: Stack<T>) : Option<T>
```

Removes and returns element from top of stack.

**Parameters:**
- `stack` - Stack

**Returns:** `Option<T>` - Top element or None if empty

### stackPeek

```atlas
fn stackPeek(stack: Stack<T>) : Option<T>
```

Returns top element without removing.

**Parameters:**
- `stack` - Stack

**Returns:** `Option<T>` - Top element or None if empty

### stackSize

```atlas
fn stackSize(stack: Stack<T>) : number
```

Returns number of elements in stack.

**Parameters:**
- `stack` - Stack

**Returns:** `number`

### stackIsEmpty

```atlas
fn stackIsEmpty(stack: Stack<T>) : bool
```

Checks if stack is empty.

**Parameters:**
- `stack` - Stack

**Returns:** `bool`

### stackClear

```atlas
fn stackClear(stack: Stack<T>) : Stack<T>
```

Removes all elements.

**Parameters:**
- `stack` - Stack

**Returns:** `Stack<T>` - Empty stack

### stackToArray

```atlas
fn stackToArray(stack: Stack<T>) : T[]
```

Converts stack to array.

**Parameters:**
- `stack` - Stack

**Returns:** `T[]` - Array in LIFO order (top to bottom)
