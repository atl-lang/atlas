# Phase 11b: Async I/O Operations

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Phase-11a complete (Future type), File I/O, HTTP modules exist.

**Verification:**
```bash
cargo test async_future_tests
ls crates/atlas-runtime/src/stdlib/io.rs
ls crates/atlas-runtime/src/stdlib/http.rs
```

**What's needed:**
- Future type from phase-11a
- File I/O from phase-05
- HTTP from phase-10a/10b
- Tokio runtime initialized

**If missing:** Complete phase-11a first

---

## Objective
Implement async versions of file and network I/O operations returning Futures - enabling non-blocking concurrent operations for responsive high-performance applications.

## Files
**Create:** `crates/atlas-runtime/src/stdlib/async_io.rs` (~600 lines)
**Update:** `crates/atlas-runtime/src/stdlib/io.rs` (~100 lines async wrappers)
**Update:** `crates/atlas-runtime/src/stdlib/http.rs` (~100 lines async wrappers)
**Create:** `crates/atlas-runtime/tests/async_io_tests.rs` (~500 lines)

## Dependencies
- Future type from phase-11a
- tokio::fs for async file I/O
- tokio integration with reqwest (already async)
- Stream handling for large files
- Permission checks before async ops

## Implementation

### Async File Reading
readFileAsync returns Future of string. Uses tokio::fs::read_to_string. Permission check before operation. Error handling with async Result. UTF-8 validation. Large file handling. Cancel safety.

### Async File Writing
writeFileAsync returns Future of null. Uses tokio::fs::write. Creates parent directories if needed. Atomic write option. Permission check before operation. Error propagation through future. Flush and sync options.

### Async File Appending
appendFileAsync returns Future of null. Uses tokio::fs::OpenOptions with append. Permission checks. Error handling. Efficient for log files. Line buffering option.

### Async File Streams
File streaming for large files. Async read line-by-line. Async write streaming. Backpressure handling. Memory-efficient processing. Stream combinators.

### Async HTTP Requests
httpGetAsync returns Future of HttpResponse. httpPostAsync, httpPutAsync, httpDeleteAsync. Reuses HttpRequest configuration. Non-blocking network I/O. Connection pooling. Timeout handling.

### Concurrent Requests
futureAll for parallel HTTP requests. Efficient concurrent file operations. Connection reuse. Resource limits. Error handling per request. Ordered results.

### Stream Response Bodies
Stream large HTTP responses. Avoid loading entire body in memory. Line-by-line processing. Chunk processing. Backpressure. Cancel support.

### Error Handling
Async I/O errors wrapped in Future. File not found (AT0015). Permission denied (AT0016). Network errors (AT0120-AT0124). Timeout errors (AT0170). Clear error messages.

## Tests (TDD - Use rstest)

**Async file reading (8):**
1. Read small file async
2. Read large file async
3. Read non-existent file (error)
4. Read with permission denial (error)
5. Multiple concurrent reads
6. Read UTF-8 file
7. Read empty file
8. Cancel read operation

**Async file writing (8):**
1. Write small file async
2. Write large file async
3. Overwrite existing file
4. Write with permission denial (error)
5. Multiple concurrent writes
6. Atomic write option
7. Write to non-existent directory (creates)
8. Write empty string

**Async file appending (5):**
1. Append to existing file
2. Append creates new file
3. Multiple concurrent appends
4. Append with permission denial (error)
5. Append empty string

**Async HTTP operations (10):**
1. GET request async
2. POST request async
3. PUT request async
4. DELETE request async
5. Multiple concurrent requests
6. Request timeout
7. Network error handling
8. Large response handling
9. Concurrent requests to different hosts
10. Request with custom headers async

**Stream operations (5):**
1. Stream large file read
2. Stream large HTTP response
3. Line-by-line file processing
4. Chunk processing
5. Cancel stream mid-operation

**Concurrent operations (8):**
1. Parallel file reads with futureAll
2. Parallel HTTP requests with futureAll
3. Mixed file and network operations
4. futureRace for first completed operation
5. Error in one of parallel operations
6. Many concurrent operations (10+)
7. Resource limit handling
8. Ordered results from futureAll

**Error handling (6):**
1. File not found async error
2. Permission denied async error
3. Network timeout async error
4. HTTP error status handling
5. Error propagation through chain
6. Retry on transient error

**Minimum test count:** 50 tests

## Integration Points
- Uses: Future type from phase-11a
- Uses: File I/O functions from phase-05
- Uses: HTTP functions from phase-10a/10b
- Uses: tokio runtime from phase-11a
- Enables: High-performance async I/O patterns
- Foundation: Async-first applications

## Acceptance
- ✅ Async file operations work correctly
- ✅ Async HTTP operations work correctly
- ✅ Concurrent operations efficient
- ✅ Stream operations functional
- ✅ Error handling comprehensive
- ✅ Permission checks enforced
- ✅ 50+ tests pass
- ✅ No clippy warnings
- ✅ cargo test passes
- ✅ Performance benefits measurable
