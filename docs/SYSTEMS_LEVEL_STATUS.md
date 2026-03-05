# Systems-Level Conversion Status

Atlas is converting from "AI experiment" architecture to proper systems-level compiler design.
This document tracks what's done, what's in progress, and what's legacy.

**Current Phase:** Battle-testing and hardening (paused mid-conversion)

---

## Architecture Layers

| Layer | Status | Notes |
|-------|--------|-------|
| Lexer | Legacy | Works, but not systems-level |
| Parser | Partial | Core syntax works, some AST nodes not wired |
| Type System | Partial | Basic inference works, advanced features pending |
| Interpreter | Legacy | Functional but architecture debt |
| VM | Partial | Exists, parity with interpreter required |
| JIT | Scaffolded | Infrastructure exists, not production-ready |
| LSP | Scaffolded | Basic functionality, needs hardening |

---

## Feature Implementation Status

### Fully Implemented (Parser + Interpreter + VM)
- Variables (`let`, `let mut`)
- Functions (`fn`)
- Structs and field access
- Arrays and indexing
- Control flow (`if`, `while`, `for`)
- Match expressions
- Result/Option types
- Compound assignment (`+=`, `-=`, etc.)
- Template strings

### Partial Implementation (IN PROGRESS - DO NOT DELETE)
These have AST definitions but incomplete parser/runtime support.
They are **intentional scaffolding** for systems-level conversion.

| Feature | AST | Parser | Interpreter | Notes |
|---------|-----|--------|-------------|-------|
| Traits | Yes | No | No | Systems-level type system prep |
| Impl blocks | Yes | No | No | Systems-level type system prep |
| Ranges | Yes | Partial | Partial | Syntax parses, runtime varies |

### Removed Features (H-034 and related)
These were intentionally removed. Artifacts may exist but should be cleaned up.

| Feature | Removal Issue | Reason |
|---------|---------------|--------|
| `++`/`--` operators | H-034 | C-style, not AI-friendly |
| C-style `for` loops | H-034 | Replaced with `for..in` |

---

## Conversion Blocks (from planning)

| Block | Theme | Status |
|-------|-------|--------|
| B1-B5 | Core runtime | Complete |
| B6 | Hardening | Complete |
| B7 | Types/Inference | Pending (last systems-level block) |
| B8-B9 | TBD | Pending |

---

## How to Update This Document

When fixing bugs or implementing features:
1. If converting legacy code to systems-level → update the layer status
2. If wiring up a partial implementation → move it to "Fully Implemented"
3. If removing dead code from a removed feature → note the cleanup

This document is for AI continuity. Keep it accurate.
