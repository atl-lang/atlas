# LSP Gaps Impacting AI-First Tooling

Target: atlas-lsp
Severity: Medium
Status: Open

## Finding 1: Go-to-definition is stubbed

Evidence:
- /Users/proxikal/dev/projects/atlas/crates/atlas-lsp/src/server.rs:293-311
- /Users/proxikal/dev/projects/atlas/crates/atlas-lsp/src/navigation.rs:109-118

What/Why:
- `goto_definition` always returns `None` because symbol position data is not wired through. `find_definition` is a stub.

Impact:
- AI tools and editors cannot resolve definitions reliably, reducing AI-first productivity.

Recommendation:
- Use spans from AST/symbols to implement location mapping and return `GotoDefinitionResponse`.

---

## Finding 2: Symbol locations and reference ranges are defaults

Evidence:
- /Users/proxikal/dev/projects/atlas/crates/atlas-lsp/src/navigation.rs:35-88

What/Why:
- Document symbols and references use `Range::default()` due to missing span-to-range conversion.

Impact:
- UI features (symbols, references, outline) are effectively non-functional or misleading.

Recommendation:
- Implement span-to-range conversion in `convert.rs` and apply real ranges throughout symbol and reference builders.

---

## Finding 3: Cross-file indexing is not implemented

Evidence:
- /Users/proxikal/dev/projects/atlas/crates/atlas-lsp/src/index.rs:134-197

What/Why:
- Imports/exports are ignored, and position-based lookups are TODO.

Impact:
- Workspace-wide symbol search, rename, and references are incomplete, limiting AI agent navigation across modules.

Recommendation:
- Index import/export tables and resolve to module paths; add position-aware lookups.

---

## Finding 4: Refactor-extract incomplete (capture + return type)

Evidence:
- /Users/proxikal/dev/projects/atlas/crates/atlas-lsp/src/refactor/extract.rs:91-92

What/Why:
- Extract refactor does not analyze captured variables or infer return types.

Impact:
- Generated refactors are incorrect or non-compiling in realistic code, degrading AI-assisted refactoring.

Recommendation:
- Implement capture analysis from AST + typechecker and derive return type from extracted block.
