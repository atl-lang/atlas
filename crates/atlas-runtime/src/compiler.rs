//! AST to bytecode compiler

use crate::ast::Program;
use crate::bytecode::Bytecode;
use crate::diagnostic::Diagnostic;

/// Compiler state
pub struct Compiler {
    _placeholder: (),
}

impl Compiler {
    /// Create a new compiler
    pub fn new() -> Self {
        Self { _placeholder: () }
    }

    /// Compile an AST to bytecode
    pub fn compile(&mut self, _program: &Program) -> Result<Bytecode, Vec<Diagnostic>> {
        // Placeholder implementation
        Ok(Bytecode::new())
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Program;

    #[test]
    fn test_compiler_creation() {
        let mut compiler = Compiler::new();
        let program = Program { items: Vec::new() };
        let bytecode = compiler.compile(&program).unwrap();
        assert_eq!(bytecode.instructions.len(), 0);
    }
}
