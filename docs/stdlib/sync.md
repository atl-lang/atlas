# sync namespace

Synchronization primitives for concurrent programs: RwLock, Semaphore, and AtomicCounter.

These are OS-level blocking primitives (std::sync), suitable for multi-threaded use but not for cooperative async contexts. For async-safe shared state, use `asyncMutex` from the `async` namespace instead.

---

## Handle Model

`sync` handles are opaque array values tagged with an internal marker. Do not inspect or construct them directly. Handles remain valid for the lifetime of the program — there is no explicit destroy operation.

---

## RwLock

A reader-writer lock that allows concurrent readers or one exclusive writer. Multiple readers can hold the lock simultaneously; a writer gets exclusive access.

### rwLockNew

```atlas
rwLockNew(initialValue: any): handle
```

Create a new RwLock wrapping `initialValue`.

```atlas
let lock = rwLockNew(0);
let lock = rwLockNew("initial state");
let lock = rwLockNew([1, 2, 3]);
```

### rwLockRead

```atlas
rwLockRead(handle: handle): any
```

Acquire a read lock, take a snapshot of the current value, and release the lock. Multiple callers can read concurrently. Throws if the lock is poisoned (a previous writer panicked while holding it).

```atlas
let value = rwLockRead(lock);
console.log(value.toString());
```

### rwLockWrite

```atlas
rwLockWrite(handle: handle, newValue: any): null
```

Acquire the exclusive write lock, replace the stored value with `newValue`, and release the lock. Blocks until no readers or writers are active. Throws if the lock is poisoned.

```atlas
rwLockWrite(lock, 42);
rwLockWrite(lock, rwLockRead(lock) + 1);
```

### rwLockTryRead

```atlas
rwLockTryRead(handle: handle): Option<any>
```

Attempt to acquire the read lock without blocking. Returns `Some(value)` if the lock was acquired, `None` if it is currently held by a writer.

```atlas
match rwLockTryRead(lock) {
    Some(v) => console.log("got: " + v.toString()),
    None => console.log("lock busy"),
}
```

### rwLockTryWrite

```atlas
rwLockTryWrite(handle: handle, newValue: any): bool
```

Attempt to acquire the write lock without blocking. Returns `true` and sets the value if successful, `false` if the lock is currently held.

```atlas
if rwLockTryWrite(lock, newValue) {
    console.log("updated");
} else {
    console.log("could not acquire write lock");
}
```

---

## Semaphore

A counting semaphore. Tracks a pool of `n` permits. `acquire` blocks until a permit is available; `release` returns one.

### semaphoreNew

```atlas
semaphoreNew(permits: number): handle
```

Create a semaphore with `permits` available slots. `permits` must be a positive integer greater than zero.

```atlas
let sem = semaphoreNew(5);  // max 5 concurrent holders
```

### semaphoreAcquire

```atlas
semaphoreAcquire(handle: handle): null
```

Acquire one permit, blocking until one is available.

```atlas
semaphoreAcquire(sem);
// do work
semaphoreRelease(sem);
```

### semaphoreTryAcquire

```atlas
semaphoreTryAcquire(handle: handle): bool
```

Attempt to acquire a permit without blocking. Returns `true` if a permit was obtained, `false` otherwise.

```atlas
if semaphoreTryAcquire(sem) {
    doWork();
    semaphoreRelease(sem);
} else {
    console.log("at capacity, skipping");
}
```

### semaphoreRelease

```atlas
semaphoreRelease(handle: handle): null
```

Release one permit back to the pool. Does not release more permits than the original capacity.

```atlas
semaphoreRelease(sem);
```

### semaphoreAvailable

```atlas
semaphoreAvailable(handle: handle): number
```

Return the number of currently available permits.

```atlas
let free = semaphoreAvailable(sem);
console.log(free.toString() + " permits available");
```

---

## AtomicCounter

A lock-free integer counter using sequential-consistency ordering. Suitable for simple counters shared across threads without a full mutex.

### atomicNew

```atlas
atomicNew(initial: number): handle
```

Create a new atomic integer with the given initial value.

```atlas
let counter = atomicNew(0);
```

### atomicLoad

```atlas
atomicLoad(handle: handle): number
```

Read the current value atomically.

```atlas
let n = atomicLoad(counter);
```

### atomicStore

```atlas
atomicStore(handle: handle, value: number): null
```

Store a value atomically.

```atlas
atomicStore(counter, 0);
```

### atomicAdd

```atlas
atomicAdd(handle: handle, delta: number): number
```

Atomically add `delta` to the counter. Returns the **previous** value (before the addition).

```atlas
let prev = atomicAdd(counter, 1);
console.log("was: " + prev.toString());
```

### atomicSub

```atlas
atomicSub(handle: handle, delta: number): number
```

Atomically subtract `delta`. Returns the **previous** value.

```atlas
let prev = atomicSub(counter, 1);
```

### atomicCompareExchange

```atlas
atomicCompareExchange(handle: handle, expected: number, desired: number): bool
```

Atomically compare the current value to `expected`. If equal, replace it with `desired` and return `true`. If not equal, leave the value unchanged and return `false`.

```atlas
let ok = atomicCompareExchange(counter, 0, 1);
if ok {
    console.log("CAS succeeded: counter is now 1");
} else {
    console.log("CAS failed: counter was not 0");
}
```

---

## Patterns

### Concurrency-limited execution

```atlas
let sem = semaphoreNew(3);  // at most 3 concurrent workers

fn worker(borrow id: number): void {
    semaphoreAcquire(sem);
    console.log("worker " + id.toString() + " started");
    // ... work ...
    semaphoreRelease(sem);
}
```

### Shared read-heavy state

```atlas
let config = rwLockNew("initial config");

// Many tasks can read simultaneously
fn getConfig(): string {
    return rwLockRead(config);
}

// Only one task writes at a time
fn reloadConfig(borrow newConfig: string): void {
    rwLockWrite(config, newConfig);
}
```

### Lock-free counter

```atlas
let hits = atomicNew(0);

fn recordHit(): void {
    atomicAdd(hits, 1);
}

fn getHitCount(): number {
    return atomicLoad(hits);
}
```

### Try-lock with fallback

```atlas
let lock = rwLockNew(0);

fn tryUpdate(borrow value: number): bool {
    return rwLockTryWrite(lock, value);
}

// In a hot loop — skip update if locked rather than block
for i in [1, 2, 3, 4, 5] {
    if !tryUpdate(i) {
        console.log("skipped " + i.toString());
    }
}
```
