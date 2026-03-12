# Atlas CLI Reference

## Commands Overview

| Command | Alias | Description |
|---------|-------|-------------|
| `atlas run <file>` | `r` | Run an Atlas program |
| `atlas test` | `t` | Run tests |
| `atlas fmt <files>` | `f` | Format source files |
| `atlas build` | `b` | Build project |
| `atlas debug <file>` | `d` | Debug interactively |
| `atlas repl` | - | Interactive REPL |
| `atlas lsp` | - | Start language server |
| `atlas explain <code>` | - | Explain an error code |
| `atlas new <name>` | `n` | Create new project |
| `atlas init` | `i` | Initialize project in current dir |

## atlas run

Run an Atlas program.

```bash
atlas run main.atl              # run program
atlas run main.atl --watch      # watch for changes
atlas run main.atl --json       # JSON diagnostics
atlas run main.atl --verbose    # timing info
```

| Flag | Description |
|------|-------------|
| `--watch`, `-w` | Auto-rerun on file changes |
| `--json` | Output diagnostics as JSON |
| `--verbose`, `-v` | Show timing information |
| `--no-clear` | Don't clear terminal in watch mode |

## atlas test

Run tests. See [testing.md](testing.md) for full guide.

```bash
atlas test                      # all tests in current dir
atlas test <pattern>            # filter by name
atlas test --dir=tests/         # specific directory
atlas test --verbose            # show each test name
atlas test --sequential         # disable parallel
atlas test --json               # JSON output
```

## atlas fmt

Format Atlas source files.

```bash
atlas fmt src/                  # format directory
atlas fmt main.atl              # format single file
atlas fmt . --check             # check without modifying
atlas fmt . --write             # write changes (explicit)
```

| Flag | Description |
|------|-------------|
| `--check` | Check formatting, exit 1 if changes needed |
| `--write`, `-w` | Write changes to files |
| `--indent-size=N` | Indentation spaces (default: 4) |
| `--max-width=N` | Max line width (default: 100) |
| `--verbose`, `-v` | Show timing info |
| `--quiet`, `-q` | Suppress non-error output |

## atlas build

Build an Atlas project (requires atlas.toml).

```bash
atlas build                     # default profile
atlas build --release           # optimized build
atlas build --profile=test      # custom profile
atlas build --clean             # ignore cache
```

| Flag | Description |
|------|-------------|
| `--release` | Build with release profile |
| `--profile=NAME` | Use named profile |
| `--clean` | Clean build, ignore cache |
| `--verbose`, `-v` | Verbose output |
| `--quiet`, `-q` | Errors only |
| `--json` | JSON diagnostics |

## atlas debug

Interactive debugger.

```bash
atlas debug main.atl            # start debugging
atlas debug main.atl -b 10      # break at line 10
atlas debug main.atl -b 10 -b 20  # multiple breakpoints
```

### Debugger Commands

| Command | Alias | Description |
|---------|-------|-------------|
| `break <line>` | `b` | Set breakpoint |
| `step` | `s` | Step into |
| `next` | `n` | Step over |
| `continue` | `c` | Continue execution |
| `print <expr>` | `p` | Evaluate expression |
| `vars` | - | Show local variables |
| `backtrace` | `bt` | Show call stack |
| `quit` | `q` | Exit debugger |

## atlas repl

Interactive Read-Eval-Print Loop.

```bash
atlas repl                      # line editor mode
atlas repl --tui                # TUI mode with highlighting
atlas repl --no-history         # disable history persistence
```

### REPL Commands

| Command | Description |
|---------|-------------|
| `:help`, `:h` | Show help |
| `:quit`, `:q` | Exit REPL |
| `:reset` | Clear all definitions |
| `:load <file>` | Load and run file |
| `:type <expr>` | Show expression type |
| `:vars` | List defined variables |

## atlas lsp

Start the Language Server Protocol server.

```bash
atlas lsp                       # stdio mode (for editors)
atlas lsp --tcp                 # TCP mode
atlas lsp --tcp --port=8080     # custom port
atlas lsp --verbose             # enable logging
```

## atlas explain

Look up error codes.

```bash
atlas explain AT1003            # explain error AT1003
atlas explain 1003              # same (AT prefix inferred)
atlas explain --list            # list all error codes
```

## atlas new

Create a new project from template.

```bash
atlas new my-app                # binary project
atlas new my-lib --lib          # library project
atlas new my-api --web          # web server project
atlas new my-app --author="..."
atlas new --list                # list available templates
```

| Flag | Description |
|------|-------------|
| `--lib` | Create library project |
| `--web` | Create web server project |
| `--template=NAME` | Specify template |
| `--author=NAME` | Set author |
| `--no-git` | Skip git initialization |
| `--no-commit` | Skip initial commit |
| `--force` | Overwrite existing directory |

## atlas init

Initialize project in current directory.

```bash
atlas init                      # interactive
atlas init my-project           # with name
atlas init --lib                # library project
atlas init --no-git             # skip git
```

## atlas profile

Profile program execution.

```bash
atlas profile slow.atl          # profile execution
atlas profile slow.atl -o report.txt  # save report
atlas profile slow.atl --summary      # brief output
```

## atlas ast

Dump AST as JSON (for tooling/debugging).

```bash
atlas ast main.atl              # print AST
atlas ast main.atl > ast.json   # save to file
```

## atlas completions

Generate shell completions.

```bash
atlas completions bash > ~/.bash_completions/atlas.bash
atlas completions zsh > ~/.zfunc/_atlas
atlas completions fish > ~/.config/fish/completions/atlas.fish
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `ATLAS_JSON` | Set to `1` for JSON output by default |
| `ATLAS_NO_HISTORY` | Set to `1` to disable REPL history |
| `NO_COLOR` | Set to disable colored output |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | User error (bad args, missing file) |
| 2 | Compile/type error |
| 3 | Runtime error |
