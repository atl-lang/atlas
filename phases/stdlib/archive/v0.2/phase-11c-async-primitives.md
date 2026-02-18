# Phase 11c: Async Primitives & Task Management

## 🚨 BLOCKERS - CHECK BEFORE STARTING

**⛔ BLOCKED BY FOUNDATION PHASES - DO NOT START**

**REQUIRED FOUNDATION (CRITICAL):**
- Foundation/Phase-18: Value Arc Refactor (Rc→Arc for thread safety)
- Foundation/Phase-19: Enable Parser Features (match/import)
- Foundation/Phase-20: For-In Loops (ergonomic iteration)

**REQUIRED STDLIB:**
- Phase-11a and 11b complete

**Why Blocked:**
Current Value enum uses `Rc<T>` which is NOT Send/Sync. Async tasks with
tokio::spawn REQUIRE Send trait. Phase-18 fixes this by refactoring to `Arc<T>`.

**Verification:**
```bash
# Foundation phases complete
ls phases/foundation/phase-18-value-arc-refactor.md
ls phases/foundation/phase-19-enable-parser-features.md
ls phases/foundation/phase-20-for-in-loops.md

# Async foundation ready
cargo test async_future_tests
cargo test async_io_tests

# Value is Send (after Phase-18)
! grep "Rc<String>" crates/atlas-runtime/src/value.rs
grep "Arc<String>" crates/atlas-runtime/src/value.rs
```

**What's needed:**
- Thread-safe Value enum (Arc instead of Rc)
- Future type from phase-11a
- Async I/O from phase-11b
- Tokio runtime infrastructure
- Channel primitives
- Match expressions enabled (for Result/Option handling)
- For-in loops enabled (for ergonomic iteration)

**If missing:** Complete Foundation phases 18, 19, 20 AND Stdlib phases 11a, 11b FIRST

---

## Objective
Implement task spawning, channels, timers, and other async primitives - completing the async I/O foundation with full concurrency support for building responsive applications.

## Files
**Create:** `crates/atlas-runtime/src/async_runtime/task.rs` (~400 lines)
**Create:** `crates/atlas-runtime/src/async_runtime/channel.rs` (~300 lines)
**Create:** `crates/atlas-runtime/src/async_runtime/primitives.rs` (~200 lines)
**Update:** `crates/atlas-runtime/src/value.rs` (~50 lines Task, Channel variants)
**Create:** `crates/atlas-runtime/tests/async_primitives_tests.rs` (~600 lines)
**Create:** `docs/async-io.md` (~800 lines comprehensive async guide)

## Dependencies
- Future and async runtime from phase-11a
- tokio::spawn, tokio::time, tokio::sync
- Task handles for cancellation
- Channel for message passing

## Implementation

### Task Spawning
spawn function launches async task. Returns TaskHandle. Task executes concurrently. Non-blocking task creation. Error isolation per task. Panic handling. Task name/ID for debugging.

### Task Management
TaskHandle allows await on completion. cancel method terminates task. Task status checking (pending/complete/cancelled). Join multiple tasks. Task local storage. Parent-child task relationships.

### Channels
channelNew creates bounded/unbounded channel. Returns [sender, receiver] array. send method adds message. receive method returns Future. close channel. Channel capacity. Backpressure handling. Multiple senders/receivers.

### Select Operation
channelSelect awaits first available message. Handles multiple channels. Returns [value, channelIndex]. Non-blocking select with timeout. Fair channel selection. Useful for event loops.

### Sleep and Timers
sleep function returns Future. Delays execution by milliseconds. Non-blocking delay. timer function for scheduled execution. Interval for repeated execution. Cancel timers. High-resolution timing.

### Async Mutex
asyncMutex creates async-aware mutex. lock method returns Future of guard. Non-blocking lock acquisition. Deadlock detection. Fair lock ordering. Guard releases on drop.

### Timeout Operations
timeout wraps Future with time limit. Returns TimeoutError if exceeded. Configurable duration. Cancel underlying operation. Useful for network operations. Nested timeout handling.

### Performance Monitoring
Task execution metrics. Event loop health monitoring. Queue depth tracking. Slow task detection. Memory usage per task. Scheduling latency measurement.

## Tests (TDD - Use rstest)

**Task spawning (10):**
1. Spawn simple task
2. Task returns value
3. Multiple concurrent tasks
4. Task error propagation
5. Cancel running task
6. Task status checking
7. Join multiple tasks with futureAll
8. Task panic handling
9. Named tasks
10. Nested task spawning

**Channels (12):**
1. Create bounded channel
2. Create unbounded channel
3. Send and receive message
4. Multiple messages in order
5. Channel full handling (bounded)
6. Channel closed error
7. Multiple senders
8. Multiple receivers
9. Channel with futures as messages
10. Channel capacity management
11. Send non-blocking
12. Receive with no messages (waits)

**Select operation (6):**
1. Select from two channels
2. Select first available message
3. Select with timeout
4. Select with all channels empty
5. Select with closed channel
6. Fair selection across channels

**Sleep and timers (8):**
1. Sleep for duration
2. Multiple concurrent sleeps
3. Sleep with cancellation
4. Timer scheduled execution
5. Interval repeated execution
6. Cancel timer
7. High-resolution sleep
8. Sleep in task

**Async mutex (6):**
1. Create mutex
2. Lock and unlock
3. Concurrent lock attempts
4. Lock across tasks
5. Mutex guard auto-release
6. Nested mutex locking

**Timeout operations (6):**
1. Operation completes before timeout
2. Operation exceeds timeout
3. Timeout error handling
4. Cancel on timeout
5. Nested timeouts
6. Timeout with async I/O

**Integration tests (10):**
1. Async web scraper (HTTP + tasks)
2. Concurrent file processing (I/O + channels)
3. Parallel API calls with aggregation
4. Producer-consumer pattern
5. Fan-out fan-in pattern
6. Request rate limiter
7. Timeout retry pattern
8. Mixed sync and async code
9. Complex future chains
10. Real-world async application

**Performance tests (5):**
1. Task spawn overhead
2. Channel throughput
3. Many concurrent tasks (1000+)
4. Event loop latency
5. Memory usage with futures

**Minimum test count:** 63 tests (comprehensive primitives coverage)

## Integration Points
- Uses: Future type from phase-11a
- Uses: Async I/O from phase-11b
- Uses: Tokio runtime infrastructure
- Completes: Full async I/O foundation
- Enables: High-concurrency applications
- Enables: Reactive programming patterns
- Foundation: Future async/await syntax

## Acceptance
- ✅ Task spawning and management works
- ✅ Channels functional for message passing
- ✅ Select operation works correctly
- ✅ Sleep and timers work
- ✅ Async mutex functional
- ✅ Timeout operations work
- ✅ Integration tests demonstrate patterns
- ✅ Performance acceptable
- ✅ 63+ tests pass
- ✅ Documentation complete with examples
- ✅ No clippy warnings
- ✅ cargo test passes
- ✅ Async I/O foundation complete
