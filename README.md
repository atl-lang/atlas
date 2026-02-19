<div align="center">

# Atlas

**A statically-typed programming language with dual execution engines, built in Rust.**

[![CI](https://github.com/proxikal/atlas/actions/workflows/ci.yml/badge.svg)](https://github.com/proxikal/atlas/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust: 1.70+](https://img.shields.io/badge/Rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Lines of Code](https://img.shields.io/badge/Lines%20of%20Code-125k-green.svg)](#project-metrics)

</div>

---

## Overview

Atlas is a research programming language featuring a **bytecode virtual machine** and a **tree-walking interpreter** that maintain 100% behavioral parity. The language emphasizes explicitness over magic: no implicit type coercion, no truthy/falsy values, and precise error diagnostics with source locations.

**Key characteristics:**
- **Strict type system** with inference - catch errors at compile time
- **Dual execution engines** - VM for performance, interpreter for debugging
- **Comprehensive standard library** - 200+ functions across 18 modules
- **Modern tooling** - formatter, LSP (in progress), REPL with syntax highlighting

---

## Table of Contents

- [Quick Start](#quick-start)
- [Language Features](#language-features)
  - [Variables and Types](#variables-and-types)
  - [Functions](#functions)
  - [Control Flow](#control-flow)
  - [Collections](#collections)
  - [Error Handling](#error-handling)
  - [Modules](#modules)
- [Standard Library](#standard-library)
- [Architecture](#architecture)
- [Building](#building)
- [Testing](#testing)
- [Project Status](#project-status)
- [Documentation](#documentation)
- [License](#license)

---

## Quick Start

**Requirements:** Rust 1.70 or later

```bash
# Clone the repository
git clone https://github.com/proxikal/atlas.git
cd atlas

# Build in release mode
cargo build --release

# Run the REPL
./target/release/atlas

# Run a program
./target/release/atlas run examples/vm/fibonacci.atlas
```

**REPL example:**
```
atlas> let x = 42;
atlas> print(x * 2);
84
atlas> fn greet(name: string) -> string { return "Hello, " + name; }
atlas> print(greet("World"));
Hello, World
```

---

## Language Features

### Variables and Types

Atlas uses `let` for immutable bindings and `var` for mutable ones. Type annotations are optional when the type can be inferred.

```atlas
// Immutable (default)
let name = "Atlas";
let pi: number = 3.14159;

// Mutable
var counter = 0;
counter += 1;

// Primitive types
let n: number = 42;           // 64-bit float (IEEE 754)
let s: string = "hello";      // UTF-8 string
let b: bool = true;           // Boolean
let nothing: null = null;     // Explicit absence
```

### Functions

Functions are first-class values with explicit parameter and return types.

```atlas
// Basic function
fn add(a: number, b: number) -> number {
    return a + b;
}

// Generic function
fn identity<T>(x: T) -> T {
    return x;
}

// Higher-order function
fn apply(f: (number) -> number, x: number) -> number {
    return f(x);
}

// Nested functions
fn outer() -> number {
    fn helper(x: number) -> number {
        return x * 2;
    }
    return helper(21);  // Returns 42
}
```

### Control Flow

```atlas
// Conditionals (condition must be bool - no truthy/falsy)
if (x > 0) {
    print("positive");
} else {
    print("non-positive");
}

// While loop
var i = 0;
while (i < 10) {
    print(i);
    i += 1;
}

// For loop
for (var j = 0; j < 5; j++) {
    print(j);
}

// For-in loop
for item in [1, 2, 3, 4, 5] {
    print(item);
}

// Pattern matching
let result: Result<number, string> = Ok(42);
match result {
    Ok(value) => print("Success: " + str(value)),
    Err(error) => print("Error: " + error)
}
```

### Collections

Atlas provides five built-in collection types, all with thread-safe implementations.

```atlas
// Arrays - homogeneous, mutable
let numbers = [1, 2, 3, 4, 5];
push(numbers, 6);
let first = numbers[0];

// HashMap - key-value pairs
let config = hashmap_new();
hashmap_insert(config, "debug", true);
let debug = hashmap_get(config, "debug");

// HashSet - unique values
let seen = hashset_new();
hashset_insert(seen, "apple");
let has_apple = hashset_contains(seen, "apple");

// Queue - FIFO
let tasks = queue_new();
queue_push(tasks, "task1");
let next = queue_pop(tasks);

// Stack - LIFO
let history = stack_new();
stack_push(history, "page1");
let current = stack_pop(history);
```

### Error Handling

Atlas uses `Option<T>` and `Result<T, E>` for explicit error handling.

```atlas
// Option - represents optional values
let some_value: Option<number> = Some(42);
let no_value: Option<number> = None;

if (is_some(some_value)) {
    print(unwrap(some_value));
}

// Result - represents success or failure
fn divide(a: number, b: number) -> Result<number, string> {
    if (b == 0) {
        return Err("division by zero");
    }
    return Ok(a / b);
}

let result = divide(10, 2);
match result {
    Ok(value) => print(value),
    Err(msg) => print("Error: " + msg)
}
```

### Modules

Atlas supports a module system with explicit imports and exports.

```atlas
// math.atl
export fn add(a: number, b: number) -> number {
    return a + b;
}

export let PI = 3.14159;

// main.atl
import { add, PI } from "./math";

let sum = add(2, 3);
print(PI);

// Namespace import
import * as math from "./math";
print(math.add(1, 2));
```

---

## Standard Library

Atlas includes 200+ built-in functions organized into 18 modules:

| Module | Description | Example Functions |
|--------|-------------|-------------------|
| **math** | Mathematical operations | `abs`, `sqrt`, `pow`, `sin`, `cos`, `floor`, `ceil`, `round` |
| **string** | String manipulation | `len`, `substr`, `split`, `join`, `trim`, `upper`, `lower`, `replace` |
| **array** | Array operations | `push`, `pop`, `slice`, `map`, `filter`, `reduce`, `sort`, `reverse` |
| **datetime** | Date and time | `now`, `format_datetime`, `parse_datetime`, `add_days` |
| **regex** | Pattern matching | `regex_new`, `regex_match`, `regex_find_all`, `regex_replace` |
| **json** | JSON handling | `json_parse`, `json_stringify`, `json_get`, `json_set` |
| **http** | HTTP client | `http_get`, `http_post`, `http_request` |
| **fs** | File system | `read_file`, `write_file`, `file_exists`, `list_dir` |
| **path** | Path manipulation | `path_join`, `path_parent`, `path_filename`, `path_ext` |
| **io** | Input/output | `print`, `read_line`, `format` |
| **types** | Type checking | `type_of`, `is_number`, `is_string`, `is_array`, `is_some`, `is_ok` |
| **process** | Process control | `exit`, `env_get`, `env_set`, `exec` |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                           Atlas CLI                                  │
│                    (REPL, File Runner, Tools)                       │
├─────────────────────────────────────────────────────────────────────┤
│                         Atlas Runtime                                │
├──────────────────┬──────────────────┬───────────────────────────────┤
│     Frontend     │    Execution     │         Services              │
├──────────────────┼──────────────────┼───────────────────────────────┤
│  Lexer           │  Bytecode VM     │  Standard Library (200+ fns)  │
│  Parser          │  Interpreter     │  Module Resolver              │
│  Type Checker    │  (100% parity)   │  Error Reporter               │
│  Compiler        │                  │  Security Context             │
└──────────────────┴──────────────────┴───────────────────────────────┘
```

**Workspace structure:**

| Crate | Purpose |
|-------|---------|
| `atlas-runtime` | Core language: lexer, parser, type checker, compiler, VM, interpreter, stdlib |
| `atlas-cli` | Command-line interface with REPL, file runner, and developer tools |
| `atlas-formatter` | Code formatter with configurable style |
| `atlas-lsp` | Language Server Protocol implementation (in development) |
| `atlas-config` | Configuration system for projects |
| `atlas-build` | Build system for multi-file projects |
| `atlas-package` | Package manager (planned) |
| `atlas-jit` | JIT compilation foundation (planned) |

---

## Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Check without building
cargo check

# Format code
cargo fmt

# Lint
cargo clippy -- -D warnings
```

---

## Testing

Atlas uses a comprehensive testing strategy with 6,400+ tests:

```bash
# Run all tests (requires cargo-nextest)
cargo nextest run -p atlas-runtime

# Run specific test
cargo nextest run -p atlas-runtime -E 'test(test_name)'

# Run corpus tests (.atlas files)
cargo nextest run -p atlas-runtime --test corpus

# Run benchmarks
cargo bench -p atlas-runtime

# Run with coverage
cargo tarpaulin --all-features --workspace
```

**Test categories:**
- **Unit tests** - Individual function behavior
- **Integration tests** - Cross-component interaction
- **Corpus tests** - Real Atlas programs in `tests/corpus/`
- **Parity tests** - VM/Interpreter equivalence verification
- **Property tests** - Randomized input testing (proptest)
- **Fuzz tests** - Security and robustness (cargo-fuzz)
- **Snapshot tests** - Output regression (insta)

---

## Project Status

**Version:** 0.2 (Active Development)
**Progress:** 110/131 phases complete (84%)

| Category | Status | Description |
|----------|--------|-------------|
| Core Language | Complete | Lexer, parser, type checker, compiler |
| VM & Interpreter | Complete | Both engines with 100% parity |
| Standard Library | Complete | 200+ functions, 18 modules |
| Type System | Complete | Generics, Option, Result, inference |
| Module System | Complete | Import/export, namespace imports |
| CLI & REPL | In Progress | Basic functionality complete |
| LSP | Planned | Language server for IDE support |
| Package Manager | Planned | Dependency management |

---

## Project Metrics

| Metric | Value |
|--------|-------|
| Lines of Rust | 125,334 |
| Source Files | 274 |
| Test Count | 6,400+ |
| Corpus Tests | 56 |
| Example Programs | 40 |
| Stdlib Functions | 200+ |
| MSRV | Rust 1.70 |

---

## Documentation

| Resource | Location |
|----------|----------|
| Language Specification | [`docs/specification/`](docs/specification/) |
| Syntax Reference | [`docs/specification/syntax.md`](docs/specification/syntax.md) |
| Type System | [`docs/specification/types.md`](docs/specification/types.md) |
| Runtime Behavior | [`docs/specification/runtime.md`](docs/specification/runtime.md) |
| Implementation Status | [`STATUS.md`](STATUS.md) |

---

## License

Atlas is dual-licensed under MIT and Apache 2.0. See [LICENSE](LICENSE) for details.

---

<div align="center">

**Built with Rust. Designed for clarity.**

</div>
