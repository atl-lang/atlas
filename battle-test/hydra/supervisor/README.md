# Supervisor - Process Lifecycle Management

**Status**: ⚠️ Partial - Concepts proven, async limitation found

## Overview

Supervisor demonstrates process management patterns but reveals a critical limitation in Atlas v0.2: **no async process spawning**.

## What Works ✅

### State Machine (5 States)
```atlas
STATE_STOPPED    -> 0
STATE_STARTING   -> 1
STATE_RUNNING    -> 2
STATE_RESTARTING -> 3
STATE_FAILED     -> 4
```

### Process Execution
```atlas
let result: Result<json, string> = exec(["command", "arg"]);
// Returns: { exitCode, stdout, stderr, success }
```

### Health Checking
```atlas
let healthy: bool = health_check(["echo", "ping"]);
```

### Retry Logic
```atlas
let succeeded: bool = run_with_retry(command, max_attempts);
```

### Restart Cycles
```atlas
let result: string = simulate_restart_cycle(command, max_restarts);
// Returns: "SUCCESS" | "RESTARTED" | "FAILED"
```

## Critical Limitation ⚠️

**exec() is synchronous** - it blocks until the process completes.

### Impact
- ✅ Can run quick commands (health checks, status queries)
- ✅ Can demonstrate supervisor patterns
- ❌ **Cannot manage long-running processes** (MCP servers)
- ❌ Cannot spawn background processes
- ❌ Cannot monitor process while it runs

### What's Missing
Atlas v0.2 lacks:
1. **Async process spawn** - `spawn(command)` for external processes
2. **Process handles** - Objects representing running processes
3. **Non-blocking wait** - Check if process still running
4. **Process signals** - Send SIGTERM, SIGKILL, etc.

### Workarounds Attempted
- ✗ `spawn(array)` - Wrong signature (expects Future, not command)
- ✗ Background exec - No API for this
- ✗ Shell background (`&`) - exec() still blocks

## Implementation

### Files
- `supervisor_v2.atl` - Core implementation (working patterns)
- `test_supervisor_final.atl` - Battle test (all passing ✅)
- `test_background.atl` - Confirms exec() blocks
- `test_spawn.atl` - Confirms spawn() is for Atlas futures

### API

```atlas
// State management
state_name(state: number) -> string

// Command execution
execute_command(command: array) -> json
check_success(output: json) -> bool

// Retry and restart
run_with_retry(command: array, max_attempts: number) -> bool
simulate_restart_cycle(command: array, max_restarts: number) -> string

// Health
health_check(command: array) -> bool
```

## Battle Test Results

Run: `atlas run test_supervisor_final.atl`

**All tests passing** ✅

```
✅ State machine (5 states)
✅ Process execution
✅ Health checking
✅ Retry logic
✅ Restart cycles
```

**But**: Limited to synchronous, short-lived commands.

## Recommendations

### For Full Hydra Port
**Blocked**: Needs Atlas v0.3+ with async process APIs:
- `spawnProcess(command) -> ProcessHandle`
- `processWait(handle) -> Result<ExitStatus>`
- `processKill(handle, signal)`
- `processIsRunning(handle) -> bool`

### For Current Capabilities
**Feasible**: Supervisor for batch jobs or command runners
- Health check scripts
- Status monitors
- Deployment automation
- Test runners

## Learnings

1. **exec() confirmed working** - Synchronous execution solid
2. **Patterns proven** - State machine, retry, restart logic all work
3. **Async gap identified** - This is the blocker for Hydra
4. **Alternative use cases** - Supervisor still useful for non-daemon processes

## Next Steps

**Option A**: Wait for Atlas async process APIs
**Option B**: Implement remaining components that don't need this
- ✅ StateStore (done)
- 🔧 Watcher (file monitoring - should work)
- 🔧 Config (configuration - should work)
- ⏳ Proxy (depends on Supervisor - blocked)

---

**Completion**: Patterns complete, async blocker prevents production use
**Test Coverage**: 100% of implemented features
**Atlas Feedback**: Need async process spawning APIs for daemon management
