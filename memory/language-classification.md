# Atlas Language Classification & Evolution Path

## Current Classification

**Application Language** (Go-tier, not yet Rust-tier)

| Aspect | Current State |
|--------|---------------|
| Execution | Interpreter + Bytecode VM |
| Memory | Reference counting (`Arc<Mutex<>>`) |
| Output | Requires Atlas runtime |
| Tier | Python/Go level |

## Execution Modes

- **Interpreter**: Tree-walking, for REPL/debugging/dev
- **Bytecode VM**: Faster execution, production runs
- **Native (future)**: LLVM/Cranelift backend

## Evolution Path to Systems-Level

```
Current:                Future (additive):

AST ──► Interpreter     AST ──► Interpreter (dev/REPL)
    │                       │
    └──► Bytecode VM        ├──► Bytecode VM (fast iteration)
                            │
                            └──► LLVM Backend (native binaries)
```

**Key insight**: Frontend (lexer, parser, AST, types, binder) is backend-agnostic. Native codegen is ADDED, not replaced.

## What's Reusable for Systems-Level (100%)

- Lexer/Parser
- AST representation
- Type system
- Binder/Resolver
- Tooling (LSP, CLI, Formatter)
- Test infrastructure
- Bytecode compiler (as intermediate representation)

## Future Additions for Systems-Level

| Component | Purpose | When |
|-----------|---------|------|
| LLVM/Cranelift backend | Native code generation | v1.0+ |
| Stack allocation | Unboxed primitives | With native backend |
| Ownership model | Rust-level memory safety | Design decision TBD |

## Memory Model Decision (CRITICAL FUTURE WORK)

**Current:** `Arc<Mutex<T>>` — Reference counting, GC-like, application-level

**For systems-level, ONE of these paths:**

| Path | Example | Pros | Cons |
|------|---------|------|------|
| **Ownership/Borrowing** | Rust | Zero-cost, no GC, maximum perf | Complex, steep learning curve |
| **Tracing GC** | Go, Java | Simple mental model, productive | Runtime overhead, pauses |
| **Hybrid** | Swift (ARC + unsafe) | Best of both, escape hatch | Complexity in two modes |

**AI Agent Responsibility:** When the time comes to decide (v1.0+ planning), research which approach best fits Atlas's goals. Analyze:
- Rust's ownership (borrow checker complexity vs zero-cost)
- Go's GC (simplicity vs latency)
- Swift's ARC (middle ground)
- Zig's manual + safety (explicit but teachable)

**Do NOT ask the user.** Research, analyze, propose with reasoning. Document decision in `decisions.md` as DR-XXX.

---

## End Goal (NON-NEGOTIABLE)

**Atlas will become a world-class systems language rivaling Rust, Go, C, Python.**

- No MVP mindset — done properly from the start
- Native codegen via LLVM/Cranelift (v1.0+)
- Memory model decision made by AI research, not user input
- All current work builds toward this goal
- Every phase should consider: "Does this block or enable systems-level future?"

---

## For AI Agents (ALWAYS REMEMBER)

1. **End goal = Systems-level** (Rust/C tier, native binaries)
2. **Current tier = Application** (Go/Python tier, that's OK for now)
3. **Frontend = 100% reusable** (no work wasted)
4. **Memory model = Future decision** (AI researches and decides)
5. **Interpreter/VM = Keep forever** (dev tools alongside native backend)
6. **Quality over speed** — No shortcuts, no hacks, compiler standards
