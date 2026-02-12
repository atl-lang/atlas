# Engineering Standards (Atlas)

## Architecture Boundaries
- `atlas-runtime` is library-first: no CLI logic.
- `atlas-cli` is a thin wrapper around runtime APIs.
- Frontend (lexer/parser/binder/typechecker) is separate from runtime (interpreter/VM).
- Single `Value` representation is shared across interpreter and VM.

## AI-First Anchor
- AI-first principles are defined in `docs/ai-principles.md`.

## Error Handling
- All errors must flow through the `Diagnostic` type (single pipeline).
- Every diagnostic must include span, code, and message.
- Diagnostics must support human and JSON formats.
- Diagnostics should emit related spans for root-cause context.

## Testing Strategy
- Unit tests for lexer/parser/typechecker/interpreter.
- Golden tests for end-to-end behavior (input -> output).
- Keep snapshots small and deterministic.
- Avoid flaky tests (no time-based assertions).

## REPL Architecture
- REPL core is UI-agnostic and shared by all frontends.
- REPL UI is a thin layer that only handles input/output.
- Default v0.1 frontend uses a line editor; TUI is optional later.

## REPL / TUI Libraries
- If we need a full-screen TUI for REPL UX, prefer `ratatui`.
- If we only need line editing, prefer a line editor library.
- Preferred line editors: `rustyline` for maturity and broad adoption.
- Preferred line editors: `reedline` for modern features (syntax highlighting, completions, multiline).

## Security & Safety
- No `unsafe` in v0.1 unless explicitly approved.
- Use `cargo audit` before release.
- Keep dependencies minimal and vetted.

## Code Organization
- No god files; prefer < 400 lines per module unless justified.
- All AST types in `docs/ast.md` must map 1:1 to code.

## Rust Standards
- Stable Rust only (no nightly features).
- Rust edition 2021.
- `cargo fmt` compliance required.

## Style Guide
- Code style is defined in `docs/style.md`.
