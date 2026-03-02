# v0.3 Hardening Roadmap (Post-v0.2 Stabilization)

**Status:** IN PROGRESS (v0.3 Blocks 1-6 complete, hardening ongoing)
**Version Context:** v0.3 = language completeness + hardening | v0.4+ = systems-level (borrow checker, AOT)
**Target:** Functional, battle-tested foundation before compile-time ownership verification (v0.4)
**Audit Context:** See `advanced-codex-audit.md` + `docs/codex-findings/important-before-continuing.md`

---

## Priority: v0.3 Foundation Must Be Stable First

**v0.3 is not just features** — it's the architectural foundation for all future versions. Before v0.4 systems-level work (compile-time borrow checking, AOT compilation), the v0.3 scripting-level language must be stable and complete. Missing features and broken implementations block the transition.

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

## Hardening Work Streams (v0.3 Completion)

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

## Acceptance Criteria (v0.3 Complete, Ready for v0.4)

**v0.3 is complete and ready for v0.4 systems-level work when:**

1. ✅ All v0.3 blocks complete (Blocks 1-9 + any additional hardening blocks)
2. ✅ Security enforcement works (time/memory/network/FFI gated)
3. ✅ Stdlib feature parity with Node.js/Python for scripting use cases
4. ✅ Zero `unwrap()`/`panic!()` in runtime hot paths
5. ✅ Bytecode serialization stable and versioned
6. ✅ LSP navigation feature-complete
7. ✅ Parity tests cover async/IO/stdlib (100% identical output)
8. ✅ Real-world program suite (5+ non-trivial demos) runs successfully

**Then tag v0.3.0 and begin v0.4:** Compile-time ownership verification, borrow checker, AOT compilation, zero-cost abstractions.

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
- **Version plan:** `docs/internal/V03_PLAN.md` (v0.3 blocks + exit criteria)
- **Roadmap:** `ROADMAP.md` (v0.2 complete → v0.3 in progress → v0.4 systems-level)

---

## Notes for AI Agents

**v0.3 = foundation + hardening.** v0.4+ = systems-level (borrow checker, AOT compilation).

**DO NOT start v0.4 work until v0.3 hardening is complete.** The scripting-level foundation must be stable before adding compile-time ownership verification.

**When v0.3 feature blocks (1-9) finish:** Execute hardening phases H1-H5. Only after all v0.3 acceptance criteria met → tag v0.3.0 → begin v0.4.

**Test cleanup, code hygiene, inline test migration:** Lower priority than language functionality. Defer until v0.3 hardening complete.
