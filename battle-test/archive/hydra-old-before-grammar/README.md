# Hydra Atlas: Battle Test Implementation

## Overview

This is a **complete port of Hydra** (MCP server supervisor) from Go to Atlas, serving as a real-world battle test for Atlas's systems programming capabilities.

## Original Hydra (Go)

**Source:** `~/dev/projects/hydra`

Hydra is a fault-tolerant supervisor for Model Context Protocol (MCP) servers:
- Process lifecycle management (spawn, restart, crash recovery)
- Stdio sanitization (filters noise from JSON-RPC stream)
- File watching with hot-reload
- Session state persistence
- Health monitoring and metrics
- ~36 Go files, ~4,000 lines of production code

## Why This Is The Perfect Battle Test

Hydra tests Atlas on:

### Systems Programming
- ✅ Process spawning with `exec()`
- ✅ Stdio piping and filtering
- ✅ File system watching
- ✅ State machines (STOPPED/STARTING/RUNNING/RESTARTING/FAILED)
- ⚠️ Signal handling (need to verify Atlas support)

### Concurrency
- ✅ Async operations (Go goroutines → Atlas async)
- ✅ Synchronization primitives (RwLock, Semaphore, Atomic)
- ⚠️ Channels (need to verify Atlas implementation)
- ✅ Race-free code

### Real-World Complexity
- JSON-RPC protocol handling
- Error recovery patterns
- Metrics collection
- Health scoring algorithms
- Configuration management

## Architecture

```
hydra-atlas/
├── main.atlas           # Entry point
├── config/              # Configuration (load/merge/validate)
├── transport/           # Stdio + JSON-RPC
├── supervisor/          # Process lifecycle
├── sanitizer/           # Stdout filtering
├── statestore/          # Session persistence
├── watcher/             # File change detection
├── proxy/               # Main orchestrator
├── logger/              # Structured logging
├── security/            # Redaction, rate limiting
└── utils/               # Helpers
```

## Implementation Progress

| Component | Status | Notes |
|-----------|--------|-------|
| Project Setup | 🏗️ In Progress | Creating structure |
| Transport | ⏳ Pending | Stdio + JSON-RPC parsing |
| Supervisor | ⏳ Pending | Process management |
| Sanitizer | ⏳ Pending | Output filtering |
| StateStore | ⏳ Pending | Session persistence |
| Watcher | ⏳ Pending | File monitoring |
| Proxy | ⏳ Pending | Orchestration layer |
| Config | ⏳ Pending | Config management |
| Battle Tests | ⏳ Pending | Real-world scenarios |

## Atlas Capabilities Being Tested

### Currently Available
- `exec()` - Process execution
- `readFile()` / `writeFile()` - File I/O
- `json_parse()` / `json_stringify()` - JSON handling
- `rwLockNew()` - Read-write locks
- `semaphoreNew()` - Semaphores
- `atomicNew()` - Atomic operations

### Need To Verify
- Channels for inter-task communication
- Async/spawn for concurrent operations
- Signal handling for graceful shutdown
- Stdio piping for child processes

## Goals

1. **Functional Parity:** Recreate all core Hydra functionality in Atlas
2. **Performance:** Match or exceed Go version's performance
3. **Code Quality:** Demonstrate Atlas's expressiveness for systems programming
4. **Battle Test:** Identify Atlas stdlib gaps and areas for improvement

## Metrics

**Target Performance:**
- Proxy latency (P50): < 50ms
- Restart time (P50): < 500ms
- Memory usage: < 100MB
- CPU (idle): < 1%

**Comparison to Go:**
- TBD after implementation

## Running

```bash
# (Not yet implemented)
atlas run main.atlas --name my-server
```

## Learning Outcomes

This project will reveal:
- ✅ What Atlas does well for systems programming
- ⚠️ What's missing or needs improvement
- 📊 Performance characteristics under real load
- 🎯 Ergonomics for AI code generation

---

**Status:** Foundation phase - building core modules
