# LSP Decisions Archive — 2026-02

## DR-LSP-001: Refactoring Module Structure

Split refactoring into: `refactor/mod.rs`, `extract.rs`, `inline.rs`, `rename.rs`.
Status: Implemented, 40+ tests.

## DR-LSP-002: Basic Find References for Rename

Basic find_references in navigation.rs (AST-based, Range::default() placeholders).
Phase 05 will add proper span tracking. Status: Implemented.

## DR-LSP-003: WorkspaceEdit Structure

Use LSP WorkspaceEdit with `HashMap<Url, Vec<TextEdit>>`. Sort edits last-to-first.
Status: Implemented.

## DR-LSP-004: Name Generation Algorithm

Counter-based suffix: try base name, then base_1, base_2... Status: Implemented.

## DR-LSP-005: AST-to-Source Conversion Limitation

Use `format!("{:?}", var_decl.init)` as placeholder. Future: integrate atlas-formatter.
Status: Temporary.

## DR-LSP-006: Scope Analysis Limitation

Naive approach: extract at line start, rename all occurrences (no shadowing).
Future: CFG-based scope analysis. Status: Basic implementation.
