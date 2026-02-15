# Documentation Reference Map

**For AI Agents:** Use the routing system to find docs efficiently. **DO NOT read all specs.**

---

## Navigation

- **Start here:** `Atlas-SPEC.md` - Index with routing table for AI agents
- **Use routing table** to find exactly which spec file you need

---

## Core Specifications (Lazy Load)

- **Types:** `docs/specification/types.md` - Type system, generics, patterns, JSON type
- **Syntax:** `docs/specification/syntax.md` - Grammar, keywords, operators, EBNF
- **Semantics:** `docs/specification/language-semantics.md` - Evaluation rules, edge cases
- **Runtime:** `docs/specification/runtime.md` - Execution model, memory, scoping
- **Modules:** `docs/specification/modules.md` - Import/export, resolution
- **REPL:** `docs/specification/repl.md` - Interactive mode behavior
- **Bytecode:** `docs/specification/bytecode.md` - VM, compilation, instructions
- **Diagnostics:** `docs/specification/diagnostics.md` - Error codes, formats
- **Grammar Rules:** `docs/specification/grammar-conformance.md` - Parser conformance

---

## API References

- **Standard Library:** `docs/api/stdlib.md` - Function signatures, examples, errors
- **Runtime API:** `docs/api/runtime-api.md` - Embedding API and runtime hooks

---

## Implementation Guides

- **All Implementation:** `docs/implementation/` directory
  - `01-project-structure.md` - Codebase organization
  - `02-core-types.md` through `16-lsp.md` - Component-specific implementation guides
  - `13-stdlib.md` - HOW to implement stdlib (not API reference)

---

## Error & Diagnostic System

- **Diagnostic System:** `docs/specification/diagnostic-system.md` - Error codes, warning codes, diagnostic format, normalization, ordering
- **Parser Recovery:** `docs/reference/parser-recovery-policy.md` - Error recovery strategy

---

## Testing & Quality

- **Testing Guide:** `docs/guides/testing-guide.md` - Test infrastructure, rstest, insta, proptest, interpreter/VM parity requirements
- **Code Quality:** `docs/guides/code-quality-standards.md` - Code style, engineering standards, phase gates

---

## JSON Formats

- **JSON Dumps:** `docs/specification/json-formats.md` - AST dumps, typecheck dumps, debug info format, stability guarantees

---

## Other References

- **Config:** `docs/config/` - CLI config, REPL modes
- **Reference:** `docs/reference/` - Code organization, versioning, decision log, security model
- **Philosophy:** `docs/philosophy/` - AI manifesto, documentation philosophy, project principles

---

## Feature Documentation

**Phases create feature docs in `docs/features/` as they implement new capabilities.** Phase files specify which docs to create/update.
