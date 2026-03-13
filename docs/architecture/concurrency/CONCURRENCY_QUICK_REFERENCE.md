# Lightweight Concurrency: Quick Reference & Decision Matrix

**For rapid lookups and architecture decisions**

---

## One-Page Comparison

### All Five Major Models

| Dimension | Single Event Loop | Greenlets | M:N Scheduler | Isolated VMs | Hybrid |
|-----------|------------------|-----------|---------------|--------------|---------|
| **Runtime** | Python asyncio | Gevent, Lua | Go, Java VThreads | Erlang BEAM | Rust Tokio, C# TPL |
| **Stacks** | Stackless | Stackful | Stackless | Stackful | Stackless |
| **Cores Used** | 1 | 1 | N | N | N |
| **Memory/Task** | ~100 B | ~500 B | ~100 B | ~2 KB | ~100 B |
| **Scheduling** | Cooperative | Cooperative | Preemptive | Preemptive | Work-stealing |
| **Sync Primitive** | Event loop | Greenlet yield | Mutex, channels | Messages | Arc + Mutex |
| **Max Concurrent** | 10,000s | 100,000s | Millions | Millions | Millions |
| **Parallelism** | No | No | Yes | Yes | Yes |
| **Complexity** | Low | Low-Med | High | Very High | High |
| **Best for** | I/O-bound, simple | Existing sync code | High throughput | Fault tolerance | General purpose |
| **Blocking risk** | High ⚠️ | High ⚠️ | Low ✓ | Low ✓ | Low ✓ |

---

## Memory Footprint (per concurrent task)

```
Lua coroutine     ████████░░░░░░░░░░░░  ~200 B
Python asyncio    ████████░░░░░░░░░░░░  ~100-200 B
Java VThread      ████████░░░░░░░░░░░░  ~100 B
Tokio future      ████░░░░░░░░░░░░░░░░  ~64 B (+ state machine)
Gevent greenlet   ██████████░░░░░░░░░░  ~500 B - 1 KB
Go goroutine      ████████████████░░░░  ~2 KB (minimum)
Erlang process    ████████████████████  ~2 KB (minimum)
OS thread         ████████████████████████████████ ~1-8 MB
```

---

## Decision Tree: Which Model for Your VM?

```
Does your VM need true multi-core parallelism?
│
├─ NO (single-core only)
│  │
│  ├─ Want simplicity? → Single Event Loop (Python asyncio)
│  │
│  └─ Want greenlets? → Cooperative (Lua/Gevent)
│
└─ YES (multi-core parallelism)
   │
   ├─ Can you isolate VM instances? → Isolated VMs (Erlang model)
   │  ✓ Pros: No synchronization, Erlang-grade reliability
   │  ✗ Cons: 2 KB+ per task, data copying
   │
   └─ Must share single VM instance?
      │
      ├─ Can you make VM thread-safe? → M:N Scheduler with Locks
      │  ✓ Pros: Single instance, parallelism
      │  ✗ Cons: Lock contention, complexity
      │
      └─ Want lock-free scheduling? → Hybrid Model (Tokio)
         ✓ Pros: Work-stealing, low contention, proven at scale
         ✗ Cons: More complex architecture
```

---

## Problem: Non-Thread-Safe VM with Parallel Tasks

### The Five Solutions

#### 1. Single Event Loop (No Parallelism)
```
Event loop runs one task at a time
┌─────────────────────┐
│ VM (shared)         │
│ Task queue          │
│ I/O event handlers  │
└─────────────────────┘
```
- **When to use:** Single-core only, I/O-heavy workloads
- **Atlas fit:** Not suitable (want true parallelism)

#### 2. Greenlets (Single-Threaded, Stackful)
```
Main thread
├─ Greenlet 1 (suspended at await)
├─ Greenlet 2 (running)
├─ Greenlet 3 (suspended at await)
└─ Event loop (switches between them)
```
- **When to use:** Modest concurrency (10,000s), existing sync code
- **Atlas fit:** Good for single-core, too limiting for multi-core

#### 3. M:N with Locks (Shared VM, Synchronized)
```
Thread 1 ─┐
Thread 2 ─┼─ Mutex ─┬─ VM
Thread 3 ─┤        └─ Task queue
...      ─┘
```
- **When to use:** Only if you have no choice
- **Atlas fit:** Bad (serializes user code, defeats threading)

#### 4. Isolated VMs (One VM per Task)
```
Thread 1 ─ VM 1 ─ Task 1
Thread 2 ─ VM 2 ─ Task 2
Thread 3 ─ VM 3 ─ Task 3
...

Message passing only
```
- **When to use:** Fault tolerance, no shared mutable state
- **Atlas fit:** Overkill (2 KB+ per task overhead)

#### 5. Hybrid with Work-Stealing (Tokio Model) ✓ RECOMMENDED
```
Thread 1          Thread 2          Thread 3
┌──────────┐     ┌──────────┐     ┌──────────┐
│ LocalSet │     │ LocalSet │     │ LocalSet │
│ Task Q1  │     │ Task Q2  │     │ Task Q3  │
│ VM inst. │     │ VM inst. │     │ VM inst. │
└──────────┘     └──────────┘     └──────────┘
    ▲                 ▲                 ▲
    └─────────────────┼─────────────────┘
       Work-stealing scheduler
```
- **When to use:** Multi-core, lock-free efficiency, proven model
- **Atlas fit:** PERFECT (Go, Rust/Tokio, Java VThreads all use this)

---

## Comparative Feature Matrix

### Scheduling Strategy

| Aspect | Event Loop | Greenlet | M:N | Isolated | Hybrid |
|--------|------------|----------|-----|----------|---------|
| **Scheduler location** | Single-threaded | Single-threaded | Per-thread + global | Per-process | Per-thread + work-steal |
| **Preemption** | Explicit (await) | Explicit (yield) | Forced (timer) | Forced (timer) | Implicit (await) |
| **Context switch cost** | ~10 cycles | ~100 cycles | ~1,000 cycles | ~1,000 cycles | ~10 cycles (local) |
| **Lock contention** | None | None | None | None | Low (work-stealing) |
| **Task migration** | N/A | N/A | Yes (expensive) | Yes (copy) | Yes (cheap) |

### Scalability Profile

```
Throughput (tasks per second)

Isolated VMs:
  ████████████░░░░░░░░░░░░░░  1M tasks (but 2KB each = 2GB)

Hybrid M:N:
  ██████████████████░░░░░░  10M tasks efficiently

Event Loop:
  █████░░░░░░░░░░░░░░░░░░░  100K tasks, 1 core

Greenlets:
  ███████░░░░░░░░░░░░░░░░░  1M tasks (single core)
```

---

## Implementation Effort (Relative)

```
Single Event Loop:     ▓░░░░░░░░░░░░░░░░░░  ~2 weeks
Greenlets:             ▓▓▓▓▓░░░░░░░░░░░░░░  ~4 weeks
Hybrid (Tokio-based):  ▓▓▓▓▓▓▓▓▓░░░░░░░░░░  ~8-10 weeks
M:N with Locks:        ▓▓▓▓▓▓▓░░░░░░░░░░░░  ~6 weeks
Isolated VMs:          ▓▓▓▓▓▓▓▓▓▓░░░░░░░░░░  ~12 weeks
```

---

## Real-World Examples

### Event Loop
- Python asyncio, Node.js, Ruby Async
- Best for: Web servers, I/O multiplexing
- Weakness: Can't use multiple cores

### Greenlets
- Python Gevent, Lua
- Best for: Porting sync code to async
- Weakness: Monkey-patching, single-threaded

### M:N Scheduler
- Go (goroutines)
- Java (virtual threads)
- Best for: General-purpose concurrency
- Weakness: Preemption overhead, complexity

### Isolated VMs
- Erlang/Elixir (BEAM)
- Best for: Distributed systems, fault tolerance
- Weakness: Memory overhead, data copying

### Hybrid (Work-Stealing)
- Rust/Tokio, C# TPL, Java (ThreadPool)
- Best for: **High-performance systems, multi-core scaling**
- Strength: Proven, efficient, no lock contention on fast path

---

## Atlas Architecture Decision

### Current State (Suboptimal)

```rust
// async_runtime/task.rs, line 208
std::thread::spawn(move || {
    crate::async_runtime::block_on(async move {
        tokio::task::spawn_local(async move { ... })
    })
});
```

**Problems:**
1. Creates a new OS thread per task (very expensive)
2. Ignores Tokio's work-stealing scheduler
3. Doesn't scale beyond a few hundred tasks

### Recommended State (Hybrid with Work-Stealing)

**Phase 1: Stop spawning OS threads**
```rust
// Replace with: Use Tokio's spawn_local directly
tokio::task::spawn_local(async move {
    vm.execute_task(closure).await
})
```

**Phase 2: Implement per-thread VM isolation**
- Each Tokio worker thread gets a thread-local `VMContext`
- Tasks on the same thread share the same VM instance
- Work-stealing handles load balancing

**Phase 3: Expose user-facing API**
```atlas
let task = task.spawn(|| { ... });
let result = await task;
```

### Key Advantages

| Metric | Current | Recommended | Improvement |
|--------|---------|-------------|-------------|
| **Memory per task** | ~1 MB | ~100 B | 10,000x |
| **Max concurrent tasks** | ~100 | ~1,000,000 | 10,000x |
| **Scheduler efficiency** | Manual | Work-stealing | Automatic |
| **Lock contention** | Low (1 thread) | Very low | Better |
| **Multi-core scaling** | None | Linear | 4-16x on modern hardware |

---

## Language Design Consequences

### What Syntax to Support?

#### Option A: Async/Await (Go-style)
```atlas
async fn expensive() -> number { ... }

let task = task.spawn(expensive);
let result = await task;
```
- **Pros:** Familiar, explicit I/O boundaries
- **Cons:** Requires type annotations on functions

#### Option B: Greenlet-style (Lua/Gevent)
```atlas
let task = task.spawn(|| {
    // Runs in background, no await needed
    expensive_computation()
});

// Implicit: wait on access
let result = task.result();  // Blocks if not ready
```
- **Pros:** Simpler syntax, works with existing code
- **Cons:** Can deadlock, implicit blocking

#### Option C: Coroutine-style (Erlang messages)
```atlas
let task = process.spawn(() {
    send(parent, { type: "result", value: 42 })
});

receive {
    { type: "result", value: v } => { ... }
}
```
- **Pros:** Erlang-proven reliability
- **Cons:** Different paradigm, message-based

**Recommendation for Atlas:** Option A (async/await) + Option B (implicit spawn)

---

## Testing Checklist

When implementing your chosen model:

### Correctness
- [ ] Parity: Interpreter and VM produce identical output
- [ ] Determinism: Same input → same output every time
- [ ] Task isolation: One task's panic doesn't crash others
- [ ] Resource cleanup: Completed tasks are freed

### Performance
- [ ] Memory: Measure overhead per task
- [ ] Throughput: Task spawn/completion rate
- [ ] Latency: Task response time under load
- [ ] Multi-core scaling: Linear improvement with cores

### Edge Cases
- [ ] Nested spawns: task.spawn inside task.spawn
- [ ] Cancellation: task.cancel() works correctly
- [ ] Timeouts: task.timeout() doesn't leak resources
- [ ] Large task count: 1M+ concurrent tasks

### Concurrency Safety
- [ ] No data races (use Miri, ThreadSanitizer)
- [ ] No deadlocks (formal verification or testing)
- [ ] No use-after-free (ASAN)

---

## References (Quick Links)

**Go Goroutines:**
- [Go Scheduler Deep Dive](https://dev.to/arundevs/goroutines-os-threads-and-the-go-scheduler-a-deep-dive-that-actually-makes-sense-1f9f)

**Rust Tokio:**
- [Tokio Spawning Tutorial](https://tokio.rs/tokio/tutorial/spawning)

**Erlang BEAM:**
- [BEAM Book - Scheduling](https://blog.stenmans.org/theBeamBook/)

**Java Virtual Threads:**
- [Virtual Threads Documentation](https://docs.oracle.com/en/java/javase/21/core/virtual-threads.html)

**Python Asyncio:**
- [Python asyncio Docs](https://docs.python.org/3/library/asyncio.html)

**Work-Stealing:**
- [Work Stealing Scheduler Paper](https://www.cs.cornell.edu/courses/cs612/2006sp/papers/blumofe94.pdf)

---

## FAQ

### Q: Why not just make the VM thread-safe with locks?
A: Lock contention would serialize user code, defeating the purpose of multi-threading. Work-stealing avoids this by using thread-local queues.

### Q: Doesn't Tokio require all types to be Send?
A: Only for `tokio::spawn()` (multi-threaded). Use `tokio::task::spawn_local()` for !Send types (like VM state), which is what Atlas does.

### Q: How do you prevent deadlocks in hybrid model?
A: By enforcing await points in syntax (async annotation) and linting against nested synchronous blocking calls.

### Q: What about CPU-bound tasks in the event loop?
A: Use `task.spawn_blocking()` to delegate to a separate thread pool (exactly like Tokio does).

### Q: Can you migrate tasks between threads?
A: Yes, if you use stackless futures (state machines). No, if you use stackful coroutines. Hybrid model uses stackless, so task migration is cheap.

### Q: How many OS threads does Tokio create?
A: By default, one per CPU core. All tasks share these threads via work-stealing.

---

## Summary Table: The Recommendation

**For Atlas, adopt the Hybrid Model (Tokio-based):**

```
┌─────────────────────────────────────────────┐
│ Architecture: Tokio Runtime                 │
│ + Per-thread VM isolation (LocalSet)        │
│ + Work-stealing scheduler                   │
│ + Async/await syntax for user code          │
├─────────────────────────────────────────────┤
│ Expected Scaling:                           │
│ - 1M+ concurrent tasks                      │
│ - Linear improvement with cores             │
│ - <1 KB memory per task                     │
│ - <10 cycle scheduling overhead             │
├─────────────────────────────────────────────┤
│ Development:                                │
│ - 8-10 weeks full implementation           │
│ - Phase 1 (2w): Fix spawn_task() API       │
│ - Phase 2 (3w): Per-thread isolation       │
│ - Phase 3 (3w): I/O integration            │
│ - Phase 4 (2w): Testing & validation       │
└─────────────────────────────────────────────┘
```

This model is proven at scale (Go, Rust, Java) and fits Atlas's architecture.
