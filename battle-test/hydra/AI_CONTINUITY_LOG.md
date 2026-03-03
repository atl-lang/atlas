# Hydra → Atlas Battle Test (AI Continuity Log)

Date: 2026-03-03
Scope: Codebase-only scan of `/Users/proxikal/dev/projects/hydra` and Atlas runtime/grammar implementation.
Goal: Full-port attempt with friction/gap tracking for AI continuity.

---

## Status
- Battle-test workspace: `/Users/proxikal/dev/projects/atlas/battle-test/hydra`
- Source: `/Users/proxikal/dev/projects/hydra`
- Port target: Atlas

---

## Friction/Gaps (Live Log)

### 1) Grammar ambiguity: `{}` used for blocks, object literals, and structural types
- Impact: High AI confusion; JSON/object literal generation fails.
- Atlas codebase behavior: Object literal parsing uses backtracking.
- Recommendation: Disambiguate object literals with explicit syntax (e.g., `object { ... }`).

### 2) Object literals compile to HashMap but typecheck as `Unknown`
- Impact: Object literal can be assigned to any type, including `json`.
- Recommendation: Introduce a concrete object/record type or disallow object literals until typed.

### 3) `Unknown` type assignable to everything
- Impact: Hides errors, masks invalid conversions.
- Recommendation: Restrict Unknown to error recovery only; disallow Unknown → json.

### 4) `parseJSON` returns Result but common code paths expect `json`
- Impact: AI frequently skips unwrapping; errors cascade.
- Recommendation: Either make parseJSON throw or require `?` usage consistently.

### 5) No explicit JSON literal syntax
- Impact: AI uses `{}` assuming json; incorrect.
- Recommendation: Add explicit JSON literal or `json { ... }` form.

### 6) Multiple syntaxes for same concept (var/let mut, arrow/fn, C-for/for-in)
- Impact: Increased AI variance.
- Recommendation: Choose canonical syntax and remove redundant forms.

### 7) Struct expr parsing depends on uppercase naming convention
- Impact: Grammar uses naming convention rather than syntax rule.
- Recommendation: Add explicit struct literal syntax/keyword.

---

## Port Coverage Map (Per Hydra Package)

Status key: NOT STARTED / PARTIAL / COMPLETE

- adaptive: NOT STARTED
- bootstrap: NOT STARTED
- cli: NOT STARTED
- config: PARTIAL (existing Atlas files from prior hydra-atlas copy)
- discovery: NOT STARTED
- injectable: NOT STARTED
- logger: NOT STARTED
- metrics: NOT STARTED
- mocks: NOT STARTED
- protocol: NOT STARTED
- proxy: NOT STARTED
- recorder: NOT STARTED
- sanitizer: PARTIAL (existing Atlas files)
- security: NOT STARTED
- state: NOT STARTED
- statestore: PARTIAL (existing Atlas files)
- supervisor: PARTIAL (existing Atlas files)
- transport: PARTIAL (existing Atlas files)
- watcher: NOT STARTED

---

## Next Actions
- Create Atlas module skeletons for each Hydra package.
- Port Go functionality module-by-module, replacing stubs with real Atlas logic.
- Update this log with each friction/gap encountered during port.

---

## Port Actions Taken
- Created Atlas battle-test workspace at `/Users/proxikal/dev/projects/atlas/battle-test/hydra`.
- Copied existing Atlas Hydra artifacts (transport/supervisor/sanitizer/statestore) as baseline.
- Auto-generated skeleton `.atl` files for each non-test Go file under `hydra/internal/*` where no Atlas file existed.
- Added minimal `main.atl` and `cli/root.atl` stubs to mirror Go entrypoint.

## Critical Gaps Blocking Full Port
- Skeletons are placeholders; functional parity not yet achieved.
- Atlas lacks clear object/record type semantics to map Go structs and map types cleanly.
- Go interfaces and error handling patterns need explicit Atlas equivalents (traits/results).
- Concurrency patterns (goroutines, channels, contexts) need Atlas async/channel primitives to complete the port.

---

## Concurrency/Runtime Features Used in Hydra (From Code Scan)

### Goroutines + Channels + Select
- Found goroutines (`go func()`), channels (`chan`), and `select` in:
  - supervisor/process.go
  - watcher/fswatch.go
  - proxy/proxy.go
  - state/cleanup.go (tests)
- **Atlas gap:** Need stable async + channels + select semantics to port without invasive rewrites.
- **Recommendation:** Provide channel primitives + select or async await combinators.

### Context Cancellation
- `context.WithCancel` used in supervisor/process.go
- **Atlas gap:** No equivalent cancellation token in language core; needs explicit cancellation API.

### Mutex / RWMutex / WaitGroup
- Used in multiple modules (adaptive, recorder, state, statestore, metrics, security).
- **Atlas gap:** Requires synchronization primitives; current Atlas stdlib has limited concurrency helpers.

### Timers / Tickers / Time Durations
- Widespread use of `time.Duration`, `time.After`, `time.NewTicker`, `time.Sleep`, `time.Now`.
- **Atlas gap:** Needs consistent duration type + timers + ticker API.

### OS-Specific Paths / runtime.GOOS
- `runtime.GOOS` used in bootstrap/pathutil.go and state/cleanup.go.
- **Atlas gap:** Needs OS detection API and platform-specific path utilities.

---

## OS / FS / Process Features Used in Hydra

### Process Execution
- Uses `os/exec` to spawn MCP servers (supervisor/process.go).
- **Atlas gap:** Requires robust `exec` + stdio piping + process lifecycle control APIs.

### File System Watcher
- Uses `fsnotify` (watcher/fswatch.go).
- **Atlas gap:** No fsnotify equivalent; needs file system watcher API.

### File IO / Path / Env
- Uses `os.ReadFile`, `os.WriteFile`, `os.MkdirAll`, `os.Remove`, `os.Stat`, `os.ReadDir`.
- Uses `filepath.Join`, `filepath.Clean`, `filepath.Ext`, `filepath.Glob`.
- Uses `os.Getenv`, `os.Setenv`, `os.Unsetenv`, `os.UserHomeDir`.
- **Atlas gap:** Requires complete file/path/env API coverage.

### Process Introspection
- Uses `os.Getpid`, `os.FindProcess`, `runtime.GOOS`.
- **Atlas gap:** Requires process APIs + OS detection.

---

## Port Progress Update
- Implemented Atlas logger module in `/Users/proxikal/dev/projects/atlas/battle-test/hydra/logger/logger.atl`.

## Logger Gaps
- No stderr output channel in Atlas stdlib; falling back to `print()`.
- No structured logging or log-level filtering in stdlib; level ignored.

---

## Config Port Notes
- Implemented `default_registry` and `default_server_config` using object literals and JSON round-trip.
- **Gap:** Requires `Result<json, string>` return because parseJSON returns Result; this propagates error handling across call sites.
- **Gap:** No struct declarations; config is json with string keys instead of typed structs.

---

## Transport Port Notes
- Rewrote transport types to json-based objects due to missing enum/struct grammar support.
- Reader/writer now operate on json objects and string protocols.
- Implemented stdio transport as an in-memory buffer simulation (no real stdin/stdout streaming).
- **Gap:** Atlas stdlib does not expose raw stdin/stdout streaming or non-blocking reads; real transport I/O remains unimplemented.
- **Gap:** No robust byte buffers; implementation uses string buffers only.

---

## Supervisor Port Notes
- Replaced enum/struct usage with json-based state.
- Implemented synchronous exec + retry loop only.
- **Gap:** Go supervisor uses async monitoring, cancellation, restart windows, and concurrent wait; Atlas lacks channels/timers/async process APIs.
- **Gap:** Restart window logic not fully implemented (no time API in codebase for timers).

---

## Env Substitution Port Notes
- Implemented string substitution for ${env:VAR} using manual scanning and getEnv().
- **Gap:** No map/object key iteration on json; environment map substitution is not implemented.

---

## Config Loader Notes
- Implemented registry/server load/merge/validate using json objects.
- **Gap:** No .env loader equivalent to godotenv; env file loading omitted.
- **Gap:** Environment map merging is shallow; no key-by-key merge due to missing json map iteration.

---

## Proxy Queue Notes
- Implemented restart queue with json buffer.
- **Gap:** No time API; TTL expiration not implemented.
- **Gap:** Payload typed as string; original uses []byte.

---

## Proxy Core Notes
- Implemented synchronous proxy state transitions only.
- **Gap:** No channels/select/async read loops; actual routing between client/child not implemented.
- **Gap:** Transport interfaces are simulated via json; no real stdio streaming.

---

## Proxy Module Notes
- Implemented proxy core + queue + message handling + tool merge in simplified json-based form.
- **Gap:** Forwarding to actual transports is a no-op (no streaming APIs).
- **Gap:** Metrics/recording/state store integrations are no-op.
- **Gap:** Tool merge uses string search for set membership due to lack of set/map iteration.

---

## Metrics Notes
- Implemented simplified metrics collector without time/percentile logic.
- **Gap:** No time API; latency and uptime are manual milliseconds only.
- **Gap:** No percentiles or restart reasons tracking.

---

## Security Notes
- Implemented redaction using regexReplaceAll; invalid patterns not handled (regexNew returns Result).
- Rate limiter is token bucket without time-based refill (no time API).
- Size limiter truncates JSON by length only.

---

## Recorder Notes
- Implemented simplified recorder with in-memory buffer and export to JSON file.
- **Gap:** No redaction or body filtering; payload stored raw string.

---

## Watcher Notes
- Implemented watcher as synthetic event buffer only.
- **Gap:** No fsnotify or filesystem walking; real file change detection unavailable.

---

## State Manager Notes
- Implemented save/load/delete with JSON files.
- **Gap:** No PID/process liveness checks; cleanup is no-op.
- **Gap:** No timestamps (time API missing).

---

## Discovery Notes
- Implemented basic scan of explicit paths with JSON parsing.
- **Gap:** No OS-specific path discovery logic.

---

## Adaptive Notes
- Implemented minimal learner with history array.
- **Gap:** No scoring/analysis; persistence is raw JSON.

---

## Injectable Notes
- Implemented tool definitions and basic tool handlers.
- **Gap:** No actual supervisor hooks or log buffers; tools return placeholders.

---

## CLI Notes
- Replaced CLI command files with minimal `cmd_<name>(args)` stubs returning args.
- **Gap:** No argument parsing, file IO workflows, or actual command behavior implemented.

---

## Remaining Placeholder Modules
- supervisor/process*.atl, bootstrap/*, sanitizer/classifier.atl, statestore/store.atl, security/security.atl, mocks/* replaced with minimal stubs.
- **Gap:** These are not functional; require full port to meet parity.

---

## Statestore/Sanitizer/Security Updates
- Implemented statestore CRUD with JSON files.
- Implemented sanitizer classifier with JSON detection.
- Implemented security facade wrappers.

---

## 2026-03-03 Updates (Continue)
- Replaced bootstrap placeholders with simplified implementations:
  - `bootstrap/pathutil.atl` now expands `~`, detects client config paths via known locations.
  - `bootstrap/generic.atl` reads config, validates `mcpServers`, writes backup + preserves config.
  - Client wrappers (`claude`, `cline`, `cursor`, `claude-cli`, `continue`, `cursor-composer`) call generic.
- Replaced mocks/* placeholders with in-memory minimal behaviors (logger, rate limiter, redactor, sanitizer, size limiter, state store, supervisor, transport, watcher).
- Implemented `proxy/test_helpers.atl` minimal helpers.
- Implemented `state/cleanup.atl` best-effort cleanup: removes corrupt JSON state files.
- Implemented `supervisor/process_windows.atl` setup stub for parity with unix stub.

### Gaps (Still)
- Bootstrap cannot rewrite `mcpServers` entries due to lack of JSON object key iteration in Atlas.
- Registry updates are written, but MCP server entries are not injected.
- Mocks are functional but do not emulate timing/IO behaviors from Go tests.

---

## 2026-03-03 Updates (CLI + Watcher)
- Implemented CLI helpers and command functions (minimal, args-as-json):
  - `list`, `inspect`, `export`, `discover`, `init`, `run`, `status`, `ps`, `uninit`, `init_bootstrap`, `validate`.
  - `root.execute_args` dispatcher added; `execute()` now returns guidance error.
- CLI commands that require registry mutation still return explicit errors due to missing JSON object key insertion/removal.
- CLI commands for `logs`, `restart`, `recover`, `tune` return explicit not-implemented errors.
- Updated watcher and supervisor process comments to reflect simplified implementations.

### Gaps (Still)
- CLI cannot mutate registry (add/remove/tune) without JSON object key insertion or hashmap conversions.
- CLI lacks argument parsing (Cobra/Viper equivalent) and uses JSON args only.
- `run`/`status`/`ps` are synchronous, with no process supervision.
