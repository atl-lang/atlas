# JIT Audit

Target: atlas-jit
Severity: High
Status: Open

## Finding 1: JIT not wired into VM execution path

Evidence:
- /Users/proxikal/dev/projects/atlas/crates/atlas-jit/src/CLAUDE.md:1-36
- /Users/proxikal/dev/projects/atlas/crates/atlas-jit/JIT_STATUS.md:1-40
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src (no JIT references)

What/Why:
- The JIT crate exists but no runtime/VM integration is present. VM execution never calls `JitEngine::notify_call`, so JIT is inert in production.

Impact:
- Performance roadmap for AI-first workflows (fast iteration + systems performance) is blocked.

Recommendation:
- Wire hotspot tracking into VM call dispatch and fall back to interpreter on `JitError::UnsupportedOpcode` as documented in `JIT_STATUS.md`.

---

## Finding 2: Parameterized functions are compiled/executed as zero-arg

Evidence:
- /Users/proxikal/dev/projects/atlas/crates/atlas-jit/src/codegen.rs:44-66
- /Users/proxikal/dev/projects/atlas/crates/atlas-jit/src/codegen.rs:68-109
- /Users/proxikal/dev/projects/atlas/crates/atlas-jit/src/lib.rs:173-186

What/Why:
- `IrTranslator::translate()` generates a `fn() -> f64` signature. `translate_with_params()` exists but is never used by `JitEngine::try_compile`.
- `try_compile` always calls `compiled.call_no_args()` and inserts `param_count = 0`.

Impact:
- Any hot function with parameters will be compiled/executed with the wrong ABI. This is undefined behavior and can crash or silently return incorrect results.

Recommendation:
- Thread parameter count into `try_compile` and use `translate_with_params()` + `call_1arg/call_2args/...` (or a generic ABI dispatch) based on `FunctionRef::arity`.

---

## Finding 3: Cache size accounting is effectively hard-coded

Evidence:
- /Users/proxikal/dev/projects/atlas/crates/atlas-jit/src/lib.rs:183-186
- /Users/proxikal/dev/projects/atlas/crates/atlas-jit/src/backend.rs:72-106

What/Why:
- `CodeCache::insert` receives a constant `64` for code size, and `CompiledFunction::code_size` is always `0` because Cranelift size is not tracked.

Impact:
- Cache eviction and size enforcement are inaccurate, risking unbounded memory use or premature evictions.

Recommendation:
- Track generated code size if Cranelift exposes it, or maintain a conservative estimate based on function body size. Remove hard-coded `64`.

---

## Finding 4: Doc/config mismatch for JIT threshold and value model

Evidence:
- /Users/proxikal/dev/projects/atlas/crates/atlas-jit/src/CLAUDE.md:23-26 (default 1000)
- /Users/proxikal/dev/projects/atlas/crates/atlas-jit/src/lib.rs:68-75 (default 100)
- /Users/proxikal/dev/projects/atlas/crates/atlas-jit/JIT_STATUS.md:67-69 (Arc<Mutex<Vec<Value>>>)

What/Why:
- JIT documentation claims a default threshold of 1000 calls, but code defaults to 100. `JIT_STATUS.md` also references the old `Arc<Mutex<Vec<Value>>>` model, which is no longer true.

Impact:
- AI agents following docs will set the wrong expectations and may make incorrect integration choices.

Recommendation:
- Align the docs with the actual defaults and the current CoW memory model.
