//! Bytecode instruction set
//!
//! Stack-based bytecode with 30 opcodes organized by category.
//! Operands are encoded separately in the instruction stream.

use crate::span::Span;
use crate::value::Value;

/// Bytecode opcode (30 instructions)
///
/// Stack-based VM with explicit byte values for serialization.
/// Operands are encoded inline after the opcode byte.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    // ===== Constants (0x01-0x0F) =====
    /// Push constant from pool [u16 index]
    Constant = 0x01,
    /// Push null
    Null = 0x02,
    /// Push true
    True = 0x03,
    /// Push false
    False = 0x04,

    // ===== Variables (0x10-0x1F) =====
    /// Load local variable [u16 index]
    GetLocal = 0x10,
    /// Store to local variable [u16 index]
    SetLocal = 0x11,
    /// Load global variable [u16 name_index]
    GetGlobal = 0x12,
    /// Store to global variable [u16 name_index]
    SetGlobal = 0x13,

    // ===== Arithmetic (0x20-0x2F) =====
    /// Pop b, pop a, push a + b
    Add = 0x20,
    /// Pop b, pop a, push a - b
    Sub = 0x21,
    /// Pop b, pop a, push a * b
    Mul = 0x22,
    /// Pop b, pop a, push a / b
    Div = 0x23,
    /// Pop b, pop a, push a % b
    Mod = 0x24,
    /// Pop a, push -a
    Negate = 0x25,

    // ===== Comparison (0x30-0x3F) =====
    /// Pop b, pop a, push a == b
    Equal = 0x30,
    /// Pop b, pop a, push a != b
    NotEqual = 0x31,
    /// Pop b, pop a, push a < b
    Less = 0x32,
    /// Pop b, pop a, push a <= b
    LessEqual = 0x33,
    /// Pop b, pop a, push a > b
    Greater = 0x34,
    /// Pop b, pop a, push a >= b
    GreaterEqual = 0x35,

    // ===== Logical (0x40-0x4F) =====
    /// Pop a, push !a
    Not = 0x40,
    /// Short-circuit: if TOS is false, skip next instruction
    And = 0x41,
    /// Short-circuit: if TOS is true, skip next instruction
    Or = 0x42,

    // ===== Control flow (0x50-0x5F) =====
    /// Unconditional jump [i16 offset]
    Jump = 0x50,
    /// Pop condition, jump if false [i16 offset]
    JumpIfFalse = 0x51,
    /// Jump backward [i16 offset]
    Loop = 0x52,

    // ===== Functions (0x60-0x6F) =====
    /// Call function [u8 arg_count]
    Call = 0x60,
    /// Return from function
    Return = 0x61,

    // ===== Arrays (0x70-0x7F) =====
    /// Create array [u16 size] from stack
    Array = 0x70,
    /// Pop index, pop array, push array[index]
    GetIndex = 0x71,
    /// Pop value, pop index, pop array, array[index] = value
    SetIndex = 0x72,

    // ===== Stack manipulation (0x80-0x8F) =====
    /// Pop and discard top of stack
    Pop = 0x80,
    /// Duplicate top of stack
    Dup = 0x81,

    // ===== Special (0xF0-0xFF) =====
    /// End of bytecode
    Halt = 0xFF,
}

impl TryFrom<u8> for Opcode {
    type Error = ();

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            0x01 => Ok(Opcode::Constant),
            0x02 => Ok(Opcode::Null),
            0x03 => Ok(Opcode::True),
            0x04 => Ok(Opcode::False),
            0x10 => Ok(Opcode::GetLocal),
            0x11 => Ok(Opcode::SetLocal),
            0x12 => Ok(Opcode::GetGlobal),
            0x13 => Ok(Opcode::SetGlobal),
            0x20 => Ok(Opcode::Add),
            0x21 => Ok(Opcode::Sub),
            0x22 => Ok(Opcode::Mul),
            0x23 => Ok(Opcode::Div),
            0x24 => Ok(Opcode::Mod),
            0x25 => Ok(Opcode::Negate),
            0x30 => Ok(Opcode::Equal),
            0x31 => Ok(Opcode::NotEqual),
            0x32 => Ok(Opcode::Less),
            0x33 => Ok(Opcode::LessEqual),
            0x34 => Ok(Opcode::Greater),
            0x35 => Ok(Opcode::GreaterEqual),
            0x40 => Ok(Opcode::Not),
            0x41 => Ok(Opcode::And),
            0x42 => Ok(Opcode::Or),
            0x50 => Ok(Opcode::Jump),
            0x51 => Ok(Opcode::JumpIfFalse),
            0x52 => Ok(Opcode::Loop),
            0x60 => Ok(Opcode::Call),
            0x61 => Ok(Opcode::Return),
            0x70 => Ok(Opcode::Array),
            0x71 => Ok(Opcode::GetIndex),
            0x72 => Ok(Opcode::SetIndex),
            0x80 => Ok(Opcode::Pop),
            0x81 => Ok(Opcode::Dup),
            0xFF => Ok(Opcode::Halt),
            _ => Err(()),
        }
    }
}

/// Debug information for bytecode
///
/// Maps instruction offsets to source spans for error reporting
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugSpan {
    /// Byte offset of instruction in bytecode
    pub instruction_offset: usize,
    /// Source span for this instruction
    pub span: Span,
}

/// Bytecode container
///
/// Contains raw instruction bytes, constant pool, and debug information.
/// Instructions are encoded as:
/// - Opcode (1 byte)
/// - Operands (variable, depending on opcode)
#[derive(Debug, Clone)]
pub struct Bytecode {
    /// Raw instruction bytes
    pub instructions: Vec<u8>,
    /// Constant pool (referenced by index)
    pub constants: Vec<Value>,
    /// Debug information (instruction offset -> source span)
    pub debug_info: Vec<DebugSpan>,
}

impl Bytecode {
    /// Create a new empty bytecode container
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
            debug_info: Vec::new(),
        }
    }

    /// Emit an opcode and track debug information
    pub fn emit(&mut self, opcode: Opcode, span: Span) {
        self.debug_info.push(DebugSpan {
            instruction_offset: self.instructions.len(),
            span,
        });
        self.instructions.push(opcode as u8);
    }

    /// Emit a single byte operand
    pub fn emit_u8(&mut self, byte: u8) {
        self.instructions.push(byte);
    }

    /// Emit a u16 operand (big-endian)
    pub fn emit_u16(&mut self, value: u16) {
        self.instructions.push((value >> 8) as u8);
        self.instructions.push((value & 0xFF) as u8);
    }

    /// Emit an i16 operand (big-endian, signed)
    pub fn emit_i16(&mut self, value: i16) {
        self.emit_u16(value as u16);
    }

    /// Add a constant to the pool and return its index
    pub fn add_constant(&mut self, value: Value) -> u16 {
        self.constants.push(value);
        (self.constants.len() - 1) as u16
    }

    /// Get current instruction offset (for jump targets)
    pub fn current_offset(&self) -> usize {
        self.instructions.len()
    }

    /// Patch a jump instruction with the correct offset
    ///
    /// Used for forward jumps where the target isn't known yet
    pub fn patch_jump(&mut self, offset: usize) {
        let jump = (self.instructions.len() - offset - 2) as i16;
        self.instructions[offset] = ((jump >> 8) & 0xFF) as u8;
        self.instructions[offset + 1] = (jump & 0xFF) as u8;
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
    fn test_opcode_to_u8() {
        assert_eq!(Opcode::Constant as u8, 0x01);
        assert_eq!(Opcode::Null as u8, 0x02);
        assert_eq!(Opcode::Add as u8, 0x20);
        assert_eq!(Opcode::Jump as u8, 0x50);
        assert_eq!(Opcode::Halt as u8, 0xFF);
    }

    #[test]
    fn test_opcode_from_u8() {
        assert_eq!(Opcode::try_from(0x01), Ok(Opcode::Constant));
        assert_eq!(Opcode::try_from(0x02), Ok(Opcode::Null));
        assert_eq!(Opcode::try_from(0x20), Ok(Opcode::Add));
        assert_eq!(Opcode::try_from(0x50), Ok(Opcode::Jump));
        assert_eq!(Opcode::try_from(0xFF), Ok(Opcode::Halt));
        assert_eq!(Opcode::try_from(0x99), Err(())); // Invalid opcode
    }

    #[test]
    fn test_all_opcodes_roundtrip() {
        let opcodes = vec![
            Opcode::Constant,
            Opcode::Null,
            Opcode::True,
            Opcode::False,
            Opcode::GetLocal,
            Opcode::SetLocal,
            Opcode::GetGlobal,
            Opcode::SetGlobal,
            Opcode::Add,
            Opcode::Sub,
            Opcode::Mul,
            Opcode::Div,
            Opcode::Mod,
            Opcode::Negate,
            Opcode::Equal,
            Opcode::NotEqual,
            Opcode::Less,
            Opcode::LessEqual,
            Opcode::Greater,
            Opcode::GreaterEqual,
            Opcode::Not,
            Opcode::And,
            Opcode::Or,
            Opcode::Jump,
            Opcode::JumpIfFalse,
            Opcode::Loop,
            Opcode::Call,
            Opcode::Return,
            Opcode::Array,
            Opcode::GetIndex,
            Opcode::SetIndex,
            Opcode::Pop,
            Opcode::Dup,
            Opcode::Halt,
        ];

        for opcode in opcodes {
            let byte = opcode as u8;
            let decoded = Opcode::try_from(byte).unwrap();
            assert_eq!(opcode, decoded);
        }
    }

    #[test]
    fn test_bytecode_creation() {
        let bytecode = Bytecode::new();
        assert_eq!(bytecode.instructions.len(), 0);
        assert_eq!(bytecode.constants.len(), 0);
        assert_eq!(bytecode.debug_info.len(), 0);
    }

    #[test]
    fn test_emit_opcode() {
        let mut bytecode = Bytecode::new();
        bytecode.emit(Opcode::Null, Span::dummy());
        assert_eq!(bytecode.instructions.len(), 1);
        assert_eq!(bytecode.instructions[0], 0x02); // Null opcode
        assert_eq!(bytecode.debug_info.len(), 1);
        assert_eq!(bytecode.debug_info[0].instruction_offset, 0);
    }

    #[test]
    fn test_emit_u8() {
        let mut bytecode = Bytecode::new();
        bytecode.emit(Opcode::Call, Span::dummy());
        bytecode.emit_u8(3); // 3 arguments
        assert_eq!(bytecode.instructions.len(), 2);
        assert_eq!(bytecode.instructions[0], 0x60); // Call opcode
        assert_eq!(bytecode.instructions[1], 3);
    }

    #[test]
    fn test_emit_u16_big_endian() {
        let mut bytecode = Bytecode::new();
        bytecode.emit(Opcode::Constant, Span::dummy());
        bytecode.emit_u16(0x1234);
        assert_eq!(bytecode.instructions.len(), 3);
        assert_eq!(bytecode.instructions[0], 0x01); // Constant opcode
        assert_eq!(bytecode.instructions[1], 0x12); // High byte
        assert_eq!(bytecode.instructions[2], 0x34); // Low byte
    }

    #[test]
    fn test_emit_i16() {
        let mut bytecode = Bytecode::new();
        bytecode.emit(Opcode::Jump, Span::dummy());
        bytecode.emit_i16(100);
        assert_eq!(bytecode.instructions.len(), 3);
        assert_eq!(bytecode.instructions[0], 0x50); // Jump opcode
        // 100 as i16 -> u16 -> bytes
        assert_eq!(bytecode.instructions[1], 0x00);
        assert_eq!(bytecode.instructions[2], 0x64);
    }

    #[test]
    fn test_constant_pool() {
        use std::rc::Rc;
        let mut bytecode = Bytecode::new();
        let idx1 = bytecode.add_constant(Value::Number(42.0));
        let idx2 = bytecode.add_constant(Value::String(Rc::new("hello".to_string())));
        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(bytecode.constants.len(), 2);
        assert_eq!(bytecode.constants[0], Value::Number(42.0));
    }

    #[test]
    fn test_current_offset() {
        let mut bytecode = Bytecode::new();
        assert_eq!(bytecode.current_offset(), 0);
        bytecode.emit(Opcode::Null, Span::dummy());
        assert_eq!(bytecode.current_offset(), 1);
        bytecode.emit_u16(0x1234);
        assert_eq!(bytecode.current_offset(), 3);
    }

    #[test]
    fn test_patch_jump() {
        let mut bytecode = Bytecode::new();
        bytecode.emit(Opcode::JumpIfFalse, Span::dummy());
        let jump_offset = bytecode.current_offset();
        bytecode.emit_u16(0xFFFF); // Placeholder
        bytecode.emit(Opcode::Null, Span::dummy());
        bytecode.emit(Opcode::Null, Span::dummy());

        // Patch the jump to point to current position
        bytecode.patch_jump(jump_offset);

        // Jump should now be 2 (from offset+2 to current position)
        let jump = ((bytecode.instructions[jump_offset] as i16) << 8)
            | (bytecode.instructions[jump_offset + 1] as i16);
        assert_eq!(jump, 2);
    }

    #[test]
    fn test_debug_info_tracking() {
        let mut bytecode = Bytecode::new();
        let span1 = Span::new(0, 10);
        let span2 = Span::new(10, 20);

        bytecode.emit(Opcode::Constant, span1);
        bytecode.emit_u16(0);
        bytecode.emit(Opcode::Constant, span2);
        bytecode.emit_u16(1);

        assert_eq!(bytecode.debug_info.len(), 2);
        assert_eq!(bytecode.debug_info[0].instruction_offset, 0);
        assert_eq!(bytecode.debug_info[0].span, span1);
        assert_eq!(bytecode.debug_info[1].instruction_offset, 3);
        assert_eq!(bytecode.debug_info[1].span, span2);
    }

    #[test]
    fn test_bytecode_sequence() {
        // Test: let x = 2 + 3;
        // Bytecode:
        //   Constant 0  (push 2.0)
        //   Constant 1  (push 3.0)
        //   Add         (pop both, push 5.0)
        //   SetLocal 0  (store to x)

        let mut bytecode = Bytecode::new();
        let idx_2 = bytecode.add_constant(Value::Number(2.0));
        let idx_3 = bytecode.add_constant(Value::Number(3.0));

        bytecode.emit(Opcode::Constant, Span::dummy());
        bytecode.emit_u16(idx_2);
        bytecode.emit(Opcode::Constant, Span::dummy());
        bytecode.emit_u16(idx_3);
        bytecode.emit(Opcode::Add, Span::dummy());
        bytecode.emit(Opcode::SetLocal, Span::dummy());
        bytecode.emit_u16(0); // Local index 0

        assert_eq!(bytecode.instructions.len(), 10);
        assert_eq!(bytecode.constants.len(), 2);
        assert_eq!(bytecode.debug_info.len(), 4);
    }
}
