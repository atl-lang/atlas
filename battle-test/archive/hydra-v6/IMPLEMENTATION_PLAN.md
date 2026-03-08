# Hydra to Atlas Port - Implementation Plan

**Based on:** Comprehensive audit of 13 domains with 37+ friction points
**Objective:** Complete production-ready port of Hydra to Atlas v0.3
**Total Effort:** 8-10 weeks (280-400 hours)
**Team Size:** 2-3 dedicated developers

---

## Phase 1: Validation & Clarification (1-2 weeks)

### Goal: Verify all stdlib claims and document exact APIs

#### Task 1.1: Verify HashMap
```
[ ] Test HashMap.new() vs hashMapNew() signature
[ ] Document key/value types and constraints
[ ] Test iteration patterns (keys, values, entries)
[ ] Verify concurrent access patterns
[ ] Document in stdlib guide
```
**Blocker for:** StateStore, Proxy, caching
**Effort:** 3-4 days
**Critical:** YES

#### Task 1.2: Test File I/O Module
```
[ ] Review file.md specification
[ ] Test File.read(path) implementation
[ ] Test File.write(path, content) implementation
[ ] Test File.append(path, content) if available
[ ] Test directory operations
[ ] Document error handling patterns
[ ] Test relative vs absolute paths
```
**Blocker for:** Config loading, persistence, logging to file
**Effort:** 2-3 days
**Critical:** YES

#### Task 1.3: Clarify Async/Concurrency
```
[ ] Review async.md and tokio integration
[ ] Test async/await syntax with examples
[ ] Test task spawning equivalent to goroutines
[ ] Test cancellation patterns (context equivalent)
[ ] Document real concurrency vs OS threads
[ ] Evaluate performance vs Go goroutines
```
**Blocker for:** Watcher, Proxy async forwarding, real concurrency
**Effort:** 3-5 days
**Critical:** YES

#### Task 1.4: Stabilize API Usage
```
[ ] Test both old and new function signatures
[ ] Determine which will be standard
[ ] Check deprecation timeline
[ ] Plan migration strategy
[ ] Document in guide
```
**Issues:** H-150, H-152
**Effort:** 2-3 days

#### Task 1.5: Investigate Type Inference
```
[ ] Test Ok(void) type inference in various contexts
[ ] Test match assignment type loss patterns
[ ] Test empty array [] type inference edge cases
[ ] Propose solutions or identify workarounds
[ ] Document in QUICK_REFERENCE.md
```
**Issues:** H-151
**Effort:** 3-5 days

#### Task 1.6: Clarify Ownership System
```
[ ] Investigate borrow/own keyword actual syntax
[ ] Test scenarios where these apply
[ ] Clarify when Copy vs move semantics apply
[ ] Update documentation
```
**Issues:** H-149
**Effort:** 2-3 days

### Phase 1 Deliverables
- HashMap API guide with code examples
- File I/O API reference with error cases
- Async patterns tutorial (if supported)
- API migration guide (old vs new)
- Type inference troubleshooting guide
- Ownership system clarification

---

## Phase 2: Domain Implementation (3-4 weeks)

### Strategy: Implement in dependency order, parallel work

### Week 1: Foundation Domains

#### 2.1a: Config Domain (2-3 days)
```atlas
struct ServerConfig {
    cmd: Array<string>,
    cwd: string,
    maxRestarts: number,
    restartWindowMs: number,
    // ... 15+ more fields
}

trait ConfigLoader {
    fn load(self, path: string) -> Result<ServerConfig, string>,
    fn save(self, config: ServerConfig) -> Result<void, string>,
}
```
**Dependencies:** File I/O (Phase 1.2)
**Status:** Skeleton exists
**Deliverables:**
- [ ] config.atlas - 200+ lines
- [ ] ConfigLoader trait - Full implementation
- [ ] File I/O integration
- [ ] Error handling

#### 2.1b: Logger Domain (1-2 days)
```atlas
enum LogLevel { Debug, Info, Warn, Error, Fatal }

trait Logger {
    fn debug(mut self, msg: string) -> void,
    fn info(mut self, msg: string) -> void,
    fn warn(mut self, msg: string) -> void,
    fn error(mut self, msg: string) -> void,
}
```
**Dependencies:** None
**Status:** Working, enhance
**Deliverables:**
- [ ] logger.atlas - 150+ lines
- [ ] Structured logging
- [ ] All 5 log levels

#### 2.1c: Transport Domain (3-5 days)
```atlas
enum Protocol { Unknown, NDJSON, LSP }

trait Transport {
    fn read(self) -> Result<string, string>,
    fn write(self, payload: string) -> Result<void, string>,
    fn detectProtocol(self, timeoutMs: number) -> Result<Protocol, string>,
}
```
**Dependencies:** None
**Status:** Working, test thoroughly
**Deliverables:**
- [ ] transport.atlas - 250+ lines
- [ ] Protocol detection - All cases
- [ ] Integration tests

### Week 2: Core State Machine Domains

#### 2.2a: Supervisor Domain (3-5 days)
```atlas
enum ServerState { Stopped, Starting, Running, Restarting, Failed }

trait Supervisor {
    fn start(mut self) -> Result<void, string>,
    fn stop(mut self) -> Result<void, string>,
    fn restart(mut self) -> Result<void, string>,
    fn state(self) -> ServerState,
    fn uptime(self) -> number,
    fn lastError(self) -> string,
}
```
**Dependencies:** Transport, Logger
**Status:** Core proven, needs completeness
**Complexity:** HIGH (state machine)
**Deliverables:**
- [ ] supervisor.atlas - 300+ lines
- [ ] State transitions - All 15+
- [ ] Restart logic - Backoff, limits
- [ ] Monitoring - Uptime, health

#### 2.2b: Sanitizer Domain (2-3 days)
```atlas
enum ChunkType { Empty, JSONRPC, Pollution }

trait Sanitizer {
    fn classify(self, chunk: string) -> ChunkType,
    fn validateUTF8(self, data: Array<number>) -> bool,
    fn redact(self, text: string) -> string,
}
```
**Dependencies:** None
**Status:** Working, enhance
**Complexity:** MEDIUM
**Deliverables:**
- [ ] sanitizer.atlas - 200+ lines
- [ ] All filter types
- [ ] Output cleaning

#### 2.2c: Metrics Domain (2-3 days)
```atlas
trait MetricsCollector {
    fn recordRequest(mut self, method: string, durationMs: number) -> void,
    fn recordQueueWait(mut self, durationMs: number) -> void,
    fn export(self) -> string,  // Prometheus format
}
```
**Dependencies:** None
**Status:** Type inference issues fixed
**Complexity:** LOW
**Deliverables:**
- [ ] metrics.atlas - 150+ lines
- [ ] Prometheus export
- [ ] Health scoring

### Week 3: Complex Domains

#### 2.3a: StateStore Domain (3-4 days)
```atlas
trait StateStore {
    fn setInitialize(mut self, params: string) -> void,
    fn getInitialize(self) -> Option<string>,
    fn addSubscription(mut self, uri: string) -> void,
    fn removeSubscription(mut self, uri: string) -> void,
    fn getSubscriptions(self) -> Array<string>,
}
```
**Dependencies:** HashMap (Phase 1.1)
**Status:** Blocked on HashMap clarity
**Complexity:** MEDIUM
**Deliverables:**
- [ ] statestore.atlas - 200+ lines
- [ ] Subscription tracking
- [ ] Persistence patterns

#### 2.3b: Proxy Domain (4-6 days)
```atlas
trait Proxy {
    fn start(mut self) -> Result<void, string>,
    fn stop(mut self) -> Result<void, string>,
    fn forwardMessage(mut self, msg: QueueMessage) -> Result<void, string>,
    fn status(self) -> ProxyStatus,
}
```
**Dependencies:** Transport, StateStore, Async (Phase 1.3)
**Status:** Complex, async unclear
**Complexity:** VERY HIGH
**Deliverables:**
- [ ] proxy.atlas - 300+ lines
- [ ] Message forwarding
- [ ] Queue implementation
- [ ] Backpressure handling

#### 2.3c: Watcher Domain (3-5 days)
```atlas
trait Watcher {
    fn start(mut self) -> Result<void, string>,
    fn stop(mut self) -> Result<void, string>,
    fn getEvents(self) -> Array<FileWatchEvent>,
}
```
**Dependencies:** File I/O (Phase 1.2), Async (Phase 1.3)
**Status:** Async patterns blocker
**Complexity:** MEDIUM
**Deliverables:**
- [ ] watcher.atlas - 200+ lines
- [ ] File change detection
- [ ] Debouncing

### Week 3-4: Additional Domains

#### 2.4a: Recorder Domain (2-3 days)
```atlas
trait Recorder {
    fn recordRequest(mut self, msg: Message) -> Result<void, string>,
    fn recordResponse(mut self, msg: Message) -> Result<void, string>,
}
```
**Status:** Straightforward
**Complexity:** LOW
**Deliverables:**
- [ ] recorder.atlas - 150+ lines
- [ ] Traffic recording
- [ ] Message storage

#### 2.4b: Security Domain (2-3 days)
```atlas
trait RateLimiter {
    fn check(mut self, key: string) -> bool,
}

trait SizeLimiter {
    fn check(self, size: number) -> bool,
}
```
**Dependencies:** Time module (for rate limiting)
**Status:** Needs time module
**Complexity:** MEDIUM
**Deliverables:**
- [ ] security.atlas - 200+ lines
- [ ] Rate limiting
- [ ] Size limiting

#### 2.4c: Adaptive Domain (1-2 days)
```atlas
struct LearningState {
    requestCount: number,
    averageLatencyMs: number,
    healthScore: number,
}
```
**Status:** Working, enhance
**Complexity:** LOW
**Deliverables:**
- [ ] adaptive.atlas - 150+ lines
- [ ] Health scoring
- [ ] Scaling recommendations

#### 2.4d: Injectable Domain (2-3 days)
```atlas
trait ToolProvider {
    fn getTools(self) -> Array<ToolDefinition>,
    fn getTool(self, name: string) -> Option<ToolDefinition>,
}
```
**Status:** Working, complete tools
**Complexity:** MEDIUM
**Deliverables:**
- [ ] injectable.atlas - 200+ lines
- [ ] 4 tool implementations
- [ ] Tool execution

### Week 4: CLI & Main

#### 2.5a: CLI Domain (4-5 days)
```atlas
// 15+ commands: init, run, status, logs, stop, restart
trait Command {
    fn execute(mut self, args: Array<string>) -> Result<void, string>,
}
```
**Dependencies:** All above domains
**Status:** Integration layer
**Complexity:** MEDIUM
**Deliverables:**
- [ ] cli.atlas - 300+ lines
- [ ] 15+ commands
- [ ] Help system
- [ ] Error handling

#### 2.5b: Main Entry Point (2-3 days)
```atlas
fn main() -> void {
    let config = loadConfig("hydra.toml"),
    let supervisor = createSupervisor(config),
    // ... wire up components
}
```
**Dependencies:** All domains
**Status:** Integration
**Complexity:** LOW
**Deliverables:**
- [ ] main.atlas - 100+ lines
- [ ] Configuration loading
- [ ] Component initialization
- [ ] Graceful shutdown

---

## Phase 3: Testing & Validation (2-3 weeks)

### 3.1: Unit Tests (1 week)
```
[ ] Test each domain independently
[ ] 10-20 test cases per domain
[ ] Cover all error paths
[ ] Target: 85%+ code coverage
[ ] 100+ test cases total
```

### 3.2: Integration Tests (1 week)
```
[ ] Test component interactions
[ ] Message forwarding pipeline
[ ] State consistency across domains
[ ] Error recovery patterns
[ ] 30+ test scenarios
```

### 3.3: Performance Tests (3-5 days)
```
[ ] Measure throughput (messages/sec)
[ ] Measure latency (P50, P95, P99)
[ ] Memory usage over time
[ ] Compare vs Go version
[ ] Optimize hot paths if needed
```

### 3.4: Compatibility Tests (3-5 days)
```
[ ] Test with real Claude API
[ ] Test MCP protocol compliance
[ ] Test with various message patterns
[ ] Test large payloads (100MB+ JSON)
[ ] Test concurrent clients (100+)
```

---

## Phase 4: Documentation & Release (1-2 weeks)

### 4.1: Code Review
```
[ ] Review all 3000+ lines of code
[ ] Check for consistent patterns
[ ] Verify error handling
[ ] Optimize performance
[ ] Security review
```

### 4.2: Documentation
```
[ ] API reference - All public methods
[ ] Architecture guide - Component overview
[ ] Configuration guide - All options
[ ] Deployment guide - Production setup
[ ] Troubleshooting guide - Common issues
[ ] Migration guide - From Go version
[ ] Examples - All working code samples
```

### 4.3: Release
```
[ ] Version 1.0 initial release
[ ] Changelog - All changes documented
[ ] Migration path - For Go users
[ ] Community setup - Issues, discussions
```

---

## Risk Mitigation

### Risk 1: Async Patterns Unclear (HIGH)
**Impact:** Watcher, Proxy async blocked
**Mitigation:** Phase 1.3 validates
**Contingency:** Single-threaded message processing

### Risk 2: HashMap Not in Stdlib (HIGH)
**Impact:** StateStore, caching blocked
**Mitigation:** Phase 1.1 validates
**Contingency:** Custom HashMap or Array<Tuple> workaround

### Risk 3: File I/O Incomplete (MEDIUM)
**Impact:** Config loading, persistence
**Mitigation:** Phase 1.2 validates
**Contingency:** Environment variables for config

### Risk 4: Type Inference Issues (MEDIUM)
**Impact:** All domains need workarounds
**Mitigation:** Phase 1.5 investigates
**Contingency:** Explicit type annotations

### Risk 5: Performance Gap (MEDIUM)
**Impact:** Atlas port may be slower
**Mitigation:** Phase 3.3 measures
**Contingency:** Profile and optimize

---

## Success Criteria

### Phase 1: All APIs verified with examples
### Phase 2: All 13+ domains ported and compiling
### Phase 3: 100+ unit tests, 30+ integration tests passing
### Phase 4: Comprehensive documentation, zero known bugs

---

## Timeline

| Phase | Duration | Effort |
|:------|:---------|:-------|
| 1: Validation | 1-2 weeks | 40-60 hours |
| 2: Implementation | 3-4 weeks | 120-160 hours |
| 3: Testing | 2-3 weeks | 80-120 hours |
| 4: Documentation | 1-2 weeks | 40-60 hours |
| **TOTAL** | **8-10 weeks** | **280-400 hours** |

---

**Plan Created:** 2026-03-08
**Status:** Ready for execution
