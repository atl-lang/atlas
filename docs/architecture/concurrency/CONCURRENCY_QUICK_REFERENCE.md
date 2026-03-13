# Atlas Concurrency — Quick Reference

**Implementation as of B44 (complete).**
Facts only — no speculation, no roadmap items.

---

## Core Types

### AtlasFuture

**Source:** `async_runtime/future.rs`

```rust
pub struct AtlasFuture {
    inner: Arc<Mutex<AtlasFutureInner>>,
}
// AtlasFutureInner: { state: FutureState, wakers: Vec<Waker> }

pub enum FutureState { Pending, Resolved(Value), Rejected(Value) }
```

Implements `std::future::Future<Output = Result<Value, Value>>`. Resolved → `Poll::Ready(Ok(value))`, Rejected → `Poll::Ready(Err(error))`. Wakers are notified immediately on `resolve()` / `reject()`, making it compatible with Tokio and any `std::future` executor.

Constructors: `new_pending()`, `resolved(value)`, `rejected(error)`.

The VM stores `AtlasFuture` as `Value::Future(AtlasFuture)`.

### TaskHandle

**Source:** `async_runtime/task.rs`

```rust
pub struct TaskHandle {
    state: Arc<TaskState>,  // { id, name, status, cancelled, result }
}
pub enum TaskStatus { Running, Completed, Cancelled, Failed }
```

- `id()` — unique monotonic u64
- `status()` — current lifecycle state
- `cancel()` — sets cancelled flag; cooperative, checked before task body starts
- `join()` — returns `AtlasFuture` (resolved if done, pending if still running)
- `is_pending/completed/cancelled/failed()` — status predicates

### WorkerPool

**Source:** `async_runtime/worker.rs`

```rust
pub struct WorkerPool {
    workers: Vec<Worker>,    // One OS thread each
    scheduler: Scheduler,
}
```

Singleton via `static WORKER_POOL: OnceLock<WorkerPool>`. Access via `worker_pool() -> Option<&'static WorkerPool>`.

`submit(task: WorkerTask) -> Result<(), WorkerTask>` — injects into `Scheduler`, wakes a worker round-robin.

### Scheduler

**Source:** `async_runtime/scheduler.rs`

```rust
pub struct Scheduler {
    injector: Arc<Injector<WorkerTask>>,     // crossbeam_deque global queue
    stealers: Arc<Vec<Stealer<WorkerTask>>>, // One per worker, Send
    wakers: Arc<Vec<mpsc::Sender<()>>>,      // Per-worker wakeup channel
    next_wakeup: Arc<AtomicUsize>,           // Round-robin counter
}
```

`find_task(id, local)` — priority order:
1. Own local deque (`local.pop()`)
2. Injector (`injector.steal_batch_and_pop(local)`)
3. Random other worker (`stealers[random].steal()`)

Returns `Found(task)`, `Retry` (contention — yield and try again), or `Empty` (sleep up to 500µs).

### Worker / WorkerContext

**Source:** `async_runtime/worker.rs`

```rust
pub struct WorkerContext {
    pub vm: RefCell<VM>,  // Isolated VM for this worker (no Send required)
    pub worker_id: usize,
}
```

Each worker OS thread runs:
- A `tokio::runtime::Builder::new_current_thread()` + `LocalSet` for cooperative scheduling
- A `crossbeam_deque::Worker<WorkerTask>` local deque (`!Send`, stays on this thread)
- A `mpsc::Receiver<()>` wakeup channel

Worker event loop (simplified):
```rust
loop {
    match scheduler.find_task(id, &local_deque) {
        Found(task) => { spawn_local(task(ctx)); yield_now().await; }
        Retry => { yield_now().await; }
        Empty => {
            // Sleep ≤ 500µs, woken by Scheduler::push or timed out
            if channel_closed { break; }
        }
    }
}
```

`WorkerTask` type: `Box<dyn FnOnce(Rc<WorkerContext>) -> Pin<Box<dyn Future<Output=()>>> + Send + 'static>`

The closure is `Send` (travels across the MPSC channel). The returned future is `!Send + 'static` (runs inside the worker's `LocalSet`). `Rc<WorkerContext>` is `!Send` intentionally — tasks cannot be migrated mid-execution.

---

## Task Spawning Call Paths

### spawn_task(future, name) → TaskHandle

For futures. Wraps the future in a `WorkerTask` closure and submits to the pool. The task catches panics via `AssertUnwindSafe + catch_unwind`. Result is written to `Arc<TaskState>` which the `TaskHandle` observes.

### spawn_function_task(callable, args, name) → TaskHandle

For Atlas `Function` or `Closure` values. Builds a `FunctionTask` with a `oneshot::Sender`. The worker's closure calls `ctx.vm.borrow_mut().execute_task_function(callable, args, &security)`. Result is sent on the oneshot channel. A bridge OS thread waits on the channel and writes to `Arc<TaskState>`.

`callable: FunctionCallable` is either `Function(FunctionRef)` or `Closure(ClosureRef)`.

### spawn_blocking_task(callable, args, name) → TaskHandle

For CPU-bound or blocking-I/O work. Uses `tokio::task::spawn_blocking` on the global multi-thread runtime. Each blocking task gets an isolated VM cloned from `BLOCKING_BASE_VM` (registered via `init_blocking_pool`). Runs in parallel across Tokio's blocking thread pool.

### spawn_and_await(future) → Result<Value, String>

Bridges sync → async. Blocks the calling thread via `block_on(future)`. Used by stdlib operations that need async I/O without language-level `await`.

---

## VM Isolation Model

Each worker VM is created via `VM::new_for_worker()`:
- Clones `bytecode` (read-only, shared content via Arc)
- Clones `globals: HashMap<String, Value>` — **independent copy per worker**
- Fresh `VMContext` (empty stack, single main frame)
- `jit: None` — JIT is not inherited by workers

**Workers do NOT share globals at runtime.** Mutations in a spawned task are local to that worker's VM instance. Cross-worker communication uses channels.

`Value` is `Send` — enforced by a compile-time assertion in `async_runtime/mod.rs`. This is required for values to travel through `Injector<WorkerTask>`.

---

## Async Opcodes (VM → Runtime Bridge)

| Opcode | What happens |
|--------|-------------|
| `AsyncCall(fn_idx, argc)` | Calls async fn, wraps result in `Value::Future` |
| `Await` | Blocks via `block_on(future)` until resolved, pushes inner value |
| `WrapFuture` | Wraps any value in an already-resolved `AtlasFuture` |
| `SpawnTask(fn_idx, argc)` | Calls `spawn_function_task`, pushes `Value::Future` handle |

---

## Primitives

**Source:** `async_runtime/primitives.rs`

- `sleep(duration_ms: f64) -> Value::Null` — `tokio::time::sleep` via `block_on`
- `timeout(duration_ms: f64, future) -> Value::Result` — `tokio::time::timeout`
- `timer(interval_ms: f64) -> AtlasFuture` — resolves after interval
- `interval(duration_ms: f64, callback: Function) -> TaskHandle` — repeating timer
- `retry_with_timeout(fn, max_attempts, delay_ms) -> AtlasFuture` — retries with backoff
- `AsyncMutex` — wraps `tokio::sync::Mutex<Value>`

---

## Channels

**Source:** `async_runtime/channel.rs`

- `channel_bounded(capacity: usize) -> (ChannelSender, ChannelReceiver)` — bounded MPSC
- `channel_unbounded() -> (ChannelSender, ChannelReceiver)` — unbounded MPSC
- `channel_select(receivers) -> Value` — non-deterministic receive from multiple channels

`ChannelSender` wraps `tokio::sync::mpsc::Sender<Value>`.
`ChannelReceiver` wraps `tokio::sync::mpsc::Receiver<Value>`.

---

## Atlas-Level API

```atlas
// Spawn a concurrent task (returns a Future/TaskHandle)
let handle = task.spawn(myFn, arg1, arg2);

// Await the result
let result = await handle;

// Async function declaration
async fn fetchData(url: string): string {
    // ...
}

// Await an async call
let data = await fetchData("https://...");
```

`task.spawn()` in stdlib calls `spawn_function_task(FunctionCallable::Function(ref), args, None)` and returns `Value::Future`.

---

## Error Handling

| Condition | Outcome |
|-----------|---------|
| Worker pool not initialized | `TaskHandle` with `Failed` status, error message |
| Worker pool channel full (backpressure) | `TaskHandle` with `Failed` status |
| Task panics | Caught via `catch_unwind`, status → `Failed`, panic message stored |
| Task cancelled before start | Status → `Cancelled` immediately |
| `Await` on non-Future value | Runtime error AT4002 |
| Worker shuts down mid-task | oneshot channel drops, bridge thread sets `Failed` |

---

## Memory Model

| Thing | Overhead |
|-------|----------|
| `WorkerTask` closure | ~100 bytes + captured data |
| `AtlasFuture` | `Arc<Mutex<...>>` ≈ ~128 bytes |
| `TaskHandle` + `TaskState` | `Arc<Mutex<...>>` ≈ ~256 bytes |
| Worker thread | OS thread stack (~8 MB on macOS) + isolated VM |
| VM clone per worker | `bytecode` (Arc clone, cheap) + `globals` (HashMap clone) |

Worker count defaults to `thread::available_parallelism()` (one per logical CPU core).

---

## Key Invariants

1. `Value: Send` — enforced at compile time. Non-`Send` types cannot appear in `Value`.
2. One `LocalSet` per worker — tasks are `!Send` futures; they never migrate mid-execution.
3. Workers have isolated VM instances — no shared mutable state between workers.
4. `WorkerTask` closures are `Send + 'static` — required to cross thread boundaries via `Injector`.
5. `RefCell<VM>` in `WorkerContext` is safe — `LocalSet` is cooperative, only one task runs at a time per worker.
6. `WORKER_POOL` is a singleton — `init_worker_pool` panics on double-call.
