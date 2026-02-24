//! Bytecode instruction set
//!
//! Stack-based bytecode with 30 opcodes organized by category.
//! Operands are encoded separately in the instruction stream.

mod disasm;
mod opcode;
mod optimizer;
mod serialize;
pub mod validator;

pub use disasm::disassemble;
pub use opcode::Opcode;
pub use optimizer::{
    ConstantFoldingPass, DeadCodeEliminationPass, OptimizationPass, OptimizationStats, Optimizer,
    PeepholePass,
};
use serialize::{deserialize_span, deserialize_value, serialize_span, serialize_value};
pub use validator::{validate, ValidationError, ValidationErrorKind};

use crate::span::Span;
use crate::value::Value;

/// Current bytecode format version
///
/// This version is incremented when the bytecode format changes in a
/// backward-incompatible way. The VM will reject bytecode files with
/// different version numbers to prevent runtime errors from format mismatches.
///
/// Version history:
/// - Version 1: Initial bytecode format (Phase 10)
pub const BYTECODE_VERSION: u16 = 1;

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
    /// Number of local slots required by top-level code (set by compiler).
    /// Used by the VM to initialize the main frame's local_count so that
    /// SetLocal in top-level for-in loops and other constructs works correctly.
    pub top_level_local_count: usize,
}

impl Bytecode {
    /// Create a new empty bytecode container
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
            debug_info: Vec::new(),
            top_level_local_count: 0,
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

    /// Look up the source span for a given instruction offset
    ///
    /// Returns the span of the instruction at or before the given offset.
    /// This is useful for error reporting in the VM.
    pub fn get_span_for_offset(&self, offset: usize) -> Option<Span> {
        // Find the most recent debug info entry at or before the offset
        self.debug_info
            .iter()
            .rev()
            .find(|debug_span| debug_span.instruction_offset <= offset)
            .map(|debug_span| debug_span.span)
    }

    /// Serialize bytecode to binary format (.atb file)
    ///
    /// Format:
    /// - Header: Magic "ATB\0" + version u16 + flags u16
    /// - Constants: count u32 + serialized values
    /// - Instructions: length u32 + bytecode bytes
    /// - Debug info (optional): count u32 + debug spans
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Header
        bytes.extend_from_slice(b"ATB\0"); // Magic number
        bytes.extend_from_slice(&BYTECODE_VERSION.to_be_bytes()); // Version
        let flags = if self.debug_info.is_empty() {
            0u16
        } else {
            1u16
        };
        bytes.extend_from_slice(&flags.to_be_bytes()); // Flags

        // Constants section
        bytes.extend_from_slice(&(self.constants.len() as u32).to_be_bytes());
        for value in &self.constants {
            serialize_value(value, &mut bytes);
        }

        // Instructions section
        bytes.extend_from_slice(&(self.instructions.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&self.instructions);

        // Debug info section (optional)
        if !self.debug_info.is_empty() {
            bytes.extend_from_slice(&(self.debug_info.len() as u32).to_be_bytes());
            for debug_span in &self.debug_info {
                bytes.extend_from_slice(&(debug_span.instruction_offset as u32).to_be_bytes());
                serialize_span(&debug_span.span, &mut bytes);
            }
        }

        bytes
    }

    /// Deserialize bytecode from binary format (.atb file)
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        // Read and validate header
        if bytes.len() < 8 {
            return Err("Invalid bytecode file: too short".to_string());
        }
        if &bytes[0..4] != b"ATB\0" {
            return Err("Invalid bytecode file: bad magic number. Expected 'ATB\\0', this may not be an Atlas bytecode file.".to_string());
        }
        let version = u16::from_be_bytes([bytes[4], bytes[5]]);
        if version != BYTECODE_VERSION {
            return Err(format!(
                "Bytecode version mismatch: file has version {}, but this VM supports version {}. \
                 Recompile the source file with the current Atlas compiler.",
                version, BYTECODE_VERSION
            ));
        }
        let flags = u16::from_be_bytes([bytes[6], bytes[7]]);
        let has_debug_info = (flags & 1) != 0;

        // Start reading sections after header (8 bytes)
        let mut offset = 8;

        // Read constants
        if offset + 4 > bytes.len() {
            return Err("Invalid bytecode: constants section truncated".to_string());
        }
        let const_count = u32::from_be_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]) as usize;
        offset += 4;

        let mut constants = Vec::with_capacity(const_count);
        for _ in 0..const_count {
            let (value, consumed) = deserialize_value(&bytes[offset..])?;
            constants.push(value);
            offset += consumed;
        }

        // Read instructions
        if offset + 4 > bytes.len() {
            return Err("Invalid bytecode: instructions section truncated".to_string());
        }
        let instr_len = u32::from_be_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]) as usize;
        offset += 4;

        if offset + instr_len > bytes.len() {
            return Err("Invalid bytecode: instructions data truncated".to_string());
        }
        let instructions = bytes[offset..offset + instr_len].to_vec();
        offset += instr_len;

        // Read debug info (optional)
        let mut debug_info = Vec::new();
        if has_debug_info {
            if offset + 4 > bytes.len() {
                return Err("Invalid bytecode: debug info section truncated".to_string());
            }
            let debug_count = u32::from_be_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ]) as usize;
            offset += 4;

            for _ in 0..debug_count {
                if offset + 4 > bytes.len() {
                    return Err("Invalid bytecode: debug span truncated".to_string());
                }
                let instruction_offset = u32::from_be_bytes([
                    bytes[offset],
                    bytes[offset + 1],
                    bytes[offset + 2],
                    bytes[offset + 3],
                ]) as usize;
                offset += 4;

                let (span, consumed) = deserialize_span(&bytes[offset..])?;
                debug_info.push(DebugSpan {
                    instruction_offset,
                    span,
                });
                offset += consumed;
            }
        }

        // Verify we consumed exactly the expected amount of data
        if offset != bytes.len() {
            return Err(format!(
                "Invalid bytecode: expected {} bytes, but only consumed {}",
                bytes.len(),
                offset
            ));
        }

        Ok(Bytecode {
            instructions,
            constants,
            debug_info,
            top_level_local_count: 0,
        })
    }

    /// Append another bytecode chunk to this one
    ///
    /// This adjusts:
    /// - Instruction offsets in debug info
    /// - Bytecode offsets in Function values in constants
    /// - Constant indices in the new instructions (opcodes that reference constants)
    ///
    /// Used by Runtime to accumulate bytecode across multiple eval() calls.
    pub fn append(&mut self, other: Bytecode) {
        let instruction_offset = self.instructions.len();
        let constant_offset = self.constants.len() as u16;

        // Append constants FIRST, adjusting function bytecode offsets
        for constant in other.constants {
            match constant {
                Value::Function(mut func_ref) => {
                    // Adjust bytecode offset to account for accumulated instructions
                    if func_ref.bytecode_offset > 0 {
                        func_ref.bytecode_offset += instruction_offset;
                    }
                    self.constants.push(Value::Function(func_ref));
                }
                other_value => {
                    self.constants.push(other_value);
                }
            }
        }

        // Append instructions, adjusting constant indices in opcodes that use them
        let mut i = 0;
        while i < other.instructions.len() {
            let opcode_byte = other.instructions[i];
            self.instructions.push(opcode_byte);
            i += 1;

            // Check if this opcode uses a constant index (u16 operand)
            let uses_constant = matches!(
                opcode_byte,
                x if x == Opcode::Constant as u8
                    || x == Opcode::GetGlobal as u8
                    || x == Opcode::SetGlobal as u8
            );

            if uses_constant && i + 1 < other.instructions.len() {
                // Read the u16 constant index
                let high = other.instructions[i] as u16;
                let low = other.instructions[i + 1] as u16;
                let old_index = (high << 8) | low;

                // Adjust by constant_offset
                let new_index = old_index + constant_offset;

                // Write adjusted index
                self.instructions.push((new_index >> 8) as u8);
                self.instructions.push((new_index & 0xFF) as u8);
                i += 2;
            } else if uses_constant {
                // Malformed bytecode, but continue
                while i < other.instructions.len() && i < 2 {
                    self.instructions.push(other.instructions[i]);
                    i += 1;
                }
            } else {
                // Check opcode operand size and copy remaining bytes
                // Most opcodes have known operand sizes
                let operand_size = match opcode_byte {
                    x if x == Opcode::Jump as u8
                        || x == Opcode::JumpIfFalse as u8
                        || x == Opcode::GetLocal as u8
                        || x == Opcode::SetLocal as u8
                        || x == Opcode::Array as u8 =>
                    {
                        2 // u16 operand
                    }
                    x if x == Opcode::Call as u8 => 1, // u8 operand
                    _ => 0,                            // No operand
                };

                for _ in 0..operand_size {
                    if i < other.instructions.len() {
                        self.instructions.push(other.instructions[i]);
                        i += 1;
                    }
                }
            }
        }

        // Append debug info, adjusting instruction offsets
        for mut debug_span in other.debug_info {
            debug_span.instruction_offset += instruction_offset;
            self.debug_info.push(debug_span);
        }
    }
}

impl Default for Bytecode {
    fn default() -> Self {
        Self::new()
    }
}
