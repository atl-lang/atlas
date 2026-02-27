# atlas-cli/src/

The `atlas` binary — CLI entry point for running, checking, formatting, testing,
debugging, and managing Atlas projects. Built with `clap`.

## Top-Level Structure

| Path | Role |
|------|------|
| `main.rs` | `Cli` + `Commands` clap enum; dispatches to `commands/` |
| `config.rs` | CLI-level config helpers (ATLAS_JSON, ATLAS_NO_HISTORY, NO_COLOR env vars) |

## Commands (`commands/`)

| File | Subcommand | Description |
|------|-----------|-------------|
| `run.rs` | `atlas run` | Compile + execute a `.atlas` file; supports `--watch`, `--json`, `--verbose` |
| `check.rs` | `atlas check` | Type-check without executing; exits non-zero on errors |
| `fmt.rs` | `atlas fmt` | Format source files via `atlas-formatter`; `--check` mode for CI |
| `build.rs` | `atlas build` | Build project via `atlas-build`; `--release`, `--target` flags |
| `test.rs` | `atlas test` | Discover and run tests via `testing/` |
| `repl.rs` | `atlas repl` | Basic REPL |
| `repl_tui.rs` | `atlas repl --tui` | TUI REPL with syntax highlighting |
| `debug.rs` | `atlas debug` | Interactive debugger frontend (calls `debugger/`) |
| `typecheck.rs` | `atlas typecheck` | Alias for `check` with extended type info output |
| `ast.rs` | `atlas ast` | Print AST as JSON/pretty for debugging |
| `profile.rs` | `atlas profile` | Run with profiling enabled; outputs flamegraph data |
| `lsp.rs` | `atlas lsp` | Start LSP server (stdio transport) |
| `init.rs` | `atlas init` | Initialize new project in current directory |
| `new.rs` | `atlas new` | Create new project in a new directory |
| `add.rs` | `atlas add` | Add a dependency to `atlas.toml` |
| `remove.rs` | `atlas remove` | Remove a dependency |
| `install.rs` | `atlas install` | Install all dependencies (resolve + download) |
| `update.rs` | `atlas update` | Update dependencies to latest compatible versions |
| `publish.rs` | `atlas publish` | Publish package to registry |
| `watch.rs` | `atlas watch` | Watch + re-run on file changes (standalone) |
| `mod.rs` | — | Re-exports all command modules |

## Subdirectories

| Path | Role |
|------|------|
| `debugger/mod.rs` | `Debugger` struct — interactive debug session state |
| `debugger/repl.rs` | Debug REPL loop — reads commands, calls `atlas-runtime::debugger` |
| `templates/mod.rs` | Template registry |
| `templates/binary.rs` | Starter template for binary projects |
| `templates/library.rs` | Starter template for library crates |
| `templates/web.rs` | Starter template for web projects |
| `testing/mod.rs` | Test harness entry point |
| `testing/discovery.rs` | Test file discovery: finds `*.atlas` test files in project |
| `testing/runner.rs` | `TestRunner` — executes individual test files, collects results |
| `testing/reporter.rs` | Formats test output (pass/fail/skip counts, timing) |

## Key Patterns

- **No `--permissions` flags ever.** Security context is determined by project `atlas.toml` `[security]`
  section only. CLI never accepts permission arguments. (See `atlas-runtime` security rules.)
- Commands call `atlas-runtime::Atlas::eval()` or `atlas-runtime::Compiler` + `VM` directly.
- Exit codes: `0` = success, `1` = user error (bad args, missing file), `2` = compile/type error,
  `3` = runtime error.
- `--json` outputs diagnostics as `{"errors": [...], "warnings": [...]}` to stdout.
- Watch mode (`--watch`) uses `notify` crate for filesystem events.

## Critical Rules

- **Never add permission flags to any command.** This is a locked architectural decision (auto-memory).
- LSP starts on stdio — no TCP/port flags. `atlas lsp` is invoked by editors.
- `atlas test` runner must not exec user test files in-process — always spawn a new `atlas run` subprocess
  so test failures don't crash the runner.
