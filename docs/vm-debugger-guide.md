# Atlas VM Debugger Guide

**Version:** v0.2 | **Status:** Production Ready

The Atlas debugger provides interactive debugging for Atlas programs, supporting both the VM and interpreter execution engines.

---

## Overview

The Atlas debugger supports:
- **Breakpoints** — pause execution at specific lines
- **Stepping** — step over, step into, step out
- **Inspection** — view variable values and types
- **Backtraces** — full call stack with source locations
- **REPL integration** — evaluate expressions in the current scope
- **Source display** — shows current source line during execution

---

## Starting the Debugger

### CLI

```bash
atlas debug main.atl           # launch debugger
atlas debug main.atl --vm      # force VM mode
atlas debug main.atl --interp  # force interpreter mode
```

The debugger starts, loads the program, and pauses before the first instruction:

```
Atlas Debugger v0.2
Loading: main.atl
Paused at main.atl:1

  1 → fn greet(name: string) -> string {
  2       return concat("Hello, ", name);
  3   }
  4
  5   let msg = greet("World");

(debug) _
```

---

## Debugger Commands

### Navigation

| Command | Aliases | Description |
|---------|---------|-------------|
| `next` | `n` | Execute next line (step over functions) |
| `step` | `s` | Step into function calls |
| `out` | `o` | Step out of current function |
| `continue` | `c` | Continue until next breakpoint or end |
| `run` | `r` | Restart from the beginning |
| `quit` | `q`, `exit` | Exit the debugger |

### Breakpoints

| Command | Description |
|---------|-------------|
| `break <line>` | Set breakpoint at line number in current file |
| `break <file>:<line>` | Set breakpoint in a specific file |
| `break <function>` | Set breakpoint at function entry |
| `delete <id>` | Delete breakpoint by ID |
| `disable <id>` | Disable a breakpoint (keeps it, doesn't pause) |
| `enable <id>` | Re-enable a disabled breakpoint |
| `breakpoints` | List all breakpoints |
| `clear` | Delete all breakpoints |

### Inspection

| Command | Aliases | Description |
|---------|---------|-------------|
| `vars` | `v` | Show all local variables in current scope |
| `inspect <expr>` | `p <expr>` | Evaluate and print an expression |
| `backtrace` | `bt`, `stack` | Show the full call stack |
| `frame <n>` | `f <n>` | Switch to call frame n |
| `locals` | | Show locals for current frame |
| `globals` | | Show all global variables |

### Display

| Command | Description |
|---------|-------------|
| `list` | Show current source context (±5 lines) |
| `list <line>` | Show source around specific line |
| `list <from>,<to>` | Show source line range |
| `source` | Show full source file |

---

## Debugging Session Example

### Example Program: `calculator.atl`

```atlas
fn divide(a: number, b: number) -> Result<number, string> {
    if b == 0 {
        return Err("division by zero");
    }
    return Ok(a / b);
}

fn calculate(x: number, y: number) -> void {
    let result = divide(x, y);
    if is_ok(result) {
        print(concat("Result: ", str(unwrap(result))));
    } else {
        print(concat("Error: ", str(unwrap(result_err(result)))));
    }
}

calculate(10, 2);
calculate(5, 0);
```

### Interactive Session

```
$ atlas debug calculator.atl

Atlas Debugger v0.2
Paused at calculator.atl:16

 16 → calculate(10, 2);
 17   calculate(5, 0);

(debug) break divide
Breakpoint 1 set at function 'divide'

(debug) continue
Breakpoint 1 hit: calculator.atl:1

  1 → fn divide(a: number, b: number) -> Result<number, string> {
  2       if b == 0 {
  3           return Err("division by zero");
  4       }

(debug) vars
  a: number = 10
  b: number = 2

(debug) next
Paused at calculator.atl:5

  5 → return Ok(a / b);

(debug) inspect a / b
= 5

(debug) continue
Result: 5
Breakpoint 1 hit: calculator.atl:1

(debug) vars
  a: number = 5
  b: number = 0

(debug) next
Paused at calculator.atl:2

  2 → if b == 0 {

(debug) next
Paused at calculator.atl:3

  3 → return Err("division by zero");

(debug) backtrace
#0 divide (calculator.atl:3)
#1 calculate (calculator.atl:9)
#2 <main> (calculator.atl:17)

(debug) continue
Error: division by zero

Program complete.
(debug) quit
```

---

## Advanced Features

### Conditional Breakpoints

Set a breakpoint that only triggers when a condition is met:

```
(debug) break 5 if b == 0
Breakpoint 2 set at calculator.atl:5 (condition: b == 0)
```

### Watchpoints

Break when a variable's value changes:

```
(debug) watch result
Watchpoint set on 'result'
```

### Frame Navigation

Inspect different levels of the call stack:

```
(debug) backtrace
#0 divide (calculator.atl:3)
#1 calculate (calculator.atl:9)
#2 <main> (calculator.atl:17)

(debug) frame 1
Switched to frame #1: calculate (calculator.atl:9)

(debug) vars
  x: number = 5
  y: number = 0
  result: (not yet assigned)
```

### Evaluating Expressions

The `inspect` command evaluates any Atlas expression in the current scope:

```
(debug) inspect len("hello")
= 5

(debug) inspect a + b
= 7

(debug) inspect [1, 2, 3][0]
= 1
```

---

## Debugger with VM vs Interpreter

The debugger works with both execution engines:

| Feature | VM Mode | Interpreter Mode |
|---------|---------|-----------------|
| Breakpoints | ✓ | ✓ |
| Step over/into | ✓ | ✓ |
| Variable inspection | ✓ | ✓ |
| Backtrace | ✓ | ✓ |
| Source mapping | ✓ (via debug info) | ✓ (direct) |
| Performance | Optimized | Less overhead |

**Recommendation:** Use interpreter mode (`--interp`) for debugging complex logic — it has lower overhead and simpler source mapping. Use VM mode (`--vm`) to debug VM-specific behavior.

---

## Debugging Async Code

For programs using `spawn` and `await`, the debugger provides task-aware stepping:

```
(debug) tasks
Tasks:
  #0 (main) - running
  #1 (spawn@main.atl:15) - waiting

(debug) switch-task 1
Switched to task #1
Paused at main.atl:15
```

---

## Integration with REPL

The Atlas REPL (`atlas repl`) includes debugging capabilities:

```bash
atlas repl --debug
```

In the REPL, you can:
- Set breakpoints on functions before calling them
- Inspect values after execution
- Step through function bodies

---

## Limitations

- **Optimized code:** Debugging optimized VM code may show unexpected stepping behavior. Use `--no-optimize` (automatically applied when debugging).
- **Native functions:** Cannot step into stdlib functions implemented in Rust
- **FFI calls:** Debugger cannot step into C functions called via FFI
- **Async tasks:** Limited support for debugging concurrent task interactions

---

## Tips

1. **Start with `continue`** — Set all breakpoints first, then run to the first one
2. **Use `inspect` liberally** — Evaluate expressions to understand state
3. **Check the backtrace** — When lost, `bt` shows exactly where you are
4. **Use conditional breakpoints** — Avoid stepping through 1000 loop iterations
5. **Watch variables** — Watchpoints find when values change unexpectedly

---

*See also: [VM Optimizer Guide](vm-optimizer-guide.md) | [VM Profiler Guide](vm-profiler-guide.md) | [CLI Reference](cli-reference.md) | [REPL Guide](repl.md)*
