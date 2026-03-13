# Atlas Concurrency Architecture — Index

**Status:** Production implementation complete (B44).
**Last updated:** 2026-03-13

This document describes the actual concurrency implementation in Atlas as of B44.
The earlier research documents (`CONCURRENCY_RESEARCH.md`, old `CONCURRENCY_QUICK_REFERENCE.md`)
were pre-implementation speculation. Read this index and `CONCURRENCY_QUICK_REFERENCE.md` for facts.

---

## What Is Implemented (B44 Complete)

The B44 block delivered end-to-end concurrent task execution:

- **B44-P02:** `VMContext` split from `VM` — per-thread execution state isolated
- **B44-P06:** Work-stealing scheduler (`crossbeam-deque`) + `WorkerPool`
- **B44-P07:** I/O integration, `spawn_blocking`, timeout via Tokio
- **B44-P08:** Concurrency + stress test suites

**H-361/B41-P02:** `task.spawn()` works end-to-end from Atlas programs (stdlib wired to `spawn_function_task`).
**H-386:** `AtlasFuture` implements `std::future::Future` + sleep/timeout are async-safe.

---

## Key Source Files

| Component | File |
|-----------|------|
| Async runtime entry | `crates/atlas-runtime/src/async_runtime/mod.rs` |
| AtlasFuture | `crates/atlas-runtime/src/async_runtime/future.rs` |
| Task spawning | `crates/atlas-runtime/src/async_runtime/task.rs` |
| Work-stealing scheduler | `crates/atlas-runtime/src/async_runtime/scheduler.rs` |
| Worker threads | `crates/atlas-runtime/src/async_runtime/worker.rs` |
| Async primitives | `crates/atlas-runtime/src/async_runtime/primitives.rs` |
| Channels | `crates/atlas-runtime/src/async_runtime/channel.rs` |
| VM context | `crates/atlas-runtime/src/vm/context.rs` |
| VM (shared resources) | `crates/atlas-runtime/src/vm/mod.rs` |

---

## Architecture in One Diagram

```
Atlas Program
    │
    │ task.spawn(fn, args)
    ▼
VM: SpawnTask opcode  ─────────────────────────────┐
    │                                               │
    │                                               ▼
    │                                   spawn_function_task()
    │                                         │
    │                                    WorkerTask closure
    │                                    (Send + 'static)
    │                                         │
    │                                    Scheduler::push(task)
    │                                         │
    │                                   ┌─────┴──────────┐
    │                               waker                 │
    │                                   ▼                 ▼
    │                            Worker thread A   Worker thread B
    │                            ┌────────────┐   ┌────────────┐
    │                            │ LocalSet   │   │ LocalSet   │
    │                            │ VM (clone) │   │ VM (clone) │
    │                            │ local deque│   │ local deque│
    │                            └────────────┘   └────────────┘
    │                               │   ↑ steal         │
    │                               └───┴───────────────┘
    │                                work-stealing
    ▼
Value::Future (handle)
    │
    │ await
    ▼
result Value
```

---

## The Three Runtimes

Atlas uses **three Tokio runtimes** with different roles:

### 1. Global Multi-Thread Runtime (`TOKIO_RUNTIME`)

```rust
static TOKIO_RUNTIME: OnceLock<Runtime>  // new_multi_thread, enable_all
```

Initialized by `init_runtime()`. Used for:
- `block_on()` — bridges sync VM code to async Tokio futures
- `spawn_blocking_task()` — CPU-bound blocking tasks on Tokio's blocking thread pool

### 2. Per-Worker `current_thread` Runtime

Each worker thread runs its own `tokio::runtime::Builder::new_current_thread()` runtime with a `LocalSet`. This allows `!Send` futures (including `RefCell<VM>`) to run cooperatively on the worker's OS thread.

### 3. Blocking Pool Bridge

`spawn_blocking_task` uses `runtime().spawn(async { tokio::task::spawn_blocking(...) })` — delegates into Tokio's dedicated blocking thread pool. Used for CPU-bound functions that would otherwise starve cooperative async tasks.

---

## Initialization Sequence

Call order at program startup (typically in `runtime.rs`):

```rust
init_runtime();             // Creates global multi-thread Tokio runtime
// ... compile and run main program once on the main VM ...
init_blocking_pool(&vm);    // Snapshots VM for blocking tasks
init_worker_pool(0, &vm);   // Creates WorkerPool (0 = hardware_concurrency workers)
```

`init_worker_pool` panics if called more than once. `0` workers → `thread::available_parallelism()`, fallback 4.

---

## Documents in This Directory

### CONCURRENCY_QUICK_REFERENCE.md
Concise reference for the implemented architecture: key types, call paths, error handling, and Atlas-level API. **Read this for implementation work.**

### CONCURRENCY_RESEARCH.md
Pre-implementation research comparing concurrency models across Go, Rust, Erlang, C#, Python, Lua, Java. **Read for historical context only.** The recommendations in this document are now implemented facts, not proposals.

---

## What Is NOT in This Document

- Full API reference for `task.spawn()`, channels, `await` → see `docs/stdlib/task.md` (Atlas user docs)
- VM internals beyond the concurrency boundary → see `docs/architecture/vm.md`
- Async language syntax → see `docs/language/async.md`
