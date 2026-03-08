# Async and Concurrency Functions

Futures, channels, timers, and task management.

## Futures

### futureNew

```atlas
fn futureNew(executor: fn() -> T) -> Future<T>
```

Creates a new future that will be resolved by executor function.

**Parameters:**
- `executor` - Function that returns value

**Returns:** `Future<T>` - Pending future

### futureResolve

```atlas
fn futureResolve(value: T) -> Future<T>
```

Creates an immediately resolved future.

**Parameters:**
- `value` - Value to resolve with

**Returns:** `Future<T>` - Resolved future

### futureReject

```atlas
fn futureReject(error: E) -> Future<T>
```

Creates an immediately rejected future.

**Parameters:**
- `error` - Error value

**Returns:** `Future<T>` - Rejected future

## Future Chaining

### futureThen

```atlas
fn futureThen(future: Future<T>, callback: fn(T) -> U) -> Future<U>
```

Chains computation on resolved value.

**Parameters:**
- `future` - Future to chain on
- `callback` - Function to call with resolved value

**Returns:** `Future<U>` - New future with transformed value

### futureCatch

```atlas
fn futureCatch(future: Future<T>, handler: fn(E) -> T) -> Future<T>
```

Handles rejection with fallback value.

**Parameters:**
- `future` - Future that may reject
- `handler` - Function to call on error

**Returns:** `Future<T>` - Future that recovers from error

## Future Combinators

### futureRace

```atlas
fn futureRace(futures: Future<T>[]) -> Future<T>
```

Returns value of first completing future.

**Parameters:**
- `futures` - Array of futures

**Returns:** `Future<T>` - Resolves with first completed value

### futureAll

```atlas
fn futureAll(futures: Future<T>[]) -> Future<[]T>
```

Waits for all futures to complete.

**Parameters:**
- `futures` - Array of futures

**Returns:** `Future<[]T>` - Array of all resolved values

**Errors:** Rejects if any future rejects

## Future Status

### futureIsResolved

```atlas
fn futureIsResolved(future: Future<T>) -> bool
```

Checks if future is resolved.

**Parameters:**
- `future` - Future to check

**Returns:** `bool`

### futureIsRejected

```atlas
fn futureIsRejected(future: Future<T>) -> bool
```

Checks if future is rejected.

**Parameters:**
- `future` - Future to check

**Returns:** `bool`

### futureIsPending

```atlas
fn futureIsPending(future: Future<T>) -> bool
```

Checks if future is still pending.

**Parameters:**
- `future` - Future to check

**Returns:** `bool`

## Await

### await

```atlas
fn await(future: Future<T>) -> Result<T, E>
```

Waits for future to complete synchronously.

**Parameters:**
- `future` - Future to wait for

**Returns:**
- `Ok(T)` if resolved
- `Err(E)` if rejected

**Note:** Blocks current task

## Channels

### channelUnbounded

```atlas
fn channelUnbounded<T>() -> [Sender<T>, Receiver<T>]
```

Creates unbounded MPMC channel.

**Returns:** `[sender, receiver]` tuple

### channelBounded

```atlas
fn channelBounded<T>(capacity: number) -> [Sender<T>, Receiver<T>]
```

Creates bounded MPMC channel.

**Parameters:**
- `capacity` - Buffer capacity (integer)

**Returns:** `[sender, receiver]` tuple

### channelSend

```atlas
fn channelSend(sender: Sender<T>, value: T) -> Result<Null, string>
```

Sends value through channel.

**Parameters:**
- `sender` - Channel sender
- `value` - Value to send

**Returns:**
- `Ok(Null)` on success
- `Err(string)` if channel closed

### channelReceive

```atlas
fn channelReceive(receiver: Receiver<T>) -> Result<Option<T>, string>
```

Receives value from channel.

**Parameters:**
- `receiver` - Channel receiver

**Returns:**
- `Ok(Option<T>)` - Received value or None if channel empty/closed
- `Err(string)` on error

### channelIsClosed

```atlas
fn channelIsClosed(sender: Sender<T>) -> bool
```

Checks if channel is closed.

**Parameters:**
- `sender` - Channel sender

**Returns:** `bool`

### channelSelect

```atlas
fn channelSelect(receivers: Receiver<T>[]) -> Result<T, string>
```

Waits for value on any channel.

**Parameters:**
- `receivers` - Array of receivers to wait on

**Returns:**
- `Ok(T)` - First received value
- `Err(string)` on error

## Timers

### sleep

```atlas
fn sleep(seconds: number) -> Future<Null>
```

Creates future that resolves after delay.

**Parameters:**
- `seconds` - Delay in seconds (can be fractional)

**Returns:** `Future<Null>` - Future that resolves after delay

### timer

```atlas
fn timer(interval: number) -> Future<Null>
```

Creates repeating timer.

**Parameters:**
- `interval` - Interval in seconds

**Returns:** `Future<Null>` - Future for each tick

### timeout

```atlas
fn timeout(future: Future<T>, seconds: number) -> Future<Result<T, string>>
```

Adds timeout to future.

**Parameters:**
- `future` - Future to timeout
- `seconds` - Timeout in seconds

**Returns:** `Future` - Resolves with value or rejects after timeout

### interval

```atlas
fn interval(seconds: number) -> Iterator<number>
```

Creates interval iterator.

**Parameters:**
- `seconds` - Interval in seconds

**Returns:** `Iterator<number>` - Counter starting from 0

## Synchronization

### asyncMutex

```atlas
fn asyncMutex<T>(initial: T) -> AsyncMutex<T>
```

Creates async-safe mutual exclusion lock.

**Parameters:**
- `initial` - Initial value

**Returns:** `AsyncMutex<T>` - Mutex guard

### asyncMutexGet

```atlas
fn asyncMutexGet(mutex: AsyncMutex<T>) -> Future<T>
```

Gets value from mutex (waits if locked).

**Parameters:**
- `mutex` - Async mutex

**Returns:** `Future<T>` - Future resolving to current value

### asyncMutexSet

```atlas
fn asyncMutexSet(mutex: AsyncMutex<T>, value: T) -> Future<Null>
```

Sets value in mutex (waits if locked).

**Parameters:**
- `mutex` - Async mutex
- `value` - New value

**Returns:** `Future<Null>` - Future completing when set

## Tasks

### taskId

```atlas
fn taskId() -> number
```

Gets ID of current task.

**Returns:** `number` - Unique task ID

### taskName

```atlas
fn taskName() -> string
```

Gets name of current task.

**Returns:** `string` - Task name

### taskStatus

```atlas
fn taskStatus(id: number) -> string
```

Gets status of task.

**Parameters:**
- `id` - Task ID

**Returns:** `string` - Status ("pending", "running", "completed", "cancelled")

### taskJoin

```atlas
fn taskJoin(id: number) -> Future<any>
```

Waits for task to complete.

**Parameters:**
- `id` - Task ID

**Returns:** `Future` - Future resolving to task result

### taskCancel

```atlas
fn taskCancel(id: number) -> Result<Null, string>
```

Cancels a task.

**Parameters:**
- `id` - Task ID

**Returns:**
- `Ok(Null)` on success
- `Err(string)` if already completed

## Example Usage

```atlas
// Simple future
let future = futureNew(|| { 42 });
let result = await(future)?;
print(result); // 42

// Sleep
let delay = sleep(1.5);
await(delay)?;
print("Done sleeping");

// Channel
let [tx, rx] = channelUnbounded();
channelSend(tx, "hello")?;
let msg = channelReceive(rx)?;
print(msg); // "hello"

// Future chaining
let mut future = futureResolve(5);
future = futureThen(future, fn(x: number) -> number { return x * 2; });
future = futureThen(future, fn(x: number) -> number { return x + 1; });
print(await(future)?); // 11

// Timeout
let slow = futureNew(|| { await(sleep(10))?; 42 });
let result = timeout(slow, 1)?;
// Rejects after 1 second
```
