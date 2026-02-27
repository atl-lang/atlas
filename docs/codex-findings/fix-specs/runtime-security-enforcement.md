# Spec: Runtime Security Enforcement

Target: atlas-runtime
Owner: Codex audit
Status: Draft

## Goal
Make runtime sandbox and security configuration authoritative and enforced. Ensure `RuntimeConfig` flags (IO/network/time/memory) are honored, and FFI is permission-gated.

## Problems to Solve
- `RuntimeConfig.allow_network` is ignored; IO toggle grants all permissions.
- `max_execution_time` and `max_memory_bytes` are documented but not enforced.
- FFI calls bypass permissions entirely.
- Sandbox quotas exist but are not integrated with runtime execution.

## Scope
In scope:
- Runtime config application in `Runtime::with_config`
- Permission checks for FFI calls (interpreter + VM)
- Wiring sandbox quota checks into execution paths

Out of scope:
- Full policy file discovery and automatic policy resolution
- OS-level sandboxing

## Design
### 1. SecurityContext construction
- Split `allow_io` and `allow_network` to separate permission grants.
- Default deny-all when flags are false.
- Preserve existing granular checks in stdlib (`io`, `http`, `process`, `env`).

### 2. Sandbox integration
- Instantiate `Sandbox` when config specifies limits (execution time or memory).
- Thread an optional sandbox reference through interpreter and VM execution loops.
- Enforce:
  - CPU time: check at function call boundaries and loop backedges (Jump/Loop opcodes).
  - Memory: check on collection growth (array push, hashmap insert, string concat where new allocations occur).

### 3. FFI permission checks
- Introduce `Permission::FFI` or equivalent in `SecurityContext` and wire to policy mapping.
- Require FFI permission before `extern_fn.call` in interpreter and VM.

## Acceptance Criteria
- `RuntimeConfig.allow_network = false` blocks HTTP access while still allowing IO when `allow_io = true`.
- FFI calls fail with a permission error in sandboxed mode.
- Time and memory limits are enforced and tested with a deterministic failure.
- No public API behavior contradicts documentation.

## Test Plan
- Add runtime tests for network deny/allow split.
- Add FFI permission tests (interpreter + VM).
- Add sandbox timeout test (tight time budget, infinite loop).
- Add sandbox memory test (tight memory budget, growth loop).

## Files Likely Touched
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/api/runtime.rs
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/security/permissions.rs
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/security/sandbox.rs
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/interpreter/expr.rs
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/vm/mod.rs
