# Atlas CLI

This document reflects the CLI implementation in `crates/atlas-cli/src/main.rs` and command handlers.

**atlas run**
- Compiles and executes an Atlas source file.

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

Example:
```bash
atlas run main.atl
```

**atlas check**
- Performs lexing, parsing, binding, and type checking without executing.

Usage:
```
atlas check <file>
```

Options:
- `--json` — output diagnostics as JSON (also via `ATLAS_JSON=1`).
- Alias: `atlas c`.

Example:
```bash
atlas check main.atl
```

**atlas build**
- Builds an Atlas project using `atlas.toml` configuration.

Usage:
```
atlas build
```

Options:
- `-p`, `--profile` — build profile (dev, release, test, or custom).
- `--release` — shorthand for `--profile=release`.
- `--clean` — ignore cache.
- `-v`, `--verbose` — verbose output.
- `-q`, `--quiet` — errors only.
- `--json` — JSON output.
- Alias: `atlas b`.

Example:
```bash
atlas build --release
```

**atlas repl**
- Starts an interactive REPL.

Usage:
```
atlas repl
```

Options:
- `--tui` — use the TUI interface.
- `--no-history` — disable history persistence (also via `ATLAS_NO_HISTORY=1`).

Example:
```bash
atlas repl --tui
```

**atlas ast**
- Dumps the AST as JSON.

Usage:
```
atlas ast <file>
```

Example:
```bash
atlas ast main.atl > ast.json
```

**atlas typecheck**
- Dumps typecheck information as JSON.

Usage:
```
atlas typecheck <file>
```

Example:
```bash
atlas typecheck main.atl
```

**atlas fmt**
- Formats Atlas source files.

Usage:
```
atlas fmt <files...>
```

Options:
- `--check` — verify formatting without writing.
- `-w`, `--write` — write changes.
- `-c`, `--config` — formatter config path.
- `--indent-size` — indentation size in spaces.
- `--max-width` — maximum line width.
- `--trailing-commas` — enable or disable trailing commas.
- `-v`, `--verbose` — verbose output.
- `-q`, `--quiet` — errors only.
- Alias: `atlas f`.

Example:
```bash
atlas fmt src/ --check
```

**atlas profile**
- Profiles a program using VM execution tracing.

Usage:
```
atlas profile <file>
```

Options:
- `--threshold` — hotspot percentage threshold.
- `-o`, `--output` — write report to a file.
- `--summary` — summary output only.

Example:
```bash
atlas profile slow.atl --summary
```

**atlas test**
- Runs Atlas test files.

Usage:
```
atlas test [pattern]
```

Options:
- `--sequential` — disable parallelism.
- `-v`, `--verbose` — show all test names.
- `--no-color` — disable colored output.
- `--dir` — test directory (default: `.`).
- `--json` — JSON output.
- Alias: `atlas t`.

Example:
```bash
atlas test --dir=tests
```

**atlas debug**
- Starts an interactive debugger session.

Usage:
```
atlas debug <file>
```

Options:
- `-b`, `--breakpoint` — line number breakpoints (repeatable).
- Alias: `atlas d`.

Example:
```bash
atlas debug main.atl -b 12
```

**atlas lsp**
- Runs the Language Server (stdio or TCP).

Usage:
```
atlas lsp
```

Options:
- `--tcp` — TCP mode instead of stdio.
- `--port` — TCP port (default: 9257).
- `--host` — bind address (default: 127.0.0.1).
- `-v`, `--verbose` — verbose logging.

Example:
```bash
atlas lsp --tcp --port=9257
```

**atlas completions**
- Generates shell completion scripts.

Usage:
```
atlas completions <shell>
```

Options:
- `<shell>` — one of `bash`, `zsh`, `fish`, `powershell`.

Example:
```bash
atlas completions zsh > ~/.zfunc/_atlas
```

**atlas init**
- Initializes a new Atlas project.

Usage:
```
atlas init [name]
```

Options:
- `--lib` — create a library project.
- `--no-git` — skip git initialization.
- `-v`, `--verbose` — verbose output.
- Alias: `atlas i`.

Example:
```bash
atlas init my-project
```

**atlas add**
- Adds a dependency to `atlas.toml`.

Usage:
```
atlas add <package>
```

Options:
- `--ver` — version constraint.
- `--dev` — add as dev dependency.
- `--git` — git repository URL.
- `--branch`, `--tag`, `--rev` — git revision selectors.
- `--path` — local path dependency.
- `-F`, `--features` — enable features.
- `--no-default-features` — disable default features.
- `--optional` — mark as optional dependency.
- `--rename` — rename dependency.
- `--dry-run` — do not modify files.

Example:
```bash
atlas add http@1.2 --dev
```

**atlas remove**
- Removes dependencies from `atlas.toml`.

Usage:
```
atlas remove <packages...>
```

Options:
- `--dev` — remove from dev dependencies.
- `--dry-run` — do not modify files.
- `-v`, `--verbose` — verbose output.
- Alias: `atlas rm`.

Example:
```bash
atlas remove http json
```

**atlas install**
- Installs project dependencies.

Usage:
```
atlas install
```

Options:
- `--production` — skip dev dependencies.
- `--force` — reinstall even if cached.
- `--dry-run` — simulate install.
- `-v`, `--verbose` — verbose output.
- `-q`, `--quiet` — errors only.

Example:
```bash
atlas install --production
```

**atlas update**
- Updates dependencies to latest compatible versions.

Usage:
```
atlas update [packages...]
```

Options:
- `--dev` — update dev dependencies only.
- `--dry-run` — show updates without modifying files.
- `-v`, `--verbose` — verbose output.
- Alias: `atlas up`.

Example:
```bash
atlas update http
```

**atlas publish**
- Publishes a package to the registry.

Usage:
```
atlas publish
```

Options:
- `--registry` — registry URL/name.
- `--no-verify` — skip validation.
- `--dry-run` — validate without publishing.
- `--allow-dirty` — allow dirty git state.
- `-v`, `--verbose` — verbose output.

Example:
```bash
atlas publish --dry-run
```

**atlas new**
- Creates a new project from a template.

Usage:
```
atlas new <name>
```

Options:
- `--lib` — library template.
- `--web` — web server template.
- `-t`, `--template` — template name (binary, library, web).
- `--author` — author name.
- `--description` — project description.
- `--no-git` — skip git initialization.
- `--no-commit` — skip initial commit.
- `--force` — overwrite existing directory.
- `--list` — list templates.
- `-v`, `--verbose` — verbose output.
- Alias: `atlas n`.

Example:
```bash
atlas new my-app --template=binary
```

**Known Issues (See `docs/known-issues.md`)**
- Closures passed as callbacks do not persist global mutations (H-069).
