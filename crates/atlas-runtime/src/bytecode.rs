//! Bytecode instruction set

use serde::{Deserialize, Serialize};

/// Bytecode instruction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Opcode {
    /// Push constant onto stack
    Constant(u16),
    /// Push null onto stack
    Null,
    /// Push true onto stack
    True,
    /// Push false onto stack
    False,
    /// Add top two stack values
    Add,
    /// Subtract top two stack values
    Subtract,
    /// Multiply top two stack values
    Multiply,
    /// Divide top two stack values
    Divide,
    /// Return from function
    Return,
}

/// Bytecode chunk (sequence of instructions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bytecode {
    /// Instructions
    pub instructions: Vec<Opcode>,
    /// Constant pool
    pub constants: Vec<String>,
}

impl Bytecode {
    /// Create a new empty bytecode chunk
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
        }
    }

    /// Add an instruction
    pub fn emit(&mut self, opcode: Opcode) {
        self.instructions.push(opcode);
    }

    /// Add a constant and return its index
    pub fn add_constant(&mut self, value: String) -> u16 {
        self.constants.push(value);
        (self.constants.len() - 1) as u16
    }
}

impl Default for Bytecode {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytecode_creation() {
        let mut bytecode = Bytecode::new();
        bytecode.emit(Opcode::Null);
        assert_eq!(bytecode.instructions.len(), 1);
    }

    #[test]
    fn test_constant_pool() {
        let mut bytecode = Bytecode::new();
        let idx = bytecode.add_constant("hello".to_string());
        assert_eq!(idx, 0);
        assert_eq!(bytecode.constants[0], "hello");
    }
}
