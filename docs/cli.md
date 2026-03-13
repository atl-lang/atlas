# Atlas CLI Reference

## Commands Overview

| Command | Alias | Description |
|---------|-------|-------------|
| `atlas run <file>` | `r` | Compile and run an Atlas program |
| `atlas build` | `b` | Build project from `atlas.toml` |
| `atlas test` | `t` | Discover and run tests |
| `atlas fmt <files>` | `f` | Format source files |
| `atlas debug <file>` | `d` | Interactive debugger |
| `atlas repl` | | Interactive REPL |
| `atlas lsp` | | Language Server (stdio) |
| `atlas new <name>` | `n` | Create new project from template |
| `atlas init` | `i` | Initialize project in current directory |
| `atlas add <pkg>` | | Add a dependency to `atlas.toml` |
| `atlas remove <pkg>` | `rm` | Remove a dependency |
| `atlas install` | | Install all dependencies |
| `atlas update` | `up` | Update dependencies |
| `atlas publish` | | Publish package to registry |
| `atlas explain <code>` | | Explain an error code |
| `atlas profile <file>` | | Profile VM execution |
| `atlas ast <file>` | | Dump AST as JSON |
| `atlas typecheck <file>` | | Dump type information as JSON |
| `atlas completions <shell>` | | Generate shell completions |

---

## atlas run

Compile and execute an Atlas source file.

```bash
atlas run main.atl              # run program
atlas run main.atl foo bar      # pass arguments to program
atlas run main.atl --watch      # watch for changes, re-run automatically
atlas run main.atl --json       # JSON diagnostics output
atlas run main.atl --verbose    # show timing information
```

Program arguments (after the file path) are accessible via `process.getProcessArgs()`.

| Flag | Short | Description |
|------|-------|-------------|
| `--watch` | `-w` | Re-run on file changes |
| `--no-clear` | | Do not clear terminal before re-run (watch mode) |
| `--json` | | Output diagnostics as JSON |
| `--verbose` | `-v` | Show timing information |

---

## atlas build

Build an Atlas project. Requires `atlas.toml` in the current directory.

```bash
atlas build                     # development build → target/debug/<name>
atlas build --release           # release build → target/release/<name>
atlas build --profile=test      # custom profile
atlas build --clean             # ignore cache
```

The resulting binary is self-contained — it embeds the Atlas VM and requires no runtime on the target machine.

| Flag | Short | Description |
|------|-------|-------------|
| `--release` | | Optimized release build |
| `--profile=NAME` | `-p` | Use named build profile |
| `--clean` | | Clean build, ignore cache |
| `--verbose` | `-v` | Verbose output |
| `--quiet` | `-q` | Errors only |
| `--json` | | JSON diagnostics |

See [language/build.md](language/build.md) for `atlas.toml` reference.

---

## atlas test

Discover and run test files. See [testing.md](testing.md) for the full guide.

```bash
atlas test                      # all *.test.atl in current directory
atlas test auth                 # filter tests by name pattern
atlas test --dir=tests/unit     # specific directory
atlas test --verbose            # show each test name
atlas test --sequential         # disable parallel execution
atlas test --json               # JSON output for CI
atlas test --no-color           # disable colored output
```

| Flag | Short | Description |
|------|-------|-------------|
| `<pattern>` | | Filter tests whose names contain this string |
| `--dir=PATH` | | Directory to search (default: `.`) |
| `--sequential` | | Run tests one at a time |
| `--verbose` | `-v` | Print every test name |
| `--no-color` | | Disable colored output |
| `--json` | | JSON output |

Exit codes: `0` = all passed, `1` = one or more failed.

---

## atlas fmt

Format Atlas source files.

```bash
atlas fmt src/                  # format all files in directory
atlas fmt main.atl              # format single file
atlas fmt . --check             # check without modifying (CI mode)
atlas fmt . --write             # write changes (explicit)
atlas fmt main.atl --indent-size=2
```

| Flag | Short | Description |
|------|-------|-------------|
| `--check` | | Exit 1 if files need formatting, do not write |
| `--write` | `-w` | Write changes to files |
| `--config=PATH` | `-c` | Path to formatter config file |
| `--indent-size=N` | | Indentation in spaces (default: 4) |
| `--max-width=N` | | Maximum line width (default: 100) |
| `--trailing-commas` | | Enable or disable trailing commas |
| `--verbose` | `-v` | Show timing information |
| `--quiet` | `-q` | Suppress non-error output |

---

## atlas debug

Interactive debugger with breakpoints and stepping.

```bash
atlas debug main.atl            # start debugging
atlas debug main.atl -b 10      # break at line 10
atlas debug main.atl -b 10 -b 20  # multiple breakpoints
```

| Flag | Short | Description |
|------|-------|-------------|
| `--breakpoint=LINE` | `-b` | Set breakpoint at line number (repeatable) |

### Debugger Session Commands

| Command | Alias | Description |
|---------|-------|-------------|
| `break <line>` | | Set breakpoint |
| `step` | `s` | Step into |
| `next` | `n` | Step over |
| `continue` | `c` | Continue execution |
| `print <expr>` | | Evaluate and print expression |
| `vars` | | Show local variables |
| `backtrace` | `bt` | Show call stack |
| `quit` | `q` | Exit debugger |

---

## atlas repl

Interactive Read-Eval-Print Loop.

```bash
atlas repl                      # line editor mode
atlas repl --tui                # TUI mode with syntax highlighting
atlas repl --no-history         # disable history persistence
```

History is stored at `~/.atlas/history` by default.

| Flag | Description |
|------|-------------|
| `--tui` | Use ratatui TUI instead of line editor |
| `--no-history` | Disable history persistence |

### REPL Session Commands

| Command | Description |
|---------|-------------|
| `:help`, `:h` | Show help |
| `:quit`, `:q` | Exit REPL |
| `:reset` | Clear all definitions |
| `:load <file>` | Load and run a file |
| `:type <expr>` | Show inferred type of expression |
| `:vars` | List defined variables |

---

## atlas lsp

Start the Language Server Protocol server for editor integration.

```bash
atlas lsp                       # stdio mode (editors use this)
atlas lsp --tcp                 # TCP mode
atlas lsp --tcp --port=8080     # custom port (default: 9257)
atlas lsp --verbose             # enable logging
```

| Flag | Description |
|------|-------------|
| `--tcp` | Use TCP instead of stdio |
| `--port=N` | TCP port (default: 9257) |
| `--host=ADDR` | Bind address (default: 127.0.0.1) |
| `--verbose`, `-v` | Enable verbose logging |

---

## atlas new

Create a new project from a template.

```bash
atlas new my-app                # binary project (default)
atlas new my-lib --lib          # library project
atlas new my-api --web          # web server project
atlas new --list                # list available templates
```

Templates: `binary` (default), `library`, `web`.

| Flag | Short | Description |
|------|-------|-------------|
| `--lib` | | Create library project |
| `--web` | | Create web server project |
| `--template=NAME` | `-t` | Explicit template name |
| `--author=NAME` | | Set author name |
| `--description=TEXT` | | Set project description |
| `--no-git` | | Skip git initialization |
| `--no-commit` | | Skip initial commit |
| `--force` | | Overwrite existing directory |
| `--list` | | Print available templates |
| `--verbose` | `-v` | Verbose output |

---

## atlas init

Initialize an Atlas project in the current directory.

```bash
atlas init                      # interactive mode
atlas init my-project           # provide name directly
atlas init --lib                # library project
atlas init --no-git             # skip git initialization
```

| Flag | Description |
|------|-------------|
| `<name>` | Project name (defaults to directory name) |
| `--lib` | Create library project |
| `--no-git` | Skip git repository initialization |
| `--verbose`, `-v` | Verbose output |

---

## atlas add

Add a dependency to `atlas.toml`.

```bash
atlas add http                  # latest version
atlas add http@1.2              # specific version
atlas add http --dev            # dev dependency
atlas add utils --git=https://github.com/example/utils
atlas add mylib --path=../mylib # local dependency
```

| Flag | Description |
|------|-------------|
| `--ver=VERSION` | Version constraint |
| `--dev` | Add as dev dependency |
| `--git=URL` | Git repository source |
| `--branch=NAME` | Git branch |
| `--tag=NAME` | Git tag |
| `--rev=HASH` | Git revision |
| `--path=PATH` | Local path dependency |
| `--features=NAME` | Enable features (`-F` short) |
| `--no-default-features` | Disable default features |
| `--optional` | Mark as optional |
| `--rename=NAME` | Rename the dependency |
| `--dry-run` | Show what would change, don't write |

---

## atlas remove

Remove a dependency from `atlas.toml`.

```bash
atlas remove http               # remove single dependency
atlas remove http json          # remove multiple
atlas remove test-utils --dev   # remove from dev-dependencies
```

| Flag | Short | Description |
|------|-------|-------------|
| `--dev` | | Remove from dev dependencies |
| `--dry-run` | | Show what would change, don't write |
| `--verbose` | `-v` | Verbose output |

---

## atlas install

Install all dependencies listed in `atlas.toml`.

```bash
atlas install                   # install everything
atlas install --production      # skip dev dependencies
atlas install --force           # reinstall even if cached
```

| Flag | Short | Description |
|------|-------|-------------|
| `--production` | | Skip dev dependencies |
| `--force` | | Reinstall even if cached |
| `--dry-run` | | Show what would be installed |
| `--verbose` | `-v` | Verbose output |
| `--quiet` | `-q` | Errors only |

---

## atlas update

Update dependencies to their latest compatible versions.

```bash
atlas update                    # update all
atlas update http               # update specific package
atlas update --dry-run          # show what would change
```

| Flag | Short | Description |
|------|-------|-------------|
| `<packages>` | | Specific packages to update (empty = all) |
| `--dev` | | Update only dev dependencies |
| `--dry-run` | | Show what would change |
| `--verbose` | `-v` | Verbose output |

---

## atlas publish

Publish a package to the Atlas registry.

```bash
atlas publish                   # publish to default registry
atlas publish --dry-run         # validate without publishing
atlas publish --no-verify       # skip validation steps
```

| Flag | Description |
|------|-------------|
| `--registry=URL` | Target registry |
| `--no-verify` | Skip validation checks |
| `--dry-run` | Validate without publishing |
| `--allow-dirty` | Allow publishing with uncommitted changes |
| `--verbose`, `-v` | Verbose output |

---

## atlas explain

Look up an error code.

```bash
atlas explain AT1003            # explain error AT1003
atlas explain 1003              # AT prefix inferred
atlas explain at1003            # case-insensitive
atlas explain --list            # list all error codes
```

---

## atlas profile

Profile VM execution to identify hotspots.

```bash
atlas profile slow.atl          # profile and print report
atlas profile slow.atl -o report.txt  # save to file
atlas profile slow.atl --summary     # brief output only
```

| Flag | Short | Description |
|------|-------|-------------|
| `--threshold=N` | | Hotspot percentage threshold (default: 1.0) |
| `--output=FILE` | `-o` | Save report to file |
| `--summary` | | Print summary only |

---

## atlas ast

Dump the AST as JSON. Primarily for tooling and debugging.

```bash
atlas ast main.atl              # print AST
atlas ast main.atl > ast.json   # save to file
```

---

## atlas typecheck

Dump type information as JSON.

```bash
atlas typecheck main.atl        # print type info
atlas typecheck main.atl | jq   # pipe to jq
```

---

## atlas completions

Generate shell completion scripts.

```bash
atlas completions bash > ~/.bash_completions/atlas.bash
atlas completions zsh > ~/.zfunc/_atlas
atlas completions fish > ~/.config/fish/completions/atlas.fish
atlas completions powershell > atlas.ps1
```

**Installation:**
- Bash: add `source ~/.bash_completions/atlas.bash` to `~/.bashrc`
- Zsh: add `fpath=(~/.zfunc $fpath)` before `compinit` in `~/.zshrc`
- Fish: completions auto-loaded from the completions directory

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `ATLAS_DIAGNOSTICS=json` | Output diagnostics as JSON by default |
| `ATLAS_NO_HISTORY=1` | Disable REPL history persistence |
| `ATLAS_HISTORY_FILE=<path>` | Custom REPL history file path |
| `ATLAS_REPL_SHOW_TYPES=0` | Disable automatic type display in REPL |
| `ATLAS_NO_COLOR=1` | Disable colored output |
| `NO_COLOR=1` | Standard no-color convention (also respected) |

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | User error (bad arguments, missing file) |
| 2 | Compile or type error |
| 3 | Runtime error |
