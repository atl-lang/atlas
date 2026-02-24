# Phase 08: LSP Hover — Inferred Types

**Block:** 5 (Type Inference)
**Depends on:** Phase 03 + Phase 04 complete

## Goal

LSP hover shows inferred types for:
1. Local variables with no annotation: `let x = 42` → hover shows `let x: number`
2. Functions with no return type annotation: `fn double(x: number)` → hover shows `fn double(x: number) -> number`

## Current state

`hover.rs` `find_variable_hover` already reads the symbol table type. Since the typechecker
now fills in inferred types (Phase 03/04), hover should already show correct types in most
cases. This phase verifies and adds the missing annotation rendering.

## Format

For inferred types (no explicit annotation), render identically to explicit — no "(inferred)"
suffix or special marking. The user sees the resolved type; whether it was inferred or
annotated is irrelevant at the hover level.

**Exception:** If type is still `Unknown` after inference, show nothing (no type annotation
in hover output) rather than showing `unknown`.

## Inlay hints

`inlay_hints.rs` currently shows parameter types. For Phase 08, also add:
- Return type inlay hint on functions with omitted return type: `fn double(x: number)` → `→ number` inlay at end of signature

This is behind the existing inlay hint config (`InlayHintConfig`). Add `show_inferred_return: bool` flag defaulting to `true`.

## Acceptance Criteria

- [ ] `let x = 42` hover shows `let x: number` (inferred, no annotation)
- [ ] `fn f(x: number) { return x * 2; }` hover shows return type `number`
- [ ] `Unknown` inferred types do NOT show a type annotation in hover
- [ ] Inlay hint appears at end of unannotated function signature showing `→ T`
- [ ] Minimum 4 new LSP hover tests in `tests/lsp_hover_tests.rs`
- [ ] Minimum 2 new LSP inlay hint tests in `tests/lsp_inlay_tests.rs`
