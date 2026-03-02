# Pre-Systems-Level Hardening Roadmap

**Status:** IN PROGRESS (v0.3 feature work complete, hardening phase active)
**Target:** Functional, battle-tested language ready for systems-level conversion
**Audit Context:** See `advanced-codex-audit.md` + `docs/codex-findings/important-before-continuing.md`

---

## Priority: Core Language Must Work First

**Before systems-level conversion (Rust-like borrow checker, lifetime analysis, etc.), the current scripting-level language must be stable and complete.** Missing features and broken implementations block hardening.

---

## Critical Gaps (Audit Findings Summary)

### 1. Security/Sandbox (VISION-BREAKING)
- **Issue:** `RuntimeConfig` declares `allow_network`, `max_execution_time`, `max_memory_bytes` — **none are enforced**
- **Impact:** Sandbox is declared, not enforced. FFI bypasses policy entirely
- **Status:** ❌ MISSING ENFORCEMENT
- **Blocks:** AI-safe execution, production deployments

### 2. JIT Integration (DEAD CODE)
- **Issue:** JIT exists (`crates/atlas-jit/`) but is not wired to VM/runtime
- **Impact:** JIT is inert; no performance benefit
- **Status:** ⬜ Block 7 scaffolded (not started)
- **Blocks:** Performance claims, competitive benchmarks

### 3. Bytecode Serialization (INCOMPLETE)
- **Issue:** No stable artifact format — builds cannot be cached/distributed
- **Impact:** Every execution re-compiles from source
- **Status:** ❌ TODO stubs in `bytecode/mod.rs`
- **Blocks:** Build caching, deployment pipelines

### 4. LSP Navigation/Indexing (STUBBED)
- **Issue:** Go-to-definition, cross-file indexing incomplete
- **Impact:** IDE experience is degraded
- **Status:** ⚠️ Partial (hover works, navigation does not)
- **Blocks:** AI-first tooling claims

### 5. Error Discipline (NOT SYSTEMS-GRADE)
- **Issue:** High `unwrap()`/`panic!()` density in non-test code
- **Impact:** Runtime crashes on unexpected input
- **Status:** ⚠️ Mapped in audit, not fixed
- **Blocks:** Determinism, robustness

### 6. Parity Risks (COMPILER/VM DIVERGENCE)
- **Issue:** Side-effect heavy code paths not guaranteed identical
- **Impact:** Same code → different results in interpreter vs. VM
- **Status:** ⚠️ Partial coverage (pure functions tested, IO/async less so)
- **Blocks:** Correctness guarantees

### 7. Stdlib Gaps (FEATURE PARITY)
- **Missing:** WebSockets, advanced crypto, HTTP client, compression, process spawn, filesystem watching
- **Impact:** Atlas is not competitive with Node.js/Python at v0.3 scope
- **Status:** ❌ Planned but not implemented
- **Blocks:** Real-world adoption, demo viability

---

## Hardening Work Streams (Post-v0.3 Feature Freeze)

### Phase H1: Runtime Enforcement (2-3 blocks)
- [ ] Time/memory quota enforcement in VM loop
- [ ] Network permission gating (split IO vs. network)
- [ ] FFI permission checks (deny-by-default)
- [ ] Security policy integration tests

### Phase H2: Stdlib Completion (3-5 blocks)
- [ ] HTTP client (`http.get()`, `http.post()`)
- [ ] WebSockets (`ws.connect()`, `ws.send()`)
- [ ] Process spawn (`exec()`, `spawn()`)
- [ ] Compression (gzip, brotli)
- [ ] Filesystem watching (`fs.watch()`)
- [ ] Advanced crypto (beyond current hash/hmac)

### Phase H3: Error Hardening (1-2 blocks)
- [ ] Audit: `unwrap()`/`expect()` in `crates/atlas-runtime/src/` (non-test)
- [ ] Convert to `Result<T, E>` with proper error types
- [ ] Audit: `panic!()` in interpreter/VM/compiler
- [ ] Replace with graceful degradation or error propagation

### Phase H4: Infrastructure (1-2 blocks)
- [ ] Bytecode serialization format (versioned)
- [ ] Build artifact caching
- [ ] LSP go-to-definition + cross-file indexing
- [ ] LSP rename refactoring

### Phase H5: Battle Testing (ongoing)
- [ ] Fuzzing: parser, typechecker, compiler, VM (expand coverage)
- [ ] Stress tests: deep recursion, large arrays, long strings
- [ ] Real-world programs: web server, CLI tool, data processor
- [ ] Parity audit: async/await, IO, stdlib side effects

---

## Acceptance Criteria (Hardening Complete)

**The language is ready for systems-level conversion when:**

1. ✅ All v0.3 blocks complete (Blocks 1-9)
2. ✅ Security enforcement works (time/memory/network/FFI gated)
3. ✅ Stdlib feature parity with Node.js/Python for scripting use cases
4. ✅ Zero `unwrap()`/`panic!()` in runtime hot paths
5. ✅ Bytecode serialization stable and versioned
6. ✅ LSP navigation feature-complete
7. ✅ Parity tests cover async/IO/stdlib (100% identical output)
8. ✅ Real-world program suite (5+ non-trivial demos) runs successfully

**At that point:** Systems-level features (borrow checker, lifetimes, zero-cost abstractions) can be added without destabilizing the scripting-level foundation.

---

## Current State vs. Target

| Feature | v0.2 Close | v0.3 Now | Hardening Target |
|---------|-----------|----------|------------------|
| Memory model | Arc<Mutex> | CoW values ✅ | Same |
| Ownership syntax | ❌ | `own`/`borrow`/`shared` ✅ | Same |
| Traits | ❌ | `trait`/`impl` ✅ | Same |
| Closures | ❌ | Anonymous fns ✅ | Same |
| Type inference | Partial | Return types ✅ | Same |
| Error handling | Basic | `?` operator ✅ | Same |
| JIT | Exists (dead) | Exists (dead) | **Wired + working** |
| Sandbox | Declared | Declared | **Enforced** |
| Bytecode cache | ❌ | ❌ | **Serialized + cached** |
| LSP nav | ❌ | Hover only | **Full navigation** |
| Stdlib | ~300 fns | ~300 fns | **+50 fns (web/crypto/io)** |
| Error discipline | Lots of unwrap | Lots of unwrap | **Result-based** |
| Parity | Partial | Partial | **100% coverage** |

---

## References

- **Audit reports:** `advanced-codex-audit.md`, `docs/codex-findings/*.md`
- **Version plan:** `docs/internal/V03_PLAN.md` (Blocks 1-9 feature work)
- **Post-v0.3:** This file defines hardening → systems-level transition

---

## Notes for AI Agents

**DO NOT start systems-level work (borrow checker, lifetimes, manual memory, zero-cost abstractions) until hardening is complete.** The scripting-level language must be stable first.

**When v0.3 blocks finish:** Pause feature work. Execute hardening phases H1-H5. Only after hardening ACs met → begin systems-level conversion.

**Test cleanup, code hygiene, inline test migration:** Lower priority than language functionality. Defer until post-hardening.
