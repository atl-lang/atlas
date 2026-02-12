# Atlas REPL Architecture

## Goal
Provide a single REPL core that supports multiple frontends (line-editor and TUI) without duplicating language logic.

## Design
Split REPL into two layers:

### 1. REPL Core (shared)
- Responsible for:
  - Reading complete input blocks (already provided by frontend).
  - Parse -> bind -> typecheck -> evaluate.
  - Producing diagnostics in human and JSON formats.
- Exposes a minimal interface:
  - `eval(input: &str) -> ReplResult`
  - `state` persists across inputs.

### 2. REPL UI (frontend)
- Responsible for:
  - Input capture (line editor or TUI).
  - Displaying results and diagnostics.
  - Multi-line input handling (balance braces and quotes).

## Frontends
- Line editor (default): `rustyline` or `reedline`.
- Optional TUI: `ratatui` behind a flag (e.g., `atlas repl --tui`).

## ReplResult
- `value: Option<Value>`
- `diagnostics: Vec<Diagnostic>`
- `output: Vec<String>` (captured stdout, if needed)

## Testing
- REPL core tests should not depend on terminal UI.
- Frontend tests should focus on input handling only.
