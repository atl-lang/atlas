# Spec: JIT ABI Correctness and VM Integration

Target: atlas-jit + atlas-runtime VM
Owner: Codex audit
Status: Draft

## Goals
- Ensure JIT executes parameterized functions with correct ABI.
- Wire JIT into VM execution path safely.
- Align cache size accounting with actual compiled code usage.

## Problems to Solve
- `JitEngine::try_compile` always compiles/executes as `fn() -> f64`.
- JIT not wired into VM, so it is inactive in production.
- Cache size accounting uses a hard-coded constant.

## Scope
In scope:
- JIT parameter handling
- VM hotspot integration
- Cache size tracking

Out of scope:
- Full opcode coverage (still Block 7)

## Design
### 1. Parameter ABI
- Thread `param_count` from runtime `FunctionRef` into `JitEngine::notify_call` / `try_compile`.
- Use `translate_with_params` when `param_count > 0`.
- Dispatch to `call_1arg` / `call_2args` / vararg trampoline as needed.

### 2. VM integration
- Add optional `JitEngine` to VM state.
- At function call dispatch, if JIT enabled and function is hot, call `notify_call`.
- On `JitError::UnsupportedOpcode`, fall back to interpreter.

### 3. Cache size
- Track code size: either from Cranelift (if exposed) or estimate based on IR block count / bytecode length.
- Replace hard-coded `64` with computed size.

## Acceptance Criteria
- Parameterized functions return correct results under JIT.
- VM can be configured to enable/disable JIT.
- Cache limit is enforced with meaningful accounting.

## Test Plan
- Add JIT tests for 1- and 2-arg functions using `notify_call`.
- Add VM integration test with hot numeric function (JIT enabled) to confirm native execution path.

## Files Likely Touched
- /Users/proxikal/dev/projects/atlas/crates/atlas-jit/src/lib.rs
- /Users/proxikal/dev/projects/atlas/crates/atlas-jit/src/codegen.rs
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/vm/mod.rs
- /Users/proxikal/dev/projects/atlas/crates/atlas-jit/src/cache.rs
