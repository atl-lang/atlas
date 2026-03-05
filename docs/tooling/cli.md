# Atlas CLI (Selected Commands)

This document is based on the CLI implementation in `crates/atlas-cli/src/main.rs` and command handlers.

**atlas run**
- Compiles and executes an Atlas source file.
- Prints the program result if it is not `null`.
- Uses full permissions in the runtime (`SecurityContext::allow_all()`).
- Supports module loading in the runtime (see known issues for limitations).

Usage:
```
atlas run <file>
```

Options:
- `--json` — output diagnostics as JSON (also via `ATLAS_JSON=1`).
- `-w`, `--watch` — re-run on file changes.
- `--no-clear` — do not clear the terminal in watch mode.
- `-v`, `--verbose` — verbose timing output.
- Alias: `atlas r`.

Example (tested):
```bash
atlas run /tmp/atlas-docs-examples.atlas
```

**atlas check**
- Performs lexing, parsing, binding, and type checking.
- Does **not** execute the program.
- Exits with failure only on errors (warnings are reported but do not fail).

Usage:
```
atlas check <file>
```

Options:
- `--json` — output diagnostics as JSON (also via `ATLAS_JSON=1`).
- Alias: `atlas c`.

Example (tested):
```bash
atlas check /tmp/atlas-docs-examples.atlas
```

**Known Issues (See `docs/known-issues.md`)**
- `import` parses, but multi-file module resolution does not work at runtime yet (H-063).
- `.atl` extension does not execute reliably; use `.atlas` (H-067).
