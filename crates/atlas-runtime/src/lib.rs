//! Atlas Runtime - Core language implementation
//!
//! This library provides the complete Atlas language runtime including:
//! - Lexical analysis and parsing
//! - Type checking and binding
//! - Interpretation and bytecode compilation
//! - Standard library functions

/// Atlas runtime version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// Public API modules
pub mod ast;
pub mod bytecode;
pub mod compiler;
pub mod diagnostic;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod repl;
pub mod runtime;
pub mod span;
pub mod stdlib;
pub mod symbol;
pub mod token;
pub mod typechecker;
pub mod types;
pub mod value;
pub mod vm;

// Test utilities (only available in test builds)
#[cfg(test)]
pub mod test_utils;

// Re-export commonly used types
pub use bytecode::{Bytecode, Opcode};
pub use compiler::Compiler;
pub use diagnostic::{
    error_codes, normalizer, sort_diagnostics, Diagnostic, DiagnosticLevel, RelatedLocation,
    DIAG_VERSION,
};
pub use interpreter::Interpreter;
pub use lexer::Lexer;
pub use parser::Parser;
pub use repl::{ReplCore, ReplResult};
pub use runtime::{Atlas, RuntimeResult};
pub use span::Span;
pub use symbol::{Symbol, SymbolTable};
pub use token::{Token, TokenKind};
pub use typechecker::TypeChecker;
pub use types::Type;
pub use value::{RuntimeError, Value};
pub use vm::VM;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smoke() {
        // Smoke test to verify the crate builds and tests run
        assert_eq!(VERSION, "0.1.0");
    }
}
