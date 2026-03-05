# Synchronization Functions

Synchronization primitives for shared state.

### rwLockNew

```atlas
fn rwLockNew(initial_value: any) -> RwLock
```

Creates a new read/write lock initialized with a value.

**Parameters:**
- `initial_value`: Value stored in the lock

**Returns:** RwLock handle.

**Example:**
```atlas
let lock = rwLockNew(0);
```

### rwLockRead

```atlas
fn rwLockRead(lock: RwLock) -> any
```

Acquires a read lock and returns a snapshot of the stored value.

**Parameters:**
- `lock`: RwLock handle

**Returns:** Stored value.

**Example:**
```atlas
let value = rwLockRead(lock);
```

### rwLockWrite

```atlas
fn rwLockWrite(lock: RwLock, new_value: any) -> null
```

Acquires a write lock and replaces the stored value.

**Parameters:**
- `lock`: RwLock handle
- `new_value`: Replacement value

**Returns:** `null`.

**Example:**
```atlas
rwLockWrite(lock, 42);
```

### rwLockTryRead

```atlas
fn rwLockTryRead(lock: RwLock) -> Option<any>
```

Attempts to acquire a read lock without blocking.

**Parameters:**
- `lock`: RwLock handle

**Returns:** `Some(value)` if acquired, otherwise `None`.

**Example:**
```atlas
let value = rwLockTryRead(lock);
```

### rwLockTryWrite

```atlas
fn rwLockTryWrite(lock: RwLock, new_value: any) -> bool
```

Attempts to acquire a write lock without blocking.

**Parameters:**
- `lock`: RwLock handle
- `new_value`: Replacement value

**Returns:** `true` if the write succeeded, otherwise `false`.

**Example:**
```atlas
let ok = rwLockTryWrite(lock, 7);
```

### semaphoreNew

```atlas
fn semaphoreNew(permits: number) -> Semaphore
```

Creates a semaphore with the given number of permits.

**Parameters:**
- `permits`: Number of permits (> 0)

**Returns:** Semaphore handle.

**Example:**
```atlas
let sem = semaphoreNew(4);
```

### semaphoreAcquire

```atlas
fn semaphoreAcquire(sem: Semaphore) -> null
```

Blocks until a permit is available and acquires it.

**Parameters:**
- `sem`: Semaphore handle

**Returns:** `null`.

**Example:**
```atlas
semaphoreAcquire(sem);
```

### semaphoreTryAcquire

```atlas
fn semaphoreTryAcquire(sem: Semaphore) -> bool
```

Attempts to acquire a permit without blocking.

**Parameters:**
- `sem`: Semaphore handle

**Returns:** `true` if a permit was acquired.

**Example:**
```atlas
let ok = semaphoreTryAcquire(sem);
```

### semaphoreRelease

```atlas
fn semaphoreRelease(sem: Semaphore) -> null
```

Releases a previously acquired permit.

**Parameters:**
- `sem`: Semaphore handle

**Returns:** `null`.

**Example:**
```atlas
semaphoreRelease(sem);
```

### semaphoreAvailable

```atlas
fn semaphoreAvailable(sem: Semaphore) -> number
```

Returns the number of currently available permits.

**Parameters:**
- `sem`: Semaphore handle

**Returns:** Available permit count.

**Example:**
```atlas
let remaining = semaphoreAvailable(sem);
```

### atomicNew

```atlas
fn atomicNew(initial: number) -> AtomicCounter
```

Creates a new atomic counter with an initial value.

**Parameters:**
- `initial`: Initial counter value

**Returns:** Atomic counter handle.

**Example:**
```atlas
let counter = atomicNew(0);
```

### atomicLoad

```atlas
fn atomicLoad(counter: AtomicCounter) -> number
```

Loads the current value of the atomic counter.

**Parameters:**
- `counter`: Atomic counter handle

**Returns:** Current counter value.

**Example:**
```atlas
let value = atomicLoad(counter);
```

### atomicStore

```atlas
fn atomicStore(counter: AtomicCounter, value: number) -> null
```

Stores a new value into the atomic counter.

**Parameters:**
- `counter`: Atomic counter handle
- `value`: Value to store

**Returns:** `null`.

**Example:**
```atlas
atomicStore(counter, 10);
```

### atomicAdd

```atlas
fn atomicAdd(counter: AtomicCounter, delta: number) -> number
```

Adds `delta` and returns the previous value.

**Parameters:**
- `counter`: Atomic counter handle
- `delta`: Amount to add

**Returns:** Previous counter value.

**Example:**
```atlas
let prev = atomicAdd(counter, 1);
```

### atomicSub

```atlas
fn atomicSub(counter: AtomicCounter, delta: number) -> number
```

Subtracts `delta` and returns the previous value.

**Parameters:**
- `counter`: Atomic counter handle
- `delta`: Amount to subtract

**Returns:** Previous counter value.

**Example:**
```atlas
let prev = atomicSub(counter, 1);
```

### atomicCompareExchange

```atlas
fn atomicCompareExchange(counter: AtomicCounter, expected: number, desired: number) -> bool
```

Compares the current value with `expected` and, if equal, sets it to `desired`.

**Parameters:**
- `counter`: Atomic counter handle
- `expected`: Expected current value
- `desired`: Desired new value

**Returns:** `true` if the swap succeeded.

**Example:**
```atlas
let swapped = atomicCompareExchange(counter, 5, 6);
```
