# Phase 11a: Async Foundation - Future Type & Runtime

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Result types, basic stdlib infrastructure.

**Verification:**
```bash
cargo test -p atlas-runtime
ls crates/atlas-runtime/src/stdlib/mod.rs
```

**What's needed:**
- Result types from foundation/phase-09 ✅
- Value enum extensibility ✅
- Tokio async runtime

**If missing:** Foundation complete, ready to proceed

---

## Objective
Implement Future/Promise type and basic async runtime infrastructure - the foundation enabling non-blocking I/O operations in phases 11b/11c and future async/await syntax.

## Files
**Create:** `crates/atlas-runtime/src/async_runtime/mod.rs` (~300 lines)
**Create:** `crates/atlas-runtime/src/async_runtime/future.rs` (~400 lines)
**Update:** `crates/atlas-runtime/src/value.rs` (~100 lines Future variant)
**Update:** `crates/atlas-runtime/src/lib.rs` (expose async_runtime module)
**Update:** `Cargo.toml` (add tokio dependency)
**Create:** `crates/atlas-runtime/tests/async_future_tests.rs` (~400 lines)

## Dependencies
- tokio (async runtime)
- Result types for async errors
- Rc/Arc for Future sharing
- Value enum extension

## Implementation

### Future Value Type
Add Future variant to Value enum representing pending async computation. States: Pending (with Waker), Ready(Result<Value, Value>). Generic over success/error types via Value. Interior mutability via Rc<RefCell<FutureState>>. Clone support for sharing futures. Display and Debug implementations.

### Future State Machine
FutureState tracks computation state. Pending state stores waker for notification. Ready state stores final result. Transition from Pending to Ready is one-way. Thread-safe state updates. Poll method advances state machine. Wake mechanism triggers continuation.

### Future Constructor Functions
futureResolve creates immediately resolved future. futureReject creates immediately rejected future. futureNew creates pending future with executor function. Executor receives resolve/reject callbacks. Executor runs in async context. Error handling for executor panics.

### Future Chaining Methods
then method chains success handler. Returns new future with transformed value. Handler can return value or future. Automatic future flattening. catch method handles rejections. Returns new future with error recovery. Handler receives error value. Can recover or propagate error.

### Future Combinators
futureAll combines multiple futures. Returns array of all results. Rejects if any future rejects. Preserves result order. futureRace returns first completed future. Ignores remaining futures. Useful for timeouts. Handles both success and failure.

### Tokio Runtime Integration
Initialize tokio runtime at startup. Single-threaded or multi-threaded runtime. Runtime stored in global state or context. spawn_local for non-Send futures. block_on for sync/async bridge. Runtime lifecycle management.

### Error Handling
Async errors propagate through futures. RuntimeError with async-specific codes. Clear error messages for async failures. Stack trace preservation through async boundary. Timeout errors (AT0170). Cancellation errors (AT0171).

## Tests (TDD - Use rstest)

**Future type tests (10):**
1. Create resolved future
2. Create rejected future
3. Clone future preserves state
4. Display format for futures
5. Future as Value variant
6. Multiple future references
7. Future type_name is "future"
8. Equality not supported (appropriate error)
9. Future in array
10. Future in HashMap value

**Future state machine (5):**
1. Pending to Ready transition
2. Ready state is final
3. Poll pending future
4. Poll ready future
5. Waker notification

**Future chaining (8):**
1. then with value handler
2. then with future handler
3. then chaining multiple
4. catch with error handler
5. catch chaining
6. then + catch combination
7. Nested future flattening
8. Handler error propagation

**Future combinators (7):**
1. futureAll with all success
2. futureAll with one failure
3. futureAll empty array
4. futureRace with multiple futures
5. futureRace first succeeds
6. futureRace first fails
7. Combinator with resolved futures

**Runtime integration (5):**
1. Tokio runtime initializes
2. spawn_local executes future
3. block_on bridges sync/async
4. Runtime cleanup on shutdown
5. Multiple concurrent futures

**Error handling (5):**
1. Rejected future propagates error
2. Async error codes
3. Handler panic handling
4. Error through chain
5. Timeout error (basic)

**Minimum test count:** 40 tests (comprehensive foundation)

## Integration Points
- Updates: Value enum with Future variant
- Creates: async_runtime module infrastructure
- Uses: Result types for async errors
- Enables: phase-11b (async I/O operations)
- Enables: phase-11c (task management, primitives)
- Foundation: Future async/await syntax

## Acceptance
- ✅ Future value type works correctly
- ✅ Future state machine functional
- ✅ then/catch chaining works
- ✅ futureAll/futureRace combinators work
- ✅ Tokio runtime integrated
- ✅ Error handling complete
- ✅ 40+ tests pass
- ✅ No clippy warnings
- ✅ cargo test passes
- ✅ Both interpreter and VM support Future values
