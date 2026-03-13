# async namespace

Concurrency primitives: task spawning, channels, timers, timeouts, and Future combinators.

Atlas's async model is built on Tokio. Tasks run cooperatively on a LocalSet work-stealing scheduler. `spawn()` and `await` are the primary entry points.

---

## Futures

A `Future<T>` is an opaque value representing a computation that may not have completed yet. Futures have three states: `Pending`, `Resolved`, and `Rejected`.

### futureResolve

```atlas
futureResolve(value: T): Future<T>
```

Create an already-resolved future.

```atlas
let f = futureResolve(42);
```

### futureReject

```atlas
futureReject(error: T): Future<never>
```

Create an already-rejected future.

```atlas
let f = futureReject("something went wrong");
```

### futureNew

```atlas
futureNew(): Future<T>
```

Create a new pending future. Primarily useful for testing.

### futureIsPending

```atlas
futureIsPending(future: Future<T>): bool
```

Return `true` if the future has not yet settled.

### futureIsResolved

```atlas
futureIsResolved(future: Future<T>): bool
```

Return `true` if the future resolved successfully.

### futureIsRejected

```atlas
futureIsRejected(future: Future<T>): bool
```

Return `true` if the future was rejected.

### futureAll

```atlas
futureAll(futures: Future<T>[]): Future<T[]>
```

Combine multiple futures into one. Resolves when all input futures resolve (with an array of results in the same order). Rejects if any future rejects.

```atlas
let f1 = httpGetAsync("https://api.example.com/a");
let f2 = httpGetAsync("https://api.example.com/b");
let both = await futureAll([f1, f2]);
```

### futureRace

```atlas
futureRace(futures: Future<T>[]): Future<T>
```

Return the first future to settle (either resolved or rejected).

```atlas
let result = await futureRace([slowOp(), fastOp()]);
```

### future.allSettled

```atlas
future.allSettled(futures: Future<T>[]): Future<T[]>
```

Wait for all futures to settle. Never rejects — rejected futures contribute their error value to the result array.

```atlas
let results = await future.allSettled([f1, f2, f3]);
```

### future.any

```atlas
future.any(futures: Future<T>[]): Future<T>
```

Resolve with the first successfully-resolved future. Rejects only if all futures reject.

```atlas
let winner = await future.any([primary(), fallback()]);
```

### future.never

```atlas
future.never(): Future<T>
```

Return a future that never resolves (always stays pending). Useful for testing timeout logic.

```atlas
let f = future.never();
```

### future.delay

```atlas
future.delay(ms: number): Future<null>
```

Return a future that resolves after `ms` milliseconds. In the current synchronous evaluation context, resolves immediately; real delay requires `await` in an async context.

---

## Task Spawning

### spawn

```atlas
spawn(callable: Future<T> | fn | closure, name?: string | null): TaskHandle
```

Spawn an async task. Accepts a `Future`, a named function, or a closure. The optional `name` is for diagnostics.

- **Future argument:** drives the future to completion on a LocalSet worker.
- **Function/Closure argument:** dispatched as a `FunctionTask`.

Returns a `TaskHandle` that can be joined, cancelled, or inspected.

```atlas
let handle = spawn(fn(): void {
    console.log("running in background");
});

// Named task
let handle = spawn(worker, "data-processor");
```

### spawnBlocking

```atlas
spawnBlocking(callable: fn | closure, name?: string | null): TaskHandle
```

Spawn a CPU-bound or blocking-I/O function on Tokio's dedicated blocking thread pool. This keeps cooperative async tasks from being starved. Only accepts functions or closures (not futures).

```atlas
let handle = spawnBlocking(fn(): void {
    // heavy computation
    compute(10_000_000);
});
```

---

## Task Management

### taskJoin

```atlas
taskJoin(handle: TaskHandle): Future<T>
```

Return a future that resolves when the task completes.

```atlas
let handle = spawn(worker);
let result = await taskJoin(handle);
```

### taskStatus

```atlas
taskStatus(handle: TaskHandle): string
```

Return the current task status as a string: `"Running"`, `"Completed"`, `"Cancelled"`, or `"Failed"`.

```atlas
let status = taskStatus(handle);
if status == "Completed" {
    console.log("done");
}
```

### taskCancel

```atlas
taskCancel(handle: TaskHandle): null
```

Request cancellation of the task.

```atlas
taskCancel(handle);
```

### taskId

```atlas
taskId(handle: TaskHandle): number
```

Return the numeric ID of the task.

```atlas
let id = taskId(handle);
```

### taskName

```atlas
taskName(handle: TaskHandle): string | null
```

Return the name of the task, or `null` if none was given.

```atlas
let name = taskName(handle);
```

### joinAll

```atlas
joinAll(handles: TaskHandle[]): Future<T[]>
```

Join an array of task handles, returning a future that resolves with an array of results when all tasks complete.

```atlas
let h1 = spawn(worker1);
let h2 = spawn(worker2);
let results = await joinAll([h1, h2]);
```

---

## Channels

Channels provide typed message passing between tasks. `channelUnbounded` and `channelBounded` both return `[ChannelSender, ChannelReceiver]`.

### channelUnbounded

```atlas
channelUnbounded(): [ChannelSender, ChannelReceiver]
```

Create an unbounded channel. Senders never block, but the buffer can grow without limit.

```atlas
let ch = channelUnbounded();
let sender = ch[0];
let receiver = ch[1];
```

### channelBounded

```atlas
channelBounded(capacity: number): [ChannelSender, ChannelReceiver]
```

Create a bounded channel with the given buffer capacity (must be a positive integer).

```atlas
let ch = channelBounded(100);
let sender = ch[0];
let receiver = ch[1];
```

### channelSend

```atlas
channelSend(sender: ChannelSender, value: T): bool
```

Send a value through the channel. Returns `true` on success, `false` if the channel is closed.

```atlas
let ok = channelSend(sender, "hello");
```

### channelReceive

```atlas
channelReceive(receiver: ChannelReceiver): Future<T>
```

Receive the next value from the channel. Returns a future that resolves when a message is available.

```atlas
let value = await channelReceive(receiver);
```

### channelSelect

```atlas
channelSelect(receivers: ChannelReceiver[]): Future<[value, index]>
```

Select from multiple channels. Resolves with a two-element array `[value, index]` where `index` is the zero-based index of the receiver that produced the value.

```atlas
let ch1 = channelUnbounded();
let ch2 = channelUnbounded();
let result = await channelSelect([ch1[1], ch2[1]]);
let value = result[0];
let from = result[1];
```

### channelIsClosed

```atlas
channelIsClosed(sender: ChannelSender): bool
```

Return `true` if all receivers for this sender have been dropped.

```atlas
if channelIsClosed(sender) {
    console.log("no readers left");
}
```

---

## Timers and Sleep

### sleep

```atlas
sleep(milliseconds: number): Future<null>
```

Pause execution for `milliseconds`. Must be non-negative. Yields back to the Tokio runtime so other tasks can run.

```atlas
await sleep(1000);  // sleep 1 second
```

### timer

```atlas
timer(milliseconds: number): Future<null>
```

Alias for `sleep`. Returns a future that resolves after the given duration.

```atlas
let t = timer(500);
await t;
```

### interval

```atlas
interval(milliseconds: number): Future<null>
```

Create a repeating interval. The duration must be positive (greater than 0). Use inside a loop to poll on a schedule.

```atlas
while true {
    await interval(5000);
    checkHealth();
}
```

---

## Timeout

### timeout

```atlas
timeout(future: Future<T>, milliseconds: number): Future<T>
```

Wrap a future with a timeout. If the future does not resolve within `milliseconds`, the wrapping future rejects.

```atlas
let result = await timeout(httpGetAsync("https://slow.example.com"), 3000);
```

---

## Async Mutex

The `asyncMutex` is a Tokio-based mutex safe to use from async contexts (unlike `sync.md`'s blocking `RwLock`).

### asyncMutex

```atlas
asyncMutex(value: T): AsyncMutex
```

Create an async mutex wrapping an initial value.

```atlas
let counter = asyncMutex(0);
```

### asyncMutexGet

```atlas
asyncMutexGet(mutex: AsyncMutex): T
```

Acquire the lock and return the current value (blocking-safe from both sync and async contexts).

```atlas
let current = asyncMutexGet(counter);
```

### asyncMutexSet

```atlas
asyncMutexSet(mutex: AsyncMutex, value: T): null
```

Acquire the lock and replace the value.

```atlas
asyncMutexSet(counter, asyncMutexGet(counter) + 1);
```

---

## Patterns

### Producer-consumer pipeline

```atlas
let ch = channelBounded(50);
let producer = ch[0];
let consumer = ch[1];

// producer task
let writer = spawn(fn(): void {
    for i in [1, 2, 3, 4, 5] {
        channelSend(producer, i);
    }
});

// consumer loop
while true {
    let val = await channelReceive(consumer);
    console.log("got: " + val.toString());
}
```

### Timeout pattern

```atlas
fn fetchWithTimeout(borrow url: string): Result<string, string> {
    let f = httpGetAsync(url);
    let timed = timeout(f, 5000);
    let resp = await timed;
    if futureIsRejected(timed) {
        return Err("timed out");
    }
    return Ok(httpBody(resp));
}
```

### Shared counter across tasks

```atlas
let shared = asyncMutex(0);

let t1 = spawn(fn(): void {
    asyncMutexSet(shared, asyncMutexGet(shared) + 1);
});
let t2 = spawn(fn(): void {
    asyncMutexSet(shared, asyncMutexGet(shared) + 1);
});

await joinAll([t1, t2]);
console.log(asyncMutexGet(shared).toString());  // 2
```
