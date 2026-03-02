# Systems-Level Audit Handoff (2026-03-02)

## What Was Done This Session

Branch: `feat/systems-stdlib-gaps` (from main, commit 63723bc)

### 60 new stdlib functions added across 5 modules:

| Module | Functions | File |
|--------|-----------|------|
| **Crypto** | sha256, sha512, blake3Hash, hmacSha256, hmacSha256Verify, aesGcmEncrypt, aesGcmDecrypt, aesGcmGenerateKey | `stdlib/crypto.rs` |
| **Encoding** | base64Encode/Decode, base64UrlEncode/Decode, hexEncode/Decode, urlEncode/Decode | `stdlib/encoding.rs` |
| **Networking** | TCP client (connect/write/read/readBytes/close/setTimeout/setNodelay/localAddr/remoteAddr), TCP server (listen/accept/listenerAddr/listenerClose), UDP (bind/send/receive/setTimeout/close/localAddr), TLS (connect/write/read/close) | `stdlib/net.rs` |
| **Sync** | RwLock (new/read/write/tryRead/tryWrite), Semaphore (new/acquire/tryAcquire/release/available), Atomic (new/load/store/add/sub/compareExchange) | `stdlib/sync.rs` |
| **WebSocket** | wsConnect, wsSend, wsSendBinary, wsReceive, wsPing, wsClose | `stdlib/websocket.rs` |

### Quality gates passed:
- 8,248 tests: 0 failures
- Clippy: 0 warnings (-D warnings)
- All modules follow existing stdlib patterns
- Security context enforced on all network operations

### Handle pattern used:
Network/sync handles use array-tagged IDs: `[tag_string, id_number]`
- Stored in global `OnceLock<Mutex<HashMap<u64, Arc<Mutex<Resource>>>>>`
- Thread-safe, works in async contexts
- No new Value variants added (avoids blast radius)

### Dependencies added to atlas-runtime/Cargo.toml:
- sha2, hmac, aes-gcm, blake3, hex (crypto)
- rustls, webpki-roots, tungstenite (networking/websocket)
- base64 + urlencoding were already present

---

## What Remains (Priority Order)

### P1 — CRITICAL (Compiler Surgery Required)

#### 1. Struct Types (Named Product Types)
**What:** `struct Person { name: string, age: number }` with methods via impl blocks.
**Why:** Every systems language has structs. Without nominal types, domain modeling impossible.
**Touches:** Parser (new StructDecl AST node), typechecker (nominal type resolution, method dispatch), interpreter (struct instantiation, field access), VM (same), compiler (bytecode emission for struct ops), LSP (completions for struct fields/methods).
**Difficulty:** HIGH — 4-6 phases of work minimum.
**Reference:** Trait system (Block 3) already has `TraitDecl` + `ImplBlock` in AST. Structs follow same pattern but with data fields.

#### 2. Enum / Algebraic Data Types
**What:** `enum Shape { Circle(number), Rect(number, number) }` with exhaustive pattern matching.
**Why:** State machines, error hierarchies, protocol modeling. Rust's killer feature.
**Touches:** Same files as structs + exhaustiveness checker needs enum variant awareness.
**Difficulty:** HIGH — can share infrastructure with structs. 3-5 phases.
**Depends on:** Structs should land first (shared infrastructure).

#### 3. `?` Operator (Error Propagation)
**What:** `let x = try_thing()?;` — early return on Err/None.
**Why:** Without it, every fallible call needs 5+ lines of match boilerplate.
**Touches:** Parser (postfix `?` operator), interpreter (check Result/Option, early return), VM (same), compiler (emit check + jump).
**Difficulty:** MEDIUM — 2-3 phases. Already planned as v0.3 Block 6.
**Status:** Scaffolded in V03_PLAN.md, ready to execute.

### P2 — MAJOR

#### 4. JIT Integration with VM
**What:** Wire Cranelift JIT to VM execution loop. Currently arithmetic-only, needs control flow + function calls.
**Status:** v0.3 Block 7. Foundation exists in `crates/atlas-jit/`.
**Touches:** vm/mod.rs (hotspot detection → JIT compilation → native execution), atlas-jit/src/codegen.rs (add Jump/JumpIfFalse/Loop/Call opcodes).
**Difficulty:** MEDIUM-HIGH — 3-4 phases.

#### 5. Structured Concurrency
**What:** Task groups with cancellation scopes. `taskGroup { spawn(...); spawn(...); }` — if one fails, all cancel.
**Touches:** async_runtime/ (add TaskGroup, CancellationScope), stdlib registration.
**Difficulty:** MEDIUM — 2 phases.
**Reference:** Go's errgroup, Python's TaskGroup, Kotlin's coroutineScope.

#### 6. Backpressure on Channels
**What:** Bounded channels that block/error when full.
**Touches:** async_runtime/channel.rs — add blocking send on bounded channels.
**Difficulty:** LOW — 1 phase.

### P3 — MODERATE

| Item | Difficulty | Notes |
|------|-----------|-------|
| BTreeMap/BTreeSet | LOW | Wrap Rust std, add to collections/ |
| PriorityQueue/BinaryHeap | LOW | Wrap Rust std |
| String interpolation | MEDIUM | Parser + both engines |
| DNS resolution | LOW | Wrap std::net::ToSocketAddrs more explicitly |
| YAML/TOML parsing | LOW | serde_yaml + toml crates (toml already dep) |
| Test coverage metrics | LOW | Add cargo-tarpaulin to CI |
| Performance regression CI | LOW | Gate benchmarks in CI |

---

## Standards for Next Agent

1. **No stubs, no TODOs, no partial implementations.** Complete or don't start.
2. **Both engines must be updated in lockstep.** Interpreter + VM parity is sacred.
3. **All tests must pass before commit.** Run `cargo test --workspace`.
4. **Clippy must pass with `-D warnings`.** Zero warnings.
5. **Follow existing patterns.** Read existing code before writing new code.
6. **Security context must be enforced** on any operation that touches network/filesystem.
7. **Handle pattern:** Use array-tagged IDs `[tag, id]` for opaque resource handles (not new Value variants).
8. **Struct/Enum implementation** should follow the trait system pattern in ast.rs — `TraitDecl` and `ImplBlock` are the reference implementations.
9. **Read `.claude/rules/atlas-parity.md`** before touching interpreter or VM.
10. **Read `.claude/rules/atlas-ast.md`** before modifying AST nodes.

---

## File Locations (Quick Reference)

| What | Where |
|------|-------|
| AST nodes | `crates/atlas-runtime/src/ast.rs` |
| Value enum | `crates/atlas-runtime/src/value.rs` |
| Parser | `crates/atlas-runtime/src/parser/{mod,expr,stmt}.rs` |
| Type checker | `crates/atlas-runtime/src/typechecker/` |
| Interpreter | `crates/atlas-runtime/src/interpreter/{mod,expr,stmt}.rs` |
| VM | `crates/atlas-runtime/src/vm/mod.rs` |
| Bytecode compiler | `crates/atlas-runtime/src/compiler/{mod,expr,stmt}.rs` |
| Opcodes | `crates/atlas-runtime/src/bytecode/opcode.rs` |
| Stdlib registry | `crates/atlas-runtime/src/stdlib/mod.rs` |
| JIT | `crates/atlas-jit/src/` |
| LSP | `crates/atlas-lsp/src/` |
| Tests | `crates/atlas-runtime/tests/` |
| Spec | `docs/specification/` |
| Plan | `docs/internal/V03_PLAN.md` |
