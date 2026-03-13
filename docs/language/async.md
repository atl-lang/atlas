# Async and Concurrency

Atlas provides first-class async support through `async fn`, `await`, `Future<T>`, task spawning, channels, and async primitives. The runtime is backed by Tokio's multi-threaded executor.

---

## Overview

Atlas async has two layers:

1. **Language syntax** — `async fn` declarations and `await expr` expressions. Used for sequential async code.
2. **Stdlib primitives** — `task.spawn()`, channels, timers, `Future<T>` combinators. Used for concurrent tasks and coordination.

The underlying runtime is Tokio. Tasks run cooperatively on a work-stealing thread pool. Each spawned task costs ~100 bytes of overhead (not an OS thread).

---

## Async Functions

### Declaration

```atlas
async fn fetchUser(id: number): Future<string> {
    // body
}
```

An `async fn` returns `Future<T>` where `T` is the declared return type. The function body can use `await` to suspend until a future resolves.

### Return Type

The return type annotation on an async function is the **resolved type**, not `Future<T>` explicitly:

```atlas
async fn readFile(path: string): string {
    // returns string when awaited
}
```

At the type-system level, the actual type of calling `readFile(path)` is `Future<string>`.

### Exporting Async Functions

```atlas
export async fn computeSum(a: number, b: number): Future<number> {
    let result = await someAsyncOperation(a, b);
    return result;
}
```

---

## Await Expression

Use `await` to suspend the current async function until a `Future<T>` resolves:

```atlas
async fn main(): void {
    let data = await fetchUser(42);
    console.log(data);
}
```

`await` is an expression — it evaluates to the resolved value of the future. It can be used inline:

```atlas
let length = (await readFile("data.txt")).length;
```

`await` is only valid inside `async fn` bodies. Using it outside an async function is a parse error.

---

## Future Type

`Future<T>` is a first-class type representing a pending computation that will produce a value of type `T`.

### Future States

A future is always in one of three states:
- **Pending** — computation in progress
- **Resolved** — computation completed with a value
- **Rejected** — computation failed with an error

### Creating Futures (stdlib)

```atlas
// Already resolved
let f = futureResolve(42);

// Already rejected
let f = futureReject("error message");

// New pending future
let f = futureNew();
```

### Checking State (stdlib)

```atlas
futureIsPending(f);   // bool
futureIsResolved(f);  // bool
futureIsRejected(f);  // bool
```

### Combining Futures (stdlib)

```atlas
// Wait for all to resolve (rejects if any reject)
let all = futureAll([f1, f2, f3]);

// Adopt the first to settle (resolve or reject)
let first = futureRace([f1, f2, f3]);

// Wait for all, collecting results including rejections (never rejects)
let settled = future.allSettled([f1, f2, f3]);

// Resolve with the first successful result
let any = future.any([f1, f2, f3]);

// Never resolves (permanently pending)
let never = future.never();

// Delay before resolving
let delayed = future.delay(1000);
```

---

## Task Spawning

Tasks run concurrently on the worker pool. Each task is an independent unit of work.

### task.spawn()

```atlas
import { spawn, taskJoin, taskStatus } from "task";

// Spawn an async function
let handle = task.spawn(fn(): void {
    // async work
});

// Spawn with a name (for diagnostics)
let handle = task.spawn(myFunction, "worker-1");

// Spawn a Future
let future = fetchData(url);
let handle = task.spawn(future);
```

`task.spawn()` accepts:
- A `Future<T>` — drives the future to completion on a worker
- A `fn` or closure — executes the function on a worker's VM context
- An optional name string as the second argument

Returns a `TaskHandle`.

### TaskHandle

```atlas
let handle = task.spawn(myWork);

// Check status
let status = taskStatus(handle);  // "Running" | "Completed" | "Cancelled" | "Failed"

// Get task ID
let id = taskId(handle);          // number

// Get task name
let name = taskName(handle);      // string | null

// Cancel
taskCancel(handle);               // requests cancellation

// Await completion
let result = await taskJoin(handle);  // Future<T>
```

### Joining Multiple Tasks

```atlas
let h1 = task.spawn(work1);
let h2 = task.spawn(work2);
let h3 = task.spawn(work3);

let all = joinAll([h1, h2, h3]);  // Future<T[]>
let results = await all;
```

`joinAll` returns a `Future` that resolves when all handles complete, with an array of results. Rejects immediately if any task fails.

### Blocking Tasks (CPU-bound Work)

For CPU-heavy or blocking I/O work that would stall cooperative tasks:

```atlas
let handle = spawnBlocking(heavyComputation);
let handle = spawnBlocking(heavyComputation, "crunch-worker");
```

`spawnBlocking` uses Tokio's dedicated blocking thread pool. Unlike cooperative `task.spawn()`, blocking tasks run on separate OS threads and do not prevent other async tasks from executing.

---

## Timers and Sleep

### sleep

```atlas
await sleep(1000);  // sleep for 1 second
```

`sleep(milliseconds: number): Future<null>` — resolves to `null` after the delay.

Inside an async context (worker LocalSet task), sleep yields back to the executor so other tasks can run. Outside async context, the current thread blocks.

### timer

```atlas
let t = timer(500);  // one-shot timer
await t;
```

Equivalent to `sleep` — models an explicit one-shot timer.

### interval

```atlas
// Polling pattern — call interval() again for each tick
await interval(100);
doWork();
await interval(100);
doWork();
```

`interval` is equivalent to `sleep` — the caller models recurring ticks by calling it in a loop.

---

## Timeout

Wrap any future with a deadline:

```atlas
let result = timeout(fetchData(url), 5000);  // 5 second deadline

match await result {
    Ok(data) => console.log(data),
    Err(e) => console.log("timed out or failed"),
}
```

`timeout(future: Future<T>, milliseconds: number): Future<T>` — resolves with the original value, or rejects with `"Operation timed out"` if the deadline passes.

---

## Channels

Channels provide safe message passing between tasks.

### Unbounded Channel

```atlas
let (sender, receiver) = channelUnbounded();

// In one task
channelSend(sender, "hello");

// In another task
let msg = await channelReceive(receiver);
```

### Bounded Channel (Backpressure)

```atlas
let (sender, receiver) = channelBounded(10);  // capacity 10
```

Bounded channels apply backpressure when full — `channelSend` returns `false` if the channel is at capacity.

### Checking Send Result

```atlas
let ok = channelSend(sender, value);  // bool — false if channel full or closed
let closed = channelIsClosed(sender); // bool
```

### Select (Multiple Channels)

```atlas
// Wait for first message from any of these receivers
let result = await channelSelect([rx1, rx2, rx3]);
// result is [value, index] — the value and which channel it came from
```

---

## Async Mutex

For shared mutable state between tasks:

```atlas
let mutex = asyncMutex(0);  // wrap a value

// Read
let value = asyncMutexGet(mutex);

// Write
asyncMutexSet(mutex, value + 1);
```

The async mutex is safe to use from multiple tasks. Access is serialized — no two tasks hold the lock simultaneously.

---

## Error Handling in Async Code

Async functions return `Future<T>`. When a task fails, the `TaskHandle` transitions to `Failed` state and the result carries the error message.

Use the `?` operator in async functions to propagate `Result` errors:

```atlas
async fn readConfig(path: string): Result<string, string> {
    let content = file.read(path)?;  // propagates Err early
    return Ok(content);
}
```

For Future rejection, use `match` on the awaited result or check `futureIsRejected`:

```atlas
let result = await riskyOperation();
if futureIsRejected(result) {
    console.log("operation failed");
}
```

---

## Full Example

```atlas
import { spawn, taskJoin, joinAll } from "task";

async fn fetchUser(id: number): Future<string> {
    await sleep(100);  // simulate network delay
    return `User-${id}`;
}

async fn main(): void {
    // Spawn multiple concurrent tasks
    let handles = [
        task.spawn(fetchUser(1)),
        task.spawn(fetchUser(2)),
        task.spawn(fetchUser(3)),
    ];

    // Wait for all to complete
    let results = await joinAll(handles);
    for name in results {
        console.log(name);
    }
}
```

---

## stdlib Reference

| Function | Signature | Description |
|----------|-----------|-------------|
| `task.spawn` | `(callable, name?) -> TaskHandle` | Spawn async task |
| `spawnBlocking` | `(fn, name?) -> TaskHandle` | Spawn blocking task |
| `taskJoin` | `(handle) -> Future<T>` | Await task completion |
| `taskStatus` | `(handle) -> string` | Get task status |
| `taskCancel` | `(handle) -> null` | Request cancellation |
| `taskId` | `(handle) -> number` | Get task ID |
| `taskName` | `(handle) -> string \| null` | Get task name |
| `joinAll` | `(handles[]) -> Future<T[]>` | Wait for all tasks |
| `sleep` | `(ms) -> Future<null>` | Sleep for duration |
| `timer` | `(ms) -> Future<null>` | One-shot timer |
| `interval` | `(ms) -> Future<null>` | Interval tick |
| `timeout` | `(future, ms) -> Future<T>` | Wrap with deadline |
| `futureResolve` | `(value) -> Future<T>` | Already-resolved future |
| `futureReject` | `(error) -> Future<never>` | Already-rejected future |
| `futureNew` | `() -> Future<T>` | New pending future |
| `futureAll` | `(Future<T>[]) -> Future<T[]>` | Wait for all |
| `futureRace` | `(Future<T>[]) -> Future<T>` | First to settle |
| `future.allSettled` | `(Future<T>[]) -> Future<T[]>` | All results, no rejection |
| `future.any` | `(Future<T>[]) -> Future<T>` | First resolved |
| `future.never` | `() -> Future<T>` | Never resolves |
| `future.delay` | `(ms) -> Future<null>` | Delay then resolve |
| `channelUnbounded` | `() -> [Sender, Receiver]` | Unbounded channel |
| `channelBounded` | `(capacity) -> [Sender, Receiver]` | Bounded channel |
| `channelSend` | `(sender, value) -> bool` | Send message |
| `channelReceive` | `(receiver) -> Future<T>` | Receive message |
| `channelSelect` | `(receivers[]) -> Future<[T, number]>` | Select from channels |
| `channelIsClosed` | `(sender) -> bool` | Check if closed |
| `asyncMutex` | `(value) -> AsyncMutex` | Create async mutex |
| `asyncMutexGet` | `(mutex) -> T` | Read locked value |
| `asyncMutexSet` | `(mutex, value) -> null` | Update locked value |

---

## Gotchas

**`await` is only valid inside `async fn`.** The parser requires `await` to appear inside an async function body. Using it at the top level or in a regular function is a syntax error.

**`task.spawn()` requires the worker pool to be initialized.** In production code (via `atlas run`), the pool is initialized automatically. In tests or custom embeddings, call `init_worker_pool()` at startup.

**Cancellation is cooperative and pre-start only.** Calling `taskCancel(handle)` sets a cancellation flag that is checked before the task body begins executing. Mid-execution cancellation is not yet implemented — the task runs to completion if already started.

**`spawnBlocking` is for long-running or blocking work.** Using `task.spawn` for CPU-heavy work starves cooperative I/O tasks. Use `spawnBlocking` when the work will take more than ~1 ms or makes blocking system calls.

**Channels are message-passing, not shared memory.** Values sent through channels are moved. If you need shared mutable state, use `asyncMutex`.

**`futureResolve` and `futureReject` create already-settled futures.** They do not perform any async work — they're constructors for the resolved/rejected states, useful for returning from sync code that wraps async APIs.

**`futureThen` and `futureCatch` with dynamic handlers are not yet implemented.** These stdlib functions exist but return an error at runtime if called. Use `await` and `match` instead for chaining async operations.
