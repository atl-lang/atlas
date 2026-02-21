# Atlas CLI Reference

**Version:** v0.2 | **Status:** Production Ready

Complete reference for all `atlas` command-line interface commands and options.

---

## Synopsis

```
atlas <command> [options] [arguments]
```

---

## Global Options

These options apply to all commands:

| Option | Description |
|--------|-------------|
| `--help`, `-h` | Show help for the command |
| `--version`, `-V` | Print Atlas version |
| `--verbose`, `-v` | Enable verbose output |
| `--quiet`, `-q` | Suppress non-error output |
| `--json` | Output diagnostics as JSON |
| `--no-color` | Disable color output |

---

## Commands

### `atlas run` — Run a Program

Execute an Atlas source file.

**Syntax:**
```bash
atlas run <file> [options]
atlas r <file> [options]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--watch`, `-w` | Watch for changes and auto-reload |
| `--no-clear` | Don't clear terminal on reload (with `--watch`) |
| `--verbose`, `-v` | Show timing and execution info |
| `--json` | Output diagnostics as JSON |
| `--no-optimize` | Disable bytecode optimizer |
| `--vm` | Force VM execution engine |
| `--interp` | Force interpreter engine (default) |
| `--profile` | Enable profiling, print report after execution |
| `--profile-out=<file>` | Save profile data to JSON file |
| `--dump-bytecode` | Print compiled bytecode before execution |

**Examples:**

```bash
atlas run main.atl
atlas run main.atl --watch
atlas run main.atl --vm --profile
atlas run main.atl --json 2>errors.json
atlas r main.atl --verbose
```

**Exit codes:**
- `0` — Program completed successfully
- `1` — Runtime error occurred
- `2` — Compilation/type error

---

### `atlas check` — Type Check

Check an Atlas file for errors without running it.

**Syntax:**
```bash
atlas check <file> [options]
atlas c <file> [options]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--json` | Output diagnostics as JSON |
| `--verbose`, `-v` | Show all type information |

**Examples:**

```bash
atlas check main.atl
atlas check main.atl --json
atlas c src/lib.atl
```

**Output example:**

```
Checking main.atl...
error[E0012]: type mismatch
  --> main.atl:5:9
  |
5 |     let x: string = 42;
  |             ^^^^^^   -- expected `string`, found `number`

1 error found.
```

---

### `atlas build` — Build Project

Build an Atlas project defined by `atlas.toml`.

**Syntax:**
```bash
atlas build [options]
atlas b [options]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--release` | Build with full optimization (alias for `--profile=release`) |
| `--profile <name>` | Build profile: `dev` (default), `release`, `test` |
| `--clean` | Clean rebuild, ignore cached artifacts |
| `--verbose`, `-v` | Show build steps |
| `--quiet`, `-q` | Errors only |
| `--json` | JSON diagnostics output |

**Examples:**

```bash
atlas build                    # dev build
atlas build --release          # optimized release build
atlas build --clean --release  # clean release build
atlas b --profile=test
```

**Build profiles:**

| Profile | Optimization | Debug Info | Use Case |
|---------|-------------|-----------|---------|
| `dev` | None | Full | Development |
| `release` | Aggressive | Minimal | Production |
| `test` | Basic | Full | Testing |

---

### `atlas test` — Run Tests

Discover and run all test functions in a project.

**Syntax:**
```bash
atlas test [options] [filter]
atlas t [options] [filter]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--filter <pattern>` | Only run tests matching pattern |
| `--file <path>` | Run tests in a specific file only |
| `--verbose`, `-v` | Show all test output (not just failures) |
| `--no-capture` | Don't capture stdout during tests |
| `--json` | Output results as JSON |
| `--bail` | Stop after first failure |
| `--parallel` | Run tests in parallel (default: sequential) |
| `--timeout <ms>` | Per-test timeout in milliseconds (default: 30000) |

**Test discovery:**

Atlas finds test functions by name prefix:
- Any function named `test_*` is a test
- Functions must take no arguments and return `void`

```atlas
fn test_addition() -> void {
    assertEqual(1 + 1, 2);
}

fn test_string_ops() -> void {
    assertEqual(toUpperCase("hello"), "HELLO");
    assertEqual(len("atlas"), 5);
}
```

**Examples:**

```bash
atlas test                           # run all tests
atlas test --filter addition         # run tests matching "addition"
atlas test --file src/math.atl       # test one file
atlas test --bail --verbose          # stop on first failure, verbose
atlas t --json > results.json        # JSON output
```

**Output example:**

```
Running 15 tests...

  ✓ test_addition (0.1ms)
  ✓ test_subtraction (0.1ms)
  ✗ test_division
    assertion failed: assertEqual(result, 5)
    expected: 5
    actual:   4
    at math.atl:23

Test Results: 14 passed, 1 failed (15 total) in 2.3ms
```

---

### `atlas bench` — Run Benchmarks

Discover and run benchmark functions using Criterion-style measurement.

**Syntax:**
```bash
atlas bench [options] [filter]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--filter <pattern>` | Only run benchmarks matching pattern |
| `--warmup <ms>` | Warmup duration (default: 1000ms) |
| `--measure <ms>` | Measurement duration (default: 5000ms) |
| `--json` | Output results as JSON |
| `--baseline <name>` | Compare against saved baseline |
| `--save-baseline <name>` | Save results as baseline |
| `--output-dir <path>` | Output directory for HTML reports |

**Benchmark functions:**

```atlas
fn bench_sort() -> void {
    let data = generate_data(1000);
    bubble_sort(data);
}

fn bench_string_ops() -> void {
    let s = "hello world";
    let _ = toUpperCase(s);
    let _ = split(s, " ");
}
```

**Examples:**

```bash
atlas bench                              # run all benchmarks
atlas bench --filter sort                # run matching benchmarks
atlas bench --save-baseline v02          # save baseline
atlas bench --baseline v02               # compare to baseline
```

**Output example:**

```
bench_sort          time: [2.34ms 2.41ms 2.49ms]
                    change: [-5.2% -3.1% -1.0%] (improved)
bench_string_ops    time: [125ns 128ns 132ns]
```

---

### `atlas fmt` — Format Code

Format Atlas source files to the canonical style.

**Syntax:**
```bash
atlas fmt [options] [files|dirs]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--check` | Check formatting without modifying files (exit 1 if unformatted) |
| `--dry-run` | Print formatted output without modifying files |
| `--all` | Format all files in the project |
| `--verbose`, `-v` | Show which files were modified |

**Examples:**

```bash
atlas fmt main.atl              # format one file
atlas fmt src/                  # format directory recursively
atlas fmt --all                 # format entire project
atlas fmt --check               # CI check mode
atlas fmt --dry-run main.atl    # preview formatting
```

---

### `atlas doc` — Generate Documentation

Generate HTML documentation from Atlas source files.

**Syntax:**
```bash
atlas doc [options]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--output <dir>` | Output directory (default: `./docs/`) |
| `--open` | Open generated docs in browser |
| `--include-private` | Include private items |
| `--format <fmt>` | Output format: `html` (default), `json`, `markdown` |

**Examples:**

```bash
atlas doc                       # generate docs
atlas doc --output ./api-docs   # custom output dir
atlas doc --open                # generate and open in browser
```

---

### `atlas debug` — Interactive Debugger

Launch the interactive debugger for an Atlas program.

**Syntax:**
```bash
atlas debug <file> [options]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--vm` | Use VM execution engine |
| `--interp` | Use interpreter engine (default) |
| `--no-optimize` | Disable optimizer (automatically applied) |
| `--break <line>` | Set initial breakpoint at line |
| `--break <fn>` | Set initial breakpoint at function |

**Examples:**

```bash
atlas debug main.atl
atlas debug main.atl --vm --break 10
atlas debug lib.atl --break parse_input
```

See [VM Debugger Guide](vm-debugger-guide.md) for full debugger documentation.

---

### `atlas lsp` — Language Server

Start the Atlas Language Server for editor integration.

**Syntax:**
```bash
atlas lsp [options]
atlas lsp --stdio     # standard mode (default, for most editors)
atlas lsp --tcp=<port>  # TCP mode (for debugging)
```

**Options:**

| Option | Description |
|--------|-------------|
| `--stdio` | Communicate via stdin/stdout (default) |
| `--tcp=<port>` | Listen on TCP port instead |
| `--verbose` | Enable LSP trace logging |
| `--log-file <path>` | Write log to file |

**Examples:**

```bash
atlas lsp --stdio                  # used by editors
atlas lsp --tcp=5007               # debug mode
atlas lsp --stdio --log-file=/tmp/atlas-lsp.log
```

---

### `atlas watch` — Watch Mode

Watch files for changes and re-run a command automatically.

**Syntax:**
```bash
atlas watch [options] -- <command>
```

**Options:**

| Option | Description |
|--------|-------------|
| `--watch <path>` | Additional paths to watch (repeatable) |
| `--debounce <ms>` | Debounce delay (default: 100ms) |
| `--clear` | Clear terminal before each run (default: true) |
| `--no-clear` | Don't clear terminal |

**Examples:**

```bash
atlas watch -- atlas test             # re-run tests on change
atlas watch -- atlas run main.atl     # re-run program on change
atlas watch --watch src/ -- atlas build
```

Note: `atlas run --watch` is a shorthand for watch mode on a single file.

---

### `atlas repl` — Interactive REPL

Start an interactive Read-Eval-Print Loop.

**Syntax:**
```bash
atlas repl [options]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--vm` | Use VM engine |
| `--interp` | Use interpreter (default) |
| `--debug` | Enable debugger integration |
| `--load <file>` | Load a file before starting REPL |

**Examples:**

```bash
atlas repl                    # start REPL
atlas repl --load stdlib.atl  # preload a file
atlas repl --debug            # with debugger
```

**REPL commands:**

```
:help          — show help
:quit          — exit REPL
:load <file>   — load an Atlas file
:clear         — clear the screen
:history       — show command history
:type <expr>   — show type of expression without evaluating
```

---

### `atlas completions` — Shell Completions

Generate shell completion scripts.

**Syntax:**
```bash
atlas completions <shell>
```

**Supported shells:** `bash`, `zsh`, `fish`, `powershell`

**Examples:**

```bash
# Bash
atlas completions bash > ~/.bash_completions/atlas.bash
echo 'source ~/.bash_completions/atlas.bash' >> ~/.bashrc

# Zsh
atlas completions zsh > ~/.zfunc/_atlas
# Add to .zshrc: fpath=(~/.zfunc $fpath)

# Fish
atlas completions fish > ~/.config/fish/completions/atlas.fish

# PowerShell
atlas completions powershell > atlas.ps1
```

---

### `atlas new` — Create a New Project

Scaffold a new Atlas project from a template.

**Syntax:**
```bash
atlas new <name> [options]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--template <name>` | Project template: `default`, `library`, `cli`, `web` |
| `--no-git` | Don't initialize a git repository |

**Examples:**

```bash
atlas new my-project
atlas new my-lib --template library
atlas new my-cli --template cli --no-git
```

**Generated structure:**

```
my-project/
├── atlas.toml
├── .gitignore
├── README.md
└── src/
    └── main.atl
```

---

### `atlas add` — Add a Dependency

Add a package dependency to `atlas.toml`.

**Syntax:**
```bash
atlas add <package> [options]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--version <ver>` | Specific version or constraint |
| `--dev` | Add as dev dependency |

**Examples:**

```bash
atlas add http-client
atlas add json-utils --version "^1.2"
atlas add test-helpers --dev
```

---

### `atlas install` — Install Dependencies

Install all dependencies listed in `atlas.toml`.

**Syntax:**
```bash
atlas install [options]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--frozen` | Use exact versions from lock file |
| `--no-lock` | Don't update the lock file |

**Examples:**

```bash
atlas install
atlas install --frozen    # CI: use exact locked versions
```

---

## Configuration File (`atlas.toml`)

```toml
[project]
name = "my-project"
version = "1.0.0"
description = "My Atlas project"
authors = ["Alice <alice@example.com>"]
entry = "src/main.atl"

[build]
profile = "dev"
output = "dist/"

[formatter]
indent_width = 4
max_line_width = 100
trailing_commas = true

[test]
timeout = 30000      # ms per test
parallel = false

[bench]
warmup = 1000        # ms
measure = 5000       # ms

[dependencies]
http-client = "^1.0"

[dev-dependencies]
test-helpers = "^2.0"

[security]
filesystem = "read-write"
network = "allow"
process = "deny"
```

---

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | Runtime error or test failure |
| `2` | Compile error or type error |
| `3` | Configuration error |
| `4` | File not found |
| `5` | Permission denied |

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `ATLAS_ROOT` | Override the Atlas installation directory |
| `ATLAS_CACHE` | Override the package cache directory |
| `ATLAS_LOG` | Log level: `error`, `warn`, `info`, `debug`, `trace` |
| `NO_COLOR` | Disable color output (standard convention) |

---

*See also: [Formatter Guide](formatter-guide.md) | [VM Debugger Guide](vm-debugger-guide.md) | [Embedding Guide](embedding-guide.md)*
