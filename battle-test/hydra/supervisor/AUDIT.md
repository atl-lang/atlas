# Component Audit: Supervisor

**Grammar Friction Points**
- No goroutines/threads; background monitoring must be polled or simulated with async primitives.
- No field assignment on records; supervisor updates rebuild records each time.
- `if` requires parentheses; parser errors are misleading without them.
- `match` arms require commas and a trailing `;` when used as a statement.
- `hashMapPut` returns a new map; all state updates must reassign.

**Missing Features (Atlas should have)**
- Process stdlib with stdin/stdout pipes and async wait semantics (non-polling).
- Timer/ticker utilities for restart windows and watchdog loops.
- Structured error types instead of stringly-typed `Result<string>`.
- Sandbox policy introspection or allowed-list APIs for process spawning.

**Syntax Quality Rating:** 6/10

**AI Generation Experience**
- Process APIs exist (`spawnProcess`, `processWait`), but lack of streaming I/O blocks full parity.
- Restart logic is feasible, but real crash monitoring and IO wiring are missing.
- Tests had to fake running state because `spawnProcess` is permission-blocked in this runtime.
