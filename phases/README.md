# Atlas Phases

This folder is organized by long-lived sections. Each section has its own phase numbering and an `archive/` for older phase iterations.

Sections:
- `research/` (design references and constraints)
- `foundation/` (project setup and scaffolding)
- `frontend/` (lexer + parser)
- `typing/` (binder + type checker + diagnostics)
- `interpreter/` (interpreter + REPL)
- `bytecode-vm/` (bytecode compiler + VM)
- `stdlib/` (standard library)
- `cli/` (command-line tooling)
- `polish/` (stabilization and packaging)

Phase naming convention:
- `phase-01-*.md`, `phase-02-*.md`, etc.
- Each section increments independently.

Build order:
- `phases/BUILD-ORDER.md`
