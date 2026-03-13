# Lightweight Concurrency in Major Systems Languages: Research Report

**Date:** 2026-03-13
**Focus:** Execution models for parallel user-defined functions in non-thread-safe runtimes
**Audience:** Atlas architecture team

---

## Executive Summary

This report analyzes how major systems languages enable lightweight concurrent task execution when the runtime/VM is not thread-safe. We examine Go (goroutines), Rust (tokio), Erlang (BEAM), C# (.NET), Python (asyncio/gevent), Lua (coroutines), and Java (virtual threads).

**Key Finding:** There is a spectrum of approaches, from **process isolation** (Erlang) to **shared-memory with explicit synchronization** (Rust/Tokio), to **single-thread event loops** (Python asyncio). The choice depends on whether the runtime must support OS-level parallelism or cooperative concurrency.

For **Atlas specifically** (a bytecode VM that is `!Send + !Sync`): The **greenlet/fiber model with work-stealing** is most appropriate because:
1. It isolates concurrent tasks from each other
2. It avoids sharing the single VM state across threads
3. It provides efficient, memory-light task spawning
4. It leverages continuation-passing for clean semantics

---

## 1. Go: Goroutines and the M:N Scheduler

### Architecture Overview

Go implements **M:N threading**: M goroutines are multiplexed onto N OS threads via the Go scheduler. The scheduler is not thread-safe by design—each OS thread has its own scheduler context.

### Memory Efficiency

- **Per-goroutine cost:** ~2 KB initial stack (modern Go)
- **OS thread cost:** 1–8 MB stack (upfront allocation)
- **Comparison:** 10,000 goroutines ≈ 20 MB; 10,000 OS threads ≈ 20 GB
- **Stack growth:** Goroutine stacks grow/shrink dynamically via copying; no fixed limit

### Scheduler Design (GPM Model)

```
G (goroutine) - user task, ~2 KB
M (machine/thread) - OS thread, binds to P
P (processor/context) - scheduler context, local runqueue
```

When a goroutine is spawned (`go func()`):
1. Added to M's local runqueue (LIFO)
2. If local queue full → pushed to global runqueue
3. When M is idle → steals from other P's runqueues (FIFO from back)
4. **No locks on fast path:** local queue push/pop lock-free

### Thread-Safe Guarantees

- Each OS thread (M) has **non-overlapping state:** local runqueue, stack, cached memory (mcache)
- Stealing uses atomic operations + memory barriers
- **Potential issue:** mcache can consume ~2 MB per M, and if goroutines block on syscalls frequently, many Ms can exist, causing memory overhead

### Key Insight for VM Design

Go **isolates scheduler state per-thread** but shares `Value` (their heap objects) via **shared-memory concurrency**. Go forces users to protect shared data with `sync.Mutex`, channels, or atomic operations. The Go runtime itself doesn't serialize user code execution—two goroutines *can* run user code in parallel.

---

## 2. Rust: Tokio and Async/Await

### Architecture Overview

Tokio is an **async runtime** (not green threads). It uses:
- **Multi-threaded work-stealing executor** by default
- **Futures** (not stackful—continuations represented as state machines)
- **Send + 'static bounds** to ensure thread-safety

### Task Spawning and Send Bounds

```rust
tokio::spawn(async { ... })
```

**Requirements:**
- `F: Future + Send + 'static` — future must be safe to move between threads and must own all captured data
- Data shared across `.await` points must be `Send`
- Non-`Send` data (e.g., `Rc`) can exist *between* `.await` points but not *across* them

**Why?** Tokio's executor may suspend a task on one thread and resume it on another. Only `Send` futures can safely cross thread boundaries.

### Alternative: `spawn_local()`

For non-`Send` futures:
```rust
tokio::task::spawn_local(async { ... })
```

- Runs on thread-local `LocalSet`
- Tasks don't move between threads
- Allows non-`Send` types like `Rc`
- **Catch:** Must also provide an event loop (`LocalSet::run_until()`)

### Memory Efficiency

- **Per-task cost:** ~64 bytes (state machine overhead)
- **No per-task stack:** Stack is shared, reused via polling
- **Scalability:** 1M+ concurrent tasks on a single multi-threaded executor

### Key Insight for VM Design

Tokio proves that you **don't need OS threads for massive concurrency**. The trick is:
1. Make tasks `Send` (no shared mutable state, use `Arc<Mutex<T>>` for sharing)
2. Use work-stealing across threads for load balance
3. Futures are stackless—state saved in a heap-allocated state machine

**Problem for Atlas:** Atlas's `Value` type is `Send` (D-030), but the **VM itself** is not—it's held in `RefCell<Option<VM>>`. You can't safely run the bytecode across multiple OS threads without locks.

---

## 3. Erlang BEAM: Process Isolation

### Architecture Overview

Erlang processes are **completely isolated** with **no shared memory**. Each process has:
- Its own heap (2 KB minimum)
- Its own stack
- Its own state
- **Communication via immutable message passing only**

### Scheduling Model

- **Reductions:** Unit of work (function call, arithmetic, etc.)
- **Preemption:** Scheduler checks reduction count, preempts when exceeded
- **Per-core schedulers:** One scheduler per CPU core, one Erlang process per scheduler at a time
- **True parallelism:** Multiple processes run in parallel on multiple cores

### Memory Efficiency

- **Per-process:** 2 KB (heap) + stack (grows on demand)
- **Millions of processes:** Feasible on modern hardware
- **No GC pause from concurrency:** Each process' GC is independent

### Key Insight for VM Design

Erlang's **process isolation** completely eliminates synchronization. There's no race condition because there's no shared mutable state. However:
1. Requires **deep language support** (pattern matching, message queues)
2. Immutable data structures throughout
3. Not suitable for "spawn user function" semantics—messages are data, not arbitrary closures

---

## 4. C# / .NET: ThreadPool and Task Scheduling

### Architecture Overview

C# combines:
- **ThreadPool:** Fixed number of reusable threads (default = CPU count)
- **Task Parallel Library (TPL):** Logical tasks abstracted from OS threads
- **async/await:** Syntax sugar over state machines

### Task.Run and ThreadPool

```csharp
Task.Run(async () => { ... })
```

- Queues work to ThreadPool
- ThreadPool uses **work-stealing deques** for load balancing
- Threads are reused across many tasks
- **No per-task stack:** Async methods suspend/resume on the same thread

### Async/Await Implementation

- Compiler generates a **state machine** similar to Rust
- `await` doesn't create threads—it signs up a continuation
- Execution continues on the **current SynchronizationContext** (thread pool, UI thread, etc.)

### Memory Overhead

- **ThreadPool threads:** ~1 MB each (default min = CPU count)
- **Async tasks:** Minimal overhead
- **Common issue:** If many sync-over-async calls, threads block, ThreadPool grows

### Key Insight for VM Design

C# shows that **decoupling logical tasks from OS threads** is essential. The ThreadPool is a *resource*, and tasks are *logical units of work*. This is very close to what Atlas needs.

---

## 5. Python: Asyncio and Gevent

### Architecture Overview

Python has **two distinct concurrency models**, both limited by the GIL (Global Interpreter Lock):

#### Asyncio (Standard Library)
- **Event loop:** Single-threaded reactor pattern
- **Coroutines:** Async/await syntax, stackless (cooperative)
- **No true parallelism:** Only one Python code path executes at a time (single thread)
- **I/O multiplexing:** `select`/`epoll`/`kqueue` for I/O readiness

#### Gevent (Third-Party)
- **Greenlets:** Lightweight, stackful coroutines
- **Monkey-patching:** Replaces blocking I/O calls with non-blocking ones
- **Implicit scheduling:** Greenlets yield transparently on I/O
- **No true parallelism:** Still single-threaded, GIL prevents parallel execution

### Key Difference: Asyncio vs Gevent

| Feature | Asyncio | Gevent |
|---------|---------|--------|
| Stack model | Stackless (state machine) | Stackful (greenlet) |
| Scheduling | Explicit (await) | Implicit (I/O yield) |
| Existing code | Requires rewrite | Can wrap with minimal changes |
| Performance | Good for I/O-heavy | Good for existing code |

### Memory Overhead

- **Asyncio coroutine:** ~100-200 bytes
- **Gevent greenlet:** ~500 bytes - 1 KB
- **Scalability:** Asyncio: 10,000s tasks; Gevent: 100,000s greenlets

### Key Insight for VM Design

**Greenlets are stackful coroutines.** They're implemented via:
1. Saving/restoring execution context (stack pointer, registers)
2. A scheduler that decides which greenlet runs next
3. Yield points (explicit in asyncio, implicit in gevent)

Python's approach shows that a **non-thread-safe runtime can support massive concurrency** as long as you accept:
- No true OS-level parallelism
- Explicit or implicit yield points
- Single-threaded execution

---

## 6. Lua: Coroutines

### Architecture Overview

Lua coroutines are **fully contained in the interpreter**, with:
- **Asymmetric coroutines:** One coroutine can suspend, control returns to caller
- **Stackful:** Each coroutine has its own stack
- **Cooperative:** No preemption, explicit yield points
- **No scheduler:** Scheduling is user-defined

### Usage Pattern

```lua
co = coroutine.create(function() ... end)
coroutine.resume(co)  -- Run until yield
coroutine.yield()     -- Suspend from within
```

### Memory Efficiency

- **Per-coroutine:** Stack (grows on demand)
- **Concurrency:** Thousands of coroutines on modest hardware
- **Limitation:** No true parallelism; single-threaded

### Modern Extensions

**Lua-eco:** A variant with a **built-in event loop** for automatic coroutine scheduling (similar to greenlets/gevent).

### Key Insight for VM Design

Lua proves that **coroutines don't require a scheduler at the VM level**. The VM just needs:
1. Multiple stack frames (one per coroutine)
2. A resume/yield mechanism
3. Ability to save/restore VM state

---

## 7. Java: ThreadPool and Virtual Threads

### Architecture Overview

Java evolved from pure OS threads to **Virtual Threads** (Java 21+):

#### Traditional ThreadPool (Java 8+)
- **ExecutorService:** Abstraction over a pool of OS threads
- **Work-stealing:** `ForkJoinPool` uses work-stealing deques
- **Memory cost:** ~1 MB per thread

#### Virtual Threads (Java 21+)
- **M:N model:** Similar to goroutines
- **Lightweight:** ~100 bytes per virtual thread
- **Automatic scheduling:** JVM schedules onto a few carrier threads
- **Blocking I/O:** Virtual threads can call blocking I/O; JVM will transparently unmount them

### Virtual Thread Semantics

```java
try (ExecutorService executor = Executors.newVirtualThreadPerTaskExecutor()) {
    executor.submit(() -> { ... });  // New virtual thread per task
}
```

- Each task gets a **new virtual thread** (not pooled)
- JVM mounts virtual threads onto carrier threads
- If a virtual thread blocks → unmounts, allows carrier thread to run others
- Transparent to user code

### Key Insight for VM Design

Java's Virtual Threads show that **you can scale to millions of concurrent tasks** while still supporting blocking I/O operations. This is achieved through:
1. **Continuation-passing:** JVM can suspend a virtual thread (save stack) and resume later on a different carrier thread
2. **Transparent I/O integration:** Blocking calls don't block the carrier thread
3. **M:N scheduling:** Load-balanced across a small number of carrier threads

---

## 8. Comparative Analysis: Which Model for Which?

### Isolation vs Sharing Trade-off

| Model | Isolation | Sharing | Parallelism | Complexity |
|-------|-----------|---------|-------------|-----------|
| **Erlang** | Complete | None (messages) | True (per-core) | Very high |
| **Go** | Per-scheduler | Explicit (sync) | True (work-stealing) | High |
| **Rust/Tokio** | Per-task (Send) | Explicit (Arc) | True (work-stealing) | High |
| **Java Virtual Threads** | Per-thread | Explicit (volatile) | True (M:N) | Medium |
| **C# Tasks** | Per-task | Explicit (lock) | Per-ThreadPool | Medium |
| **Python Asyncio** | Per-coroutine | Single-thread | False (GIL) | Low |
| **Gevent** | Per-greenlet | Single-thread | False (GIL) | Low |
| **Lua Coroutines** | Per-coroutine | Single-thread | False | Very low |

### Memory Efficiency (per-task/task overhead)

```
Lua coroutine:          ~200 bytes (just stack + state)
Python asyncio:         ~100-200 bytes
Erlang process:         ~2 KB minimum
Gevent greenlet:        ~500 bytes - 1 KB
Java virtual thread:    ~100 bytes (modern estimate)
Go goroutine:           ~2 KB (growing on demand)
Rust Tokio future:      ~64 bytes + state machine
```

### Best For Bytecode VMs

**Ranking** (most to least suitable):

1. **Greenlets / Fibers (Lua/Gevent model)** — Stackful, simple scheduler, non-blocking user code
2. **Lua Coroutines (no scheduler)** — Even simpler, user manages scheduling
3. **Single-threaded Event Loop (Python Asyncio)** — Simple, no concurrency overhead
4. **Work-stealing on multiple threads (Go/Tokio)** — Complex isolation required, but true parallelism

---

## 9. The Core Problem: Non-Thread-Safe Runtime + Parallel User Code

### Problem Statement

Atlas has:
- **Value type:** `Send + Sync` (safe to cross thread boundaries)
- **VM type:** `!Send + !Sync` (not safe to cross thread boundaries)
- **Goal:** Run user functions in parallel while using a single VM

### Solutions Used in Practice

#### Option A: Single-Threaded Event Loop (Asyncio Model)
```
┌─────────────────────────────────┐
│ Main Thread + Event Loop        │
│  ┌──────────────────────────────┤
│  │ VM (single instance)         │
│  │  - Executes bytecode         │
│  │  - Runs one task at a time   │
│  └──────────────────────────────┤
│  ┌──────────────────────────────┤
│  │ Pending Tasks (queue)        │
│  │  - Awaiting I/O completion   │
│  │  - Ready to resume           │
│  └──────────────────────────────┤
└─────────────────────────────────┘
```

**Pros:**
- No synchronization overhead
- Simple semantics (sequential execution)
- Easy to debug

**Cons:**
- No true parallelism (one core only)
- Blocking user code stalls entire runtime
- Can't leverage multi-core hardware

#### Option B: Isolated VM Per Task (Erlang Model)
```
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│ Thread 1     │  │ Thread 2      │  │ Thread N     │
│ VM instance  │  │ VM instance   │  │ VM instance  │
│ Task 1       │  │ Task 2        │  │ Task N       │
└──────────────┘  └──────────────┘  └──────────────┘
     ▲                  ▲                   ▲
     └──────────────────┴───────────────────┘
           Message Passing Only
```

**Pros:**
- True parallelism (multi-core)
- No synchronization (complete isolation)
- Erlang-level reliability

**Cons:**
- High memory overhead (VM per task)
- Data must be copied/serialized
- Complex to implement

#### Option C: Shared VM with Locks (Thread-Safe Wrapper)
```
┌──────────────────────────────────────┐
│ Thread Pool + Sync Lock              │
│  ┌────────────────────────────────────┤
│  │ VM (wrapped in Mutex)              │
│  │  - Only one task runs at a time    │
│  │  - Mutex protects access           │
│  └────────────────────────────────────┤
│  ┌────────────────────────────────────┤
│  │ Task Queue + Work-Stealing         │
│  └────────────────────────────────────┤
└──────────────────────────────────────┘
```

**Pros:**
- Supports multiple threads
- Single VM instance
- Uses work-stealing for efficiency

**Cons:**
- High lock contention
- Serialized user code (no parallelism)
- Defeats the purpose of threads

#### Option D: Greenlets / Fibers (Tokio LocalSet Model)
```
┌────────────────────────────────────────┐
│ Main Thread                            │
│  ┌──────────────────────────────────────┤
│  │ Event Loop + Task Scheduler          │
│  │  ┌──────────────────────────────────┤
│  │  │ VM (single instance)             │
│  │  │  - Runs one task at a time       │
│  │  │  - Scheduler switches between    │
│  │  │    tasks on yield points         │
│  │  └──────────────────────────────────┤
│  │  ┌──────────────────────────────────┤
│  │  │ External I/O (multi-threaded)    │
│  │  │  - spawn_blocking for real work  │
│  │  │  - Executor handles OS threads   │
│  │  └──────────────────────────────────┤
│  └──────────────────────────────────────┤
└────────────────────────────────────────┘
```

**Pros:**
- Single VM (no duplication)
- No locks (cooperative scheduling)
- Can delegate to OS threads for I/O
- Memory-efficient (greenlet overhead ~0.5 KB)

**Cons:**
- User code must cooperatively yield (await points)
- No true CPU parallelism (single core for bytecode)
- Needs explicit "blocking" operation for CPU-bound work

#### Option E: Hybrid Model (Tokio Default)
```
┌─────────────────────────────────────────────┐
│ Worker Thread Pool (Tokio Runtime)          │
│  ┌──────────────────────────────────────────┤
│  │ T1        T2        T3  ...   TN         │
│  │ ┌────┐   ┌────┐   ┌────┐              │
│  │ │LS │   │LS │   │LS │              │ (LocalSet per thread)
│  │ └────┘   └────┘   └────┘              │
│  │  Each LocalSet has:                    │
│  │   - VM instance (thread-local)         │
│  │   - Task queue (single-threaded)       │
│  │   - Event loop                         │
│  └──────────────────────────────────────────┤
│                                             │
│  Work-stealing between threads              │
│  (only steals at task granularity)          │
└─────────────────────────────────────────────┘
```

**Pros:**
- True parallelism (multiple cores)
- No locks on fast path (work-stealing)
- Scales linearly with cores
- Cooperative scheduling within each LocalSet

**Cons:**
- Complex architecture
- Per-thread VM replication
- Requires careful task boundaries

---

## 10. Atlas Recommendations

### Current State

Atlas currently uses:
- **Tokio runtime** (multi-threaded)
- **spawn_task()** function that creates OS threads
- **LocalSet** for !Send futures
- **No proper task spawning from user code** (H-286 partial, not integrated)

From `async_runtime/task.rs` line 208:
```rust
std::thread::spawn(move || {
    crate::async_runtime::block_on(async move {
        tokio::task::spawn_local(async move { ... })
    })
});
```

**Problem:** This spawns an OS thread per task, which is wasteful and doesn't match the language design.

### Recommended Architecture

**Adopt the Hybrid Model (Option E) with the following design:**

#### Phase 1: Core Infrastructure (2-3 weeks)

1. **Create a `TaskExecutor` struct:**
   ```rust
   pub struct TaskExecutor {
       runtime: &'static tokio::runtime::Runtime,
       // Local VM state for this task execution context
       vm_context: Arc<Mutex<VMContext>>,
   }
   ```

2. **Implement per-thread VM isolation:**
   - Each Tokio worker thread gets a thread-local `VMContext`
   - `VMContext` holds a `VM` instance and execution stack
   - Users see seamless task execution, implementation hidden

3. **Create a work queue for local tasks:**
   - Tasks submitted to the same executor run on the same OS thread
   - This satisfies the `!Send` requirement for the VM

#### Phase 2: User-Facing API (2-3 weeks)

1. **Implement `task.spawn(fn)`:**
   ```atlas
   let task: Task<number> = task.spawn(async || {
       // This code runs in the background
       // Must be `async` (mark explicit I/O boundaries)
       expensive_computation()
   });

   let result: number = task.await;  // Wait for completion
   ```

2. **Add cancellation:**
   ```atlas
   task.cancel();
   if task.is_cancelled() { ... }
   ```

3. **Support task groups:**
   ```atlas
   let tasks = [task.spawn(...), task.spawn(...), ...];
   let results = task.join_all(tasks);
   ```

#### Phase 3: I/O Integration (3-4 weeks)

1. **Implement blocking-safe I/O:**
   ```atlas
   let result = await task.spawn_blocking(|| {
       // CPU-bound work or blocking I/O
       // Runs on separate thread pool, doesn't block the main executor
       expensive_disk_read()
   });
   ```

2. **Connect to stdlib I/O operations:**
   - `fs.read()`, `fs.write()` → return futures
   - `http.get()` → return futures
   - `process.exec()` → return futures

#### Phase 4: Testing and Validation (2 weeks)

1. **Parity tests:** Interpreter vs VM must produce identical output
2. **Concurrency stress tests:** 10,000+ concurrent tasks
3. **Memory profiling:** Verify overhead per task
4. **Fairness tests:** Verify tasks don't starve each other

### Implementation Details

#### Memory Model

Atlas values are `Send + Sync`, so they can be shared across threads via `Arc`:

```rust
// User writes:
let shared = Arc::new([1, 2, 3]);
let task = task.spawn(|| {
    // shared is Move-captured, Arc is cloned
    shared[0] + 1
});
```

#### Scheduling Strategy

Use **Tokio's built-in work-stealing scheduler** with `LocalSet` per thread:

```rust
#[tokio::main(flavor = "multi_thread", worker_threads = num_cpus::get())]
async fn main() {
    // Each worker thread gets a LocalSet
    // Tasks are spawned into the current thread's LocalSet
    // Work-stealing happens at the Tokio level
}
```

#### Task Representation in Bytecode

Add new opcode(s):

```
SpawnTask = 0xA0 [u16 fn_idx, u8 arg_count]
  → Spawns a new task, returns TaskHandle
  → Must verify fn is async-annotated (no blocking)

AwaitTask = 0xA1
  → Suspend current task until child task completes
  → Pops TaskHandle from stack, pushes result

SpawnBlocking = 0xA2 [u16 fn_idx, u8 arg_count]
  → Spawn work to thread pool (for blocking operations)
  → Returns Future that resolves when done
```

#### Language Syntax (Future)

```atlas
// async-annotated function (prevents blocking)
async fn expensive_work() -> number {
    let mut sum = 0;
    for i in range(0, 1_000_000) {
        sum += i;
    }
    return sum;
}

// Spawning
let task: Task<number> = task.spawn(expensive_work);

// Awaiting
let result: number = await task;

// Join all
let tasks = [task.spawn(f1), task.spawn(f2)];
let results = await task.join_all(tasks);  // [r1, r2]
```

### Comparison with Alternatives

| Aspect | Option D (Greenlet) | Option E (Hybrid) | Option A (Event Loop) |
|--------|-------------------|-------------------|----------------------|
| **Memory per task** | ~0.5 KB | ~64 bytes (Tokio) | ~0.5 KB |
| **Parallelism** | No (1 core) | Yes (N cores) | No (1 core) |
| **Lock contention** | None | None (work-stealing) | None |
| **Complexity** | Medium | High | Low |
| **Existing code** | Needs async/await | Needs async/await | Needs async/await |
| **User code blocking** | Breaks concurrency | Can use spawn_blocking | Blocks all tasks |
| **Recommended for Atlas** | If single-core only | Yes, current hardware | No (too limiting) |

### Risk Assessment

**High Risk:**
- **Parity maintenance:** Interpreter and VM must produce identical output when tasks run in different orders. Solution: Deterministic task scheduling for tests.
- **Deadlock potential:** Users can deadlock with `task.spawn(...).await` inside blocking I/O. Solution: Lint warnings in diagnostics.

**Medium Risk:**
- **Memory overhead:** Per-thread VM clones. Solution: Measure and optimize; use reference-counted bytecode.
- **Task migration:** Tokio work-stealing may move tasks between threads. Solution: LocalSet ensures tasks stay on one thread.

**Low Risk:**
- **API design:** Clear semantics with `async` annotations.
- **Testing:** Battle tests will catch issues early.

---

## 11. Key Takeaways

### What Works at Scale

1. **Go's approach** shows that M:N scheduling is practical and efficient.
2. **Erlang's approach** proves isolation eliminates synchronization overhead.
3. **Rust/Tokio** demonstrates that stackless futures with work-stealing can handle millions of concurrent tasks.
4. **Java Virtual Threads** show that transparent I/O blocking is achievable at M:N scale.
5. **Python Gevent** proves that greenlets work well for existing non-async code.

### Universal Principles

- **Isolation > Sharing:** Process isolation (Erlang) scales better than shared mutable state.
- **Work-stealing > Global Queue:** Distributed queues avoid bottlenecks.
- **Stackless > Stackful:** Futures/state machines use less memory than greenlets.
- **Explicit Yield > Implicit:** Async/await is easier to reason about than monkey-patching.

### For Atlas

- **Adopt hybrid model:** Multiple threads, each with a LocalSet
- **Use Tokio's scheduler:** Don't reinvent the wheel
- **Make async explicit:** Require `async` annotation for spawned tasks
- **Support both models:** Greenlet-style for single-core, thread-stealing for multi-core
- **Plan for evolution:** Start simple (greenlets), add parallelism later (work-stealing)

---

## 12. References and Sources

### Go Goroutines and Scheduler
- [Goroutines, OS Threads, and the Go Scheduler — DEV Community](https://dev.to/arundevs/goroutines-os-threads-and-the-go-scheduler-a-deep-dive-that-actually-makes-sense-1f9f)
- [Understanding Go's Goroutine Scheduler — Medium](https://medium.com/@aditimishra_541/understanding-gos-goroutine-scheduler-f8be8d962b45)
- [Scheduling In Go: Part II - Go Scheduler — Ardan Labs](https://www.ardanlabs.com/blog/2018/08/scheduling-in-go-part2.html)

### Rust Tokio and Async/Await
- [Spawning — Tokio Documentation](https://tokio.rs/tokio/tutorial/spawning)
- [tokio::task::spawn — Rust Docs](https://docs.rs/tokio/latest/tokio/task/fn.spawn.html)
- [Future is not Send — Rust Forum](https://users.rust-lang.org/t/future-is-not-send-inside-tokio-spawn/64898)

### Erlang BEAM VM
- [The BEAM Book: Understanding the Erlang Runtime System](https://blog.stenmans.org/theBeamBook/)
- [Deep Diving Into the Erlang Scheduler — AppSignal Blog](https://blog.appsignal.com/2024/04/23/deep-diving-into-the-erlang-scheduler.html)
- [Elixir and The Beam: How Concurrency Really Works — Medium](https://medium.com/flatiron-labs/elixir-and-the-beam-how-concurrency-really-works-3cc151cddd61)

### C# Tasks and ThreadPool
- [Overview of C# Async Programming — Medium](https://medium.com/devtechblogs/overview-of-c-async-programming-with-thread-pools-and-task-parallel-library-7b18c9fc192d)
- [Task.Run Method — Microsoft Learn](https://learn.microsoft.com/en-us/dotnet/api/system.threading.tasks.task.run?view=net-8.0)
- [Task Asynchronous Programming (TAP) Model — Microsoft Learn](https://learn.microsoft.com/en-us/dotnet/csharp/asynchronous-programming/task-asynchronous-programming-model)

### Python Asyncio and Gevent
- [asyncio vs gevent — Piccolo Blog](https://piccolo-orm.com/blog/asyncio-vs-gevent/)
- [Python concurrency: gevent had it right — Harshal Sheth](https://harshal.sheth.io/2025/09/12/python-async.html)
- [greenlet documentation — Python Package Index](https://greenlet.readthedocs.io/en/stable/)

### Lua Coroutines
- [Coroutines in Lua — PUC-Rio](https://www.inf.puc-rio.br/~roberto/docs/corosblp.pdf)
- [lightweight concurrency in lua — wingolog](https://wingolog.org/archives/2018/05/16/lightweight-concurrency-in-lua)
- [Lua-eco: Built-in event loop — GitHub](https://github.com/zhaojh329/lua-eco)

### Java Virtual Threads
- [Virtual Threads in Java — Medium](https://medium.com/hprog99/virtual-threads-in-java-unlocking-high-throughput-concurrency-55606100b6be)
- [Virtual Threads — Oracle Documentation](https://docs.oracle.com/en/java/javase/21/core/virtual-threads.html)
- [Threads, ThreadPools and Executors — SoftwareMill](https://softwaremill.com/threadpools-executors-and-java/)

### Green Threads and Fibers
- [Green thread — Wikipedia](https://en.wikipedia.org/wiki/Green_thread)
- [Introduction to fibers in C++ — These are the wrong sort of bees](https://www.romange.com/2018/12/15/introduction-to-fibers-in-c-/)
- [Fibers, green threads, stack switching, cooperative multitasking — Hacker News](https://news.ycombinator.com/item?id=37632346)

### Work-Stealing Schedulers
- [Work Stealing — Wikipedia](https://en.wikipedia.org/wiki/Work_stealing)
- [Scheduling Multithreaded Computations by Work Stealing — Cornell CS](https://www.cs.cornell.edu/courses/cs612/2006sp/papers/blumofe94.pdf)
- [Building a custom thread pool: work-stealing queue — Joe Duffy](https://joeduffyblog.com/2008/08/11/building-a-custom-thread-pool-series-part-2-a-work-stealing-queue/)

### Bytecode VM Concurrency
- [Parallelizing a bytecode interpreter — DEV Community](https://dev.to/lexplt/parallelizing-a-bytecode-interpreter-58m8)
- [Minimising virtual machine support for concurrency — arXiv](https://arxiv.org/pdf/1312.2707)

### Isolation and Threading in VMs
- [Parallelizing a bytecode interpreter — DEV Community](https://dev.to/lexplt/parallelizing-a-bytecode-interpreter-58m8)
- [The Design & Implementation of the CPython Virtual Machine — Coding Confessions](https://blog.codingconfessions.com/p/cpython-vm-internals)

---

## Appendix: Atlas Current Code References

### async_runtime/mod.rs
- **D-030:** `Value` is `Send`, enforced with compile-time assertion
- **LocalSet:** Thread-local for `!Send` futures (VM state)
- **spawn_local():** Already implemented, but not used by task API

### async_runtime/task.rs
- **Current design:** `spawn_task()` creates OS threads (inefficient)
- **Line 208:** `std::thread::spawn()` + `block_on()` + `spawn_local()`
- **Issue:** This defeats the purpose of Tokio's work-stealing

### runtime.rs
- **VM state:** Held in `RefCell<Option<VM>>` (interior mutability, single-threaded)
- **Thread-safety:** VM is `!Send + !Sync` by design

### Value type
- **D-030:** `Value` is `Send + Sync`
- **Collections:** CoW via `Arc<Vec<Value>>`, safe for sharing

### Recommendation Summary

**Replace the current `spawn_task()` implementation with a work-stealing executor that:**
1. Uses Tokio's runtime directly (don't spawn new threads)
2. Spawns tasks into the current thread's `LocalSet`
3. Lets Tokio handle work-stealing and scheduling
4. Returns a `TaskHandle` that user code can await

This aligns with Rust/Tokio's proven model and scales to thousands of concurrent tasks with minimal overhead.
