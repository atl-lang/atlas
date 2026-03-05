# Runtime Security Gaps

Target: atlas-runtime security + runtime API
Severity: High
Status: Open

## Finding 1: RuntimeConfig ignores `allow_network` and does not enforce time/memory limits

Evidence:
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/api/runtime.rs:160-181
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/api/config.rs:33-60

What/Why:
- `RuntimeConfig` exposes `allow_network`, `max_execution_time`, and `max_memory_bytes`, but `Runtime::with_config` only toggles `allow_io` and then calls `SecurityContext::allow_all()` or deny-all. This means `allow_network` is effectively ignored and all network operations are permitted whenever IO is allowed.
- The time/memory limits are explicitly noted but not enforced. This undermines the sandbox guarantees expected for AI-agent execution.

Impact:
- Sandboxed configs can still make network requests if `allow_io` is true, which violates the configuration contract.
- Untrusted code can run indefinitely or allocate beyond configured limits.

Recommendation:
- Split `allow_io` and `allow_network` into distinct SecurityContext permissions (filesystem vs. network) and enforce both.
- Wire `max_execution_time` and `max_memory_bytes` into interpreter/VM execution loop or the `security::sandbox` quota system.

---

## Finding 2: Sandbox system exists but is not integrated into runtime execution

Evidence:
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/security/sandbox.rs:126-377
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/api/runtime.rs:173-191

What/Why:
- `security::sandbox` defines quotas and enforcement hooks but is never used by `Runtime` or the interpreter/VM execution paths. The runtime config claims sandboxing but only configures permissions.

Impact:
- Resource quota enforcement is effectively a no-op, despite API promises.

Recommendation:
- Instantiate a `Sandbox` in `Runtime::with_config` and thread it through VM/interpreter execution, checking CPU time, memory, and other quotas at appropriate points (function call, allocation, I/O).

---

## Finding 3: Policy rules and resource types are partially enforced

Evidence:
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/security/policy.rs:262-283
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/security/policy.rs:306-323

What/Why:
- `ResourceType::FFI`, `FileDelete`, `NetworkListen`, `Reflection` are defined but not mapped to `Permission` in `rule_to_permission`. Policies referencing these are silently ignored.
- Wildcard patterns are allowed in policy validation but `rule_to_permission` passes raw `pattern` into `PathBuf`. This means wildcard-based filesystem rules are not translated into effective permissions.

Impact:
- Security policies can appear to allow/deny resources but have no effect in enforcement, creating false safety.

Recommendation:
- Expand `Permission` to cover all `ResourceType` variants, then handle wildcard patterns explicitly. If wildcard support is not intended, reject such patterns during validation.

---

## Finding 4: FFI calls bypass security checks

Evidence:
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/interpreter/expr.rs:300-316
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/vm/mod.rs:975-995

What/Why:
- Extern function calls invoke `unsafe { extern_fn.call(&args) }` directly with no permission checks or sandbox quota checks.

Impact:
- Untrusted Atlas code can execute arbitrary FFI calls even when sandboxed, undermining security guarantees and AI-agent safety.

Recommendation:
- Add explicit FFI permission checks in both interpreter and VM paths. Block by default unless enabled via security policy/runtime config.
