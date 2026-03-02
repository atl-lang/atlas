# Language Classification (Archived 2026-02)

Strategic/vision content moved from active memory. Reference when planning v1.0+.

---

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

**AI Agent Responsibility:** When the time comes to decide (v1.0+ planning), research which approach best fits Atlas's goals.

## End Goal

**Atlas will become a world-class systems language rivaling Rust, Go, C, Python.**

- No MVP mindset — done properly from the start
- Native codegen via LLVM/Cranelift (v1.0+)
- Memory model decision made by AI research, not user input
- All current work builds toward this goal
