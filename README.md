# Atlas

A statically-typed programming language with a bytecode virtual machine, written in Rust.

---

## What is Atlas?

Atlas is an experimental programming language being developed as a long-term research project. It features:

- **Static type system** with type inference
- **Bytecode compiler and virtual machine** for execution
- **Tree-walking interpreter** with full VM parity
- **Hand-written lexer and parser** (no parser generators)
- **Interactive REPL** for development
- **Comprehensive standard library** with collections, I/O, math, and string operations

The language emphasizes strictness and explicitness: no implicit type coercion, no truthy/falsy values, and clear error messages with source locations.

---

## Current State

**Status:** Active development (v0.2)
**Progress:** ~80% through planned phases
**Maturity:** Experimental - not production-ready

The core language works: you can write functions, use control flow, work with arrays and hashmaps, import modules, and run programs through either the interpreter or VM. The standard library covers most common operations.

What's still in progress: module system refinement, LSP support, CLI tooling, and polish phases.

---

## Language Overview

```atlas
// Variables (immutable by default)
let name = "Atlas";
var counter = 0;

// Functions with type annotations
fn greet(name: string) -> string {
    return "Hello, " + name + "!";
}

// Control flow
if counter == 0 {
    print(greet(name));
}

// Collections
let numbers = [1, 2, 3, 4, 5];
let config = { "debug": true, "version": 1 };

// Higher-order functions
let doubled = map(numbers, fn(x: number) -> number { return x * 2; });
```

### Types

- **Primitives:** `number` (f64), `string`, `bool`, `null`, `void`
- **Collections:** `Array<T>`, `HashMap<K, V>`, `HashSet<T>`, `Queue<T>`, `Stack<T>`
- **Functions:** First-class, with syntax `(T1, T2) -> R`
- **Generics:** `Option<T>`, `Result<T, E>` (in progress)

### Features

- Pattern matching with `match`
- Module system with `import`/`export`
- Immutability by default (`let` vs `var`)
- No implicit conversions
- Detailed error diagnostics with spans

---

## Building

**Requirements:** Rust 1.70+

```bash
# Build
cargo build --release

# Run tests
cargo test

# Run the CLI
cargo run --bin atlas -- --help

# Start the REPL
cargo run --bin atlas
```

---

## Project Structure

```
crates/
├── atlas-runtime/    # Core: lexer, parser, compiler, VM, interpreter, stdlib
├── atlas-cli/        # Command-line interface
├── atlas-lsp/        # Language server (in development)
├── atlas-formatter/  # Code formatter
├── atlas-build/      # Build system
├── atlas-package/    # Package manager
├── atlas-config/     # Configuration
└── atlas-jit/        # JIT compilation foundation
```

---

## Documentation

- **Language Specification:** `docs/specification/` - syntax, types, runtime behavior
- **Implementation Status:** `STATUS.md` - current progress and next steps
- **Architecture:** `docs/` - design documents and decisions

---

## Development

This is a research project built with quality as the primary goal. The codebase is developed incrementally through documented phases, with comprehensive testing at each step.

**Testing:**
```bash
cargo nextest run -p atlas-runtime    # Fast parallel tests
cargo clippy -- -D warnings           # Linting
cargo fmt --check                     # Formatting
```

See `CONTRIBUTING.md` for development guidelines.

---

## License

MIT License - see `LICENSE` for details.
