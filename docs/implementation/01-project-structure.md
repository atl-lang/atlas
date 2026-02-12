# Project Structure

## Cargo Workspace Layout

```
atlas/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── atlas-runtime/      # Library crate (all language logic)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── span.rs         # Span and source location tracking
│   │       ├── diagnostic.rs   # Diagnostic system
│   │       ├── token.rs        # Token types
│   │       ├── lexer.rs        # Lexer implementation
│   │       ├── ast.rs          # AST types
│   │       ├── parser.rs       # Parser implementation
│   │       ├── symbol.rs       # Symbol table and binding
│   │       ├── types.rs        # Type system representation
│   │       ├── typechecker.rs  # Type checking
│   │       ├── value.rs        # Runtime value representation
│   │       ├── interpreter.rs  # Interpreter
│   │       ├── bytecode.rs     # Bytecode instruction set
│   │       ├── compiler.rs     # AST to bytecode compiler
│   │       ├── vm.rs           # Virtual machine
│   │       ├── stdlib.rs       # Standard library functions
│   │       └── repl.rs         # REPL core (UI-agnostic)
│   └── atlas-cli/          # Binary crate (CLI wrapper)
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── commands/
│           │   ├── mod.rs
│           │   ├── repl.rs     # REPL UI (rustyline)
│           │   ├── run.rs      # Run command
│           │   ├── build.rs    # Build command
│           │   └── check.rs    # Check command
│           └── ui.rs           # Terminal output formatting
└── tests/
    ├── lexer/
    ├── parser/
    ├── typecheck/
    ├── interpreter/
    ├── vm/
    └── e2e/
```

## Workspace Cargo.toml

```toml
[workspace]
members = [
    "crates/atlas-runtime",
    "crates/atlas-cli",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.70"
```

## Runtime Crate Dependencies

```toml
# crates/atlas-runtime/Cargo.toml
[package]
name = "atlas-runtime"
version.workspace = true
edition.workspace = true

[dependencies]
thiserror = "2.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[dev-dependencies]
insta = "1.40"  # Golden test snapshots
```

## CLI Crate Dependencies

```toml
# crates/atlas-cli/Cargo.toml
[package]
name = "atlas-cli"
version.workspace = true
edition.workspace = true

[[bin]]
name = "atlas"
path = "src/main.rs"

[dependencies]
atlas-runtime = { path = "../atlas-runtime" }
clap = { version = "4.5", features = ["derive"] }
rustyline = "14.0"  # Line editor for REPL
anyhow = "1.0"
```

## Library Exports (atlas-runtime/src/lib.rs)

```rust
// Public API surface
pub mod span;
pub mod diagnostic;
pub mod token;
pub mod lexer;
pub mod ast;
pub mod parser;
pub mod symbol;
pub mod types;
pub mod typechecker;
pub mod value;
pub mod interpreter;
pub mod bytecode;
pub mod compiler;
pub mod vm;
pub mod stdlib;
pub mod repl;

// Re-export commonly used types
pub use span::Span;
pub use diagnostic::{Diagnostic, DiagnosticLevel};
pub use token::{Token, TokenKind};
pub use lexer::Lexer;
pub use ast::*;
pub use parser::Parser;
pub use symbol::{Symbol, SymbolTable};
pub use types::Type;
pub use typechecker::TypeChecker;
pub use value::{Value, RuntimeError};
pub use interpreter::Interpreter;
pub use bytecode::{Bytecode, Opcode};
pub use compiler::Compiler;
pub use vm::VM;
pub use repl::{ReplCore, ReplResult};
```

## Design Principle

**atlas-runtime** is library-first:
- No CLI logic
- No terminal I/O (except stdlib `print`)
- All APIs return structured data (not formatted strings)
- Testable without spawning processes

**atlas-cli** is a thin wrapper:
- Clap for argument parsing
- Rustyline for REPL UI
- Terminal output formatting only
- No language logic

This separation allows:
- Unit testing runtime without CLI
- Future embedding in other applications
- Alternative frontends (TUI, LSP, web playground)
