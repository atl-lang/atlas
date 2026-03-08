# Hydra → Atlas Port Implementation Roadmap

**Date:** 2026-03-08
**Based on:** Comprehensive audit of 13 domains (37+ friction points documented)
**Objective:** Complete production-ready port of Hydra to Atlas v0.3

---

## Current Status

| Category | Status | Confidence |
|:---------|:-------|:-----------|
| **Language features** | ✅ Ready | 95% - Traits work, state machines possible |
| **Core domains** | ✅ Ready | 90% - Transport, Supervisor, Sanitizer proven |
| **Stdlib coverage** | ⚠️ Partial | 60% - HashMap unclear, File I/O untested, Async unclear |
| **Documentation** | ⚠️ Outdated | 40% - API deprecated, docs don't match compiler |
| **Production ready** | ⚠️ Nearly | 65% - Needs stdlib clarity and stability |

---

## Phase 1: Validation & Clarification (1-2 weeks)

### Goal
Verify all stdlib claims and document exact API surfaces before porting.

### Tasks

#### 1.1: Verify HashMap (H-164)
```
[ ] Test HashMap.new() vs hashMapNew()
[ ] Document iteration patterns (keys, values, entries)
[ ] Test concurrent access patterns
[ ] Verify performance characteristics
[ ] Document in stdlib guide
```

**Owner:** TBD
**Effort:** 3-4 days
**Blocker for:** StateStore, Proxy, others

#### 1.2: Test File I/O (H-165)
```
[ ] Review file.md thoroughly
[ ] Test File.read(), File.write(), File.append()
[ ] Test directory operations
[ ] Document error handling patterns
[ ] Test relative vs absolute paths
```

**Owner:** TBD
**Effort:** 2-3 days
**Blocker for:** Config loading, persistence

#### 1.3: Clarify Async Patterns (H-166)
```
[ ] Review async.md and tokio integration
[ ] Test async/await syntax
[ ] Test task spawning equivalent
[ ] Test cancellation patterns
[ ] Document real concurrency vs OS threads
```

**Owner:** TBD
**Effort:** 3-5 days
**Blocker for:** Watcher, Proxy async forwarding

#### 1.4: Verify API Transitions (H-150)
```
[ ] Test both old and new function signatures
[ ] Determine which will be standard
[ ] Check deprecation timeline
[ ] Plan migration strategy
```

**Owner:** TBD
**Effort:** 2-3 days
**Impact:** High - affects all code

#### 1.5: Fix Type Inference Issues (H-151)
```
[ ] Investigate Ok(void) type inference
[ ] Investigate match assignment type loss
[ ] Investigate empty array [] type inference
[ ] Propose solutions or workarounds
```

**Owner:** TBD
**Effort:** 3-5 days
**Impact:** All domains

#### 1.6: Clarify Ownership System (H-149)
```
[ ] Document borrow/own keyword syntax
[ ] Test scenarios where these apply
[ ] Clarify when Copy vs move
[ ] Update documentation
```

**Owner:** TBD
**Effort:** 2-3 days

### Phase 1 Deliverables
- [ ] HashMap guide with examples
- [ ] File I/O API reference
- [ ] Async patterns tutorial
- [ ] API migration guide
- [ ] Type inference clarifications
- [ ] Ownership system documentation

---

## Phase 2: Domain Implementation (3-4 weeks)

### Strategy
Implement domains in order of dependency and simplicity. Parallel work where possible.

### Phase 2.1: Foundation Domains (Week 1)

#### 2.1a: Config Domain
```atlas
// Load Hydra configuration files
struct ServerConfig {
    cmd: Array<string>,
    cwd: string,
    // ... 15+ fields
}

trait ConfigLoader {
    fn load(self, path: string) -> Result<ServerConfig, string>,
    fn save(self, config: ServerConfig) -> Result<void, string>,
}
```

**Estimated effort:** 2-3 days
**Dependencies:** None
**Status:** Skeleton exists, needs File I/O

**Deliverables:**
- [ ] config.atlas - Complete with file loading
- [ ] ConfigLoader trait - Fully implemented
- [ ] Error handling - All Go error cases covered

#### 2.1b: Logger Domain
```atlas
enum LogLevel { Debug, Info, Warn, Error, Fatal }

trait Logger {
    fn debug(mut self, msg: string) -> void,
    fn info(mut self, msg: string) -> void,
    fn error(mut self, msg: string) -> void,
    // ... structured logging
}
```

**Estimated effort:** 1-2 days
**Dependencies:** None
**Status:** Working, simple enhancement

**Deliverables:**
- [ ] logger.atlas - Enhanced with all Go features
- [ ] Structured logging - JSON output
- [ ] Log levels - All 5 levels working

#### 2.1c: Transport Domain
```atlas
enum Protocol { Unknown, NDJSON, LSP }

trait Transport {
    fn read(self) -> Result<string, string>,
    fn write(self, payload: string) -> Result<void, string>,
    fn detectProtocol(self, timeoutMs: number) -> Result<Protocol, string>,
}
```

**Estimated effort:** 3-5 days
**Dependencies:** None
**Status:** Working, needs stress testing

**Deliverables:**
- [ ] transport.atlas - Complete implementation
- [ ] StdioTransport - Fully functional
- [ ] Protocol detection - All cases covered
- [ ] Integration tests - Round-trip testing

### Phase 2.2: Core Domains (Week 2)

#### 2.2a: Supervisor Domain
```atlas
enum ServerState { Stopped, Starting, Running, Restarting, Failed }

trait Supervisor {
    fn start(mut self) -> Result<void, string>,
    fn stop(mut self) -> Result<void, string>,
    fn restart(mut self) -> Result<void, string>,
    fn state(self) -> ServerState,
    fn uptime(self) -> number,
}
```

**Estimated effort:** 3-5 days
**Dependencies:** Transport, Logger
**Status:** Core functionality proven, needs complete coverage

**Deliverables:**
- [ ] supervisor.atlas - 300+ lines, all methods
- [ ] State transitions - All 15+ transitions
- [ ] Restart logic - Backoff, limits, cleanup
- [ ] Monitoring - Uptime, health checks
- [ ] Tests - State machine testing

#### 2.2b: Sanitizer Domain
```atlas
enum ChunkType { Empty, JSONRPC, Pollution }

trait Sanitizer {
    fn classify(self, chunk: string) -> ChunkType,
    fn validateUTF8(self, data: Array<number>) -> bool,
    fn redact(self, text: string) -> string,
}
```

**Estimated effort:** 2-3 days
**Dependencies:** None
**Status:** Working, needs completeness

**Deliverables:**
- [ ] sanitizer.atlas - All filter types
- [ ] Output cleaning - All patterns
- [ ] UTF-8 handling - Invalid sequence handling
- [ ] Performance - Streaming patterns

#### 2.2c: Metrics Domain
```atlas
trait MetricsCollector {
    fn recordRequest(mut self, method: string, durationMs: number) -> void,
    fn recordQueueWait(mut self, durationMs: number) -> void,
    fn export(self) -> string,  // Prometheus format
}
```

**Estimated effort:** 2-3 days
**Dependencies:** None
**Status:** Type inference issues fixed

**Deliverables:**
- [ ] metrics.atlas - All collectors
- [ ] Prometheus export - Correct format
- [ ] Health scoring - Weighted calculation

### Phase 2.3: Complex Domains (Week 3)

#### 2.3a: StateStore Domain
```atlas
trait StateStore {
    fn setInitialize(mut self, params: string) -> void,
    fn getInitialize(self) -> Option<string>,
    fn addSubscription(mut self, uri: string) -> void,
    fn getSubscriptions(self) -> Array<string>,
    // ... subscription tracking
}
```

**Estimated effort:** 3-4 days
**Dependencies:** HashMap availability (H-164)
**Status:** Blocker on HashMap clarity

**Deliverables:**
- [ ] statestore.atlas - Complete state management
- [ ] Subscription tracking - HashMap or Array approach
- [ ] Persistence - File-based if needed
- [ ] Tests - State consistency

#### 2.3b: Proxy Domain
```atlas
trait Proxy {
    fn start(mut self) -> Result<void, string>,
    fn stop(mut self) -> Result<void, string>,
    fn forwardMessage(mut self, msg: QueueMessage) -> Result<void, string>,
    fn status(self) -> ProxyStatus,
}
```

**Estimated effort:** 4-6 days
**Dependencies:** Transport, StateStore, Async patterns (H-166)
**Status:** Complex state machine, async TBD

**Deliverables:**
- [ ] proxy.atlas - Full message forwarding
- [ ] Queue implementation - Efficient dequeue
- [ ] State machine - All transitions
- [ ] Async forwarding - If supported
- [ ] Backpressure handling - Queue limits

#### 2.3c: Watcher Domain
```atlas
trait Watcher {
    fn start(mut self) -> Result<void, string>,
    fn stop(mut self) -> Result<void, string>,
    fn getEvents(self) -> Array<FileWatchEvent>,
    fn waitForChange(self, timeoutMs: number) -> Option<FileWatchEvent>,
}
```

**Estimated effort:** 3-5 days
**Dependencies:** File I/O (H-165), Async patterns (H-166)
**Status:** Async patterns blocker

**Deliverables:**
- [ ] watcher.atlas - File change detection
- [ ] Debouncing - Configurable debounce
- [ ] Event queue - Efficient buffering
- [ ] Platform specifics - OS-specific handling if needed

### Phase 2.4: Additional Domains (Week 3-4)

#### 2.4a: Recorder Domain
```atlas
trait Recorder {
    fn recordRequest(mut self, msg: QueueMessage) -> Result<void, string>,
    fn recordResponse(mut self, msg: QueueMessage) -> Result<void, string>,
    fn getRecording(self, sessionId: string) -> Array<RecordedMessage>,
}
```

**Estimated effort:** 2-3 days
**Dependencies:** None
**Status:** Straightforward, type inference fixed

**Deliverables:**
- [ ] recorder.atlas - Traffic recording
- [ ] Message storage - Efficient format
- [ ] Playback - For debugging

#### 2.4b: Security Domain
```atlas
trait RateLimiter {
    fn check(mut self, key: string) -> bool,
    fn reset(mut self, key: string) -> void,
}

trait SizeLimiter {
    fn check(self, size: number) -> bool,
}
```

**Estimated effort:** 2-3 days
**Dependencies:** Time module (for rate limiting)
**Status:** Blocker on time module

**Deliverables:**
- [ ] security.atlas - Rate limiting, size limiting
- [ ] Token bucket - Time-based algorithm
- [ ] Redaction - Pattern-based filtering

#### 2.4c: Adaptive Domain
```atlas
struct LearningState {
    requestCount: number,
    averageLatencyMs: number,
    healthScore: number,
}

trait Learner {
    fn recordObservation(mut self, latencyMs: number) -> void,
    fn getHealthScore(self) -> number,
    fn shouldScale(self) -> bool,
}
```

**Estimated effort:** 1-2 days
**Dependencies:** None
**Status:** Working, enhancement only

**Deliverables:**
- [ ] adaptive.atlas - Health scoring, learning
- [ ] Scaling recommendations - Based on metrics
- [ ] Weighted calculations - Go algorithm ported

#### 2.4d: Injectable Domain
```atlas
trait ToolProvider {
    fn getTools(self) -> Array<ToolDefinition>,
    fn getTool(self, name: string) -> Option<ToolDefinition>,
    fn executeTool(mut self, name: string, params: string) -> Result<string, string>,
}
```

**Estimated effort:** 2-3 days
**Dependencies:** None
**Status:** Working, needs all 4 tool implementations

**Deliverables:**
- [ ] injectable.atlas - Tool provider
- [ ] hydra_restart tool - Full implementation
- [ ] hydra_status tool - Full implementation
- [ ] hydra_logs tool - Full implementation
- [ ] hydra_signal tool - Full implementation

### Phase 2.5: CLI & Main (Week 4)

#### 2.5a: CLI Domain
```atlas
// 15+ commands: init, run, status, logs, stop, restart, etc.
trait Command {
    fn execute(mut self, args: Array<string>) -> Result<void, string>,
    fn help(self) -> string,
}
```

**Estimated effort:** 4-5 days
**Dependencies:** All above domains
**Status:** Integration layer

**Deliverables:**
- [ ] cli.atlas - Command dispatcher
- [ ] 15+ commands - All functional
- [ ] Help system - Comprehensive
- [ ] Error handling - User-friendly messages

#### 2.5b: Main Entry Point
```atlas
// Bootstrap and connect all domains
fn main() -> void {
    let config = loadConfig("hydra.toml"),
    let supervisor = createSupervisor(config),
    let proxy = createProxy(),
    // ... wire up all components
}
```

**Estimated effort:** 2-3 days
**Dependencies:** All domains
**Status:** Integration and testing

**Deliverables:**
- [ ] main.atlas - Entry point
- [ ] Configuration loading - All options
- [ ] Component setup - Proper initialization
- [ ] Cleanup - Graceful shutdown

---

## Phase 3: Testing & Validation (2-3 weeks)

### Strategy
Comprehensive testing to ensure Atlas port behaves identically to Go version.

#### 3.1: Unit Tests (1 week)
```atlas
// For each domain: 10-20 test cases
test("Supervisor starts process", fn() {
    let supervisor = newSupervisor(...),
    let result = supervisor.start(),
    assert(result.isOk()),
    assert(supervisor.state() == ServerState::Running),
})
```

**Deliverables:**
- [ ] Unit tests - 100+ test cases total
- [ ] Coverage - 85%+ code coverage
- [ ] Edge cases - All error paths

#### 3.2: Integration Tests (1 week)
```atlas
// Test component interactions
test("Full message forwarding pipeline", fn() {
    let proxy = newProxy(),
    let transport = newStdioTransport(),
    proxy.start(),
    let msg = QueueMessage{...},
    let result = proxy.forwardMessage(msg),
    assert(result.isOk()),
})
```

**Deliverables:**
- [ ] Integration tests - 30+ test scenarios
- [ ] Message forwarding - End-to-end
- [ ] State consistency - Across domains
- [ ] Error recovery - Fault tolerance

#### 3.3: Performance Tests (3-5 days)
```atlas
// Measure latency, throughput, memory
test("Process 1000 messages/sec", fn() {
    // Measure time for 1000 message forwards
    // Target: < 1ms per message
})
```

**Deliverables:**
- [ ] Throughput tests - Messages/sec
- [ ] Latency tests - P50, P95, P99
- [ ] Memory tests - Stable or growing
- [ ] Comparison - vs Go version

#### 3.4: Compatibility Tests (3-5 days)
```
[ ] Test with real Claude API
[ ] Test MCP protocol compliance
[ ] Test with various message patterns
[ ] Test large payloads (100MB+ JSON)
[ ] Test concurrent clients (100+)
```

**Deliverables:**
- [ ] MCP compliance - Full protocol
- [ ] Real-world testing - Production patterns
- [ ] Performance comparison - vs Go

---

## Phase 4: Documentation & Release (1-2 weeks)

### Documentation
```
[ ] API reference - All public methods
[ ] Architecture guide - Component overview
[ ] Configuration guide - All options
[ ] Deployment guide - Production setup
[ ] Troubleshooting - Common issues
[ ] Migration guide - From Go version
```

### Quality Assurance
```
[ ] Code review - All 3000+ lines
[ ] Documentation review - Accuracy
[ ] Example code - All working
[ ] Performance review - Optimization
[ ] Security review - No vulnerabilities
```

### Release
```
[ ] Version 1.0 - Initial release
[ ] Changelog - All changes documented
[ ] Migration path - For Go users
[ ] Support - Community ready
```

---

## Risk Mitigation

### Risk 1: Async Patterns Unclear
**Risk Level:** HIGH
**Impact:** Watcher, Proxy async forwarding blocked
**Mitigation:**
- Phase 1.3 validates async capability
- If not available: Design sync-only version
- Contingency: Single-threaded message processing

### Risk 2: HashMap Not in Stdlib
**Risk Level:** HIGH
**Impact:** StateStore, caching patterns blocked
**Mitigation:**
- Phase 1.1 validates HashMap API
- If not available: Use Array<Tuple> workaround
- Contingency: Custom HashMap implementation

### Risk 3: File I/O Incomplete
**Risk Level:** MEDIUM
**Impact:** Config loading, persistence
**Mitigation:**
- Phase 1.2 validates all File APIs
- If not available: Implement workarounds
- Contingency: Environment variables for config

### Risk 4: Type Inference Issues
**Risk Level:** MEDIUM
**Impact:** All domains need workarounds
**Mitigation:**
- Phase 1.5 investigates solutions
- Workarounds documented in code
- Consider type helper functions

### Risk 5: Performance Gap
**Risk Level:** MEDIUM
**Impact:** Atlas port may be slower
**Mitigation:**
- Phase 3.3 measures performance
- Profile and optimize hot paths
- Compare against Go baseline
- If significant: Consider Rust FFI

---

## Success Criteria

### Phase 1: Validation
- [ ] All stdlib APIs verified with code examples
- [ ] All blockers identified and documented
- [ ] Workaround strategy for each friction point
- [ ] Team alignment on approach

### Phase 2: Implementation
- [ ] All 13+ domains ported
- [ ] Each compiles successfully
- [ ] Core functionality matches Go version
- [ ] Code review approval

### Phase 3: Testing
- [ ] 100+ unit tests passing
- [ ] 30+ integration tests passing
- [ ] Performance within 20% of Go version
- [ ] Zero known bugs

### Phase 4: Documentation
- [ ] Comprehensive API reference
- [ ] Clear architecture documentation
- [ ] All examples tested
- [ ] Migration guide complete

---

## Timeline Summary

| Phase | Duration | Effort |
|:------|:---------|:-------|
| **Phase 1: Validation** | 1-2 weeks | 40-60 hours |
| **Phase 2: Implementation** | 3-4 weeks | 120-160 hours |
| **Phase 3: Testing** | 2-3 weeks | 80-120 hours |
| **Phase 4: Documentation** | 1-2 weeks | 40-60 hours |
| **Total** | **7-11 weeks** | **280-400 hours** |

**Estimated Effort:** 2-3 person-weeks of dedicated work

---

## Next Steps (IMMEDIATE)

1. **Review this roadmap** - Does it match priorities?
2. **Validate Phase 1 assumptions** - Can we run these tests?
3. **Assign owners** - Who leads each phase?
4. **Plan Phase 1.1** - Start HashMap validation immediately
5. **Plan Phase 1.2** - Start File I/O testing
6. **Plan Phase 1.3** - Start async pattern research

---

## Questions for Stakeholders

1. **Async Support** - Is async/await critical or optional?
2. **Timeline** - Is 8-10 week timeline acceptable?
3. **Performance** - What's acceptable overhead vs Go?
4. **Production** - Production deployment needed before v1.0?
5. **Resources** - How many developers assigned?

---

**Roadmap Created:** 2026-03-08
**Based on:** Comprehensive 5-hour audit of 13 domains
**Author:** Atlas-Hydra Audit Team
**Status:** Ready for stakeholder review

