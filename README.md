<div align="center">

# Atlas

### The AI-First Systems Language

**Designed for AI code generation. Built entirely by AI. Built to go systems-level.**

[![CI](https://github.com/proxikal/atlas/actions/workflows/ci.yml/badge.svg)](https://github.com/proxikal/atlas/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[\![Status](https://img.shields.io/badge/Status-v0.3%20Active-blue.svg)](docs/README.md)

</div>

---

## Why Atlas?

**Other programming languages were built before AI existed.** They're being retrofitted with AI tooling as an afterthought.

**Atlas is different.** Every design decision asks: *"What's best for AI?"*

```mermaid
graph LR
    subgraph Traditional Languages
        A[Human writes code] --> B[Compiler]
        B --> C[AI tools added later]
    end

    subgraph Atlas
        D[AI generates code] --> E[AI-optimized compiler]
        E --> F[AI-native from day one]
    end

    style Atlas fill:#e8f5e9
```

**The result:** A language that AI agents can generate, analyze, and debug with unprecedented reliability — and one that is being built to scale all the way to systems programming.

### The Long Game

Atlas starts where other languages wish they'd started: with AI as a first-class consumer of the language, and a memory model designed for systems-level work from day one.

- **No garbage collector.** Deterministic allocation. Value semantics by default.
- **No hidden ownership rules.** Ownership is explicit in syntax — AI can read it, write it, verify it.
- **No retrofit.** The foundation is correct before the features are built. Unlike Go (chose GC early, never went systems), unlike Swift (retrofitting ownership into ARC), Atlas gets the memory model right in v0.3 while it's still young.