//! Optimized instruction dispatch for the VM
//!
//! Uses a static lookup table for O(1) opcode decoding instead of
//! match-based dispatch, reducing branch mispredictions in the hot loop.

use crate::bytecode::Opcode;

/// Static dispatch table mapping byte values to optional Opcodes.
/// Indexed by the raw u8 opcode byte for O(1) lookup.
static OPCODE_TABLE: [Option<Opcode>; 256] = {
    let mut table: [Option<Opcode>; 256] = [None; 256];

    // Constants (0x01-0x04)
    table[0x01] = Some(Opcode::Constant);
    table[0x02] = Some(Opcode::Null);
    table[0x03] = Some(Opcode::True);
    table[0x04] = Some(Opcode::False);

    // Variables (0x10-0x16)
    table[0x10] = Some(Opcode::GetLocal);
    table[0x11] = Some(Opcode::SetLocal);
    table[0x12] = Some(Opcode::GetGlobal);
    table[0x13] = Some(Opcode::SetGlobal);
    table[0x14] = Some(Opcode::MakeClosure);
    table[0x15] = Some(Opcode::GetUpvalue);
    table[0x16] = Some(Opcode::SetUpvalue);

    // Arithmetic (0x20-0x25)
    table[0x20] = Some(Opcode::Add);
    table[0x21] = Some(Opcode::Sub);
    table[0x22] = Some(Opcode::Mul);
    table[0x23] = Some(Opcode::Div);
    table[0x24] = Some(Opcode::Mod);
    table[0x25] = Some(Opcode::Negate);

    // Comparison (0x30-0x35)
    table[0x30] = Some(Opcode::Equal);
    table[0x31] = Some(Opcode::NotEqual);
    table[0x32] = Some(Opcode::Less);
    table[0x33] = Some(Opcode::LessEqual);
    table[0x34] = Some(Opcode::Greater);
    table[0x35] = Some(Opcode::GreaterEqual);

    // Logical (0x40-0x42)
    table[0x40] = Some(Opcode::Not);
    table[0x41] = Some(Opcode::And);
    table[0x42] = Some(Opcode::Or);

    // Control flow (0x50-0x52)
    table[0x50] = Some(Opcode::Jump);
    table[0x51] = Some(Opcode::JumpIfFalse);
    table[0x52] = Some(Opcode::Loop);

    // Functions (0x60-0x6F)
    table[0x60] = Some(Opcode::Call);
    table[0x61] = Some(Opcode::Return);
    table[0x62] = Some(Opcode::TraitDispatch);

    // Arrays (0x70-0x7A)
    table[0x70] = Some(Opcode::Array);
    table[0x71] = Some(Opcode::GetIndex);
    table[0x72] = Some(Opcode::SetIndex);
    table[0x73] = Some(Opcode::HashMap);
    table[0x74] = Some(Opcode::Slice);
    table[0x75] = Some(Opcode::SliceFrom);
    table[0x76] = Some(Opcode::SliceTo);
    table[0x77] = Some(Opcode::SliceFull);
    table[0x78] = Some(Opcode::GetField);
    table[0x79] = Some(Opcode::SetField);
    table[0x7A] = Some(Opcode::Range);
    table[0x7B] = Some(Opcode::Struct);
    table[0x7C] = Some(Opcode::Tuple);
    table[0x7D] = Some(Opcode::TupleGet);

    // Stack manipulation (0x80-0x83)
    table[0x80] = Some(Opcode::Pop);
    table[0x81] = Some(Opcode::Dup);
    table[0x82] = Some(Opcode::Dup2);
    table[0x83] = Some(Opcode::Rot3);
    table[0x84] = Some(Opcode::ToString);

    // Pattern matching (0x90-0x97)
    table[0x90] = Some(Opcode::IsOptionSome);
    table[0x91] = Some(Opcode::IsOptionNone);
    table[0x92] = Some(Opcode::IsResultOk);
    table[0x93] = Some(Opcode::IsResultErr);
    table[0x94] = Some(Opcode::ExtractOptionValue);
    table[0x95] = Some(Opcode::ExtractResultValue);
    table[0x96] = Some(Opcode::IsArray);
    table[0x97] = Some(Opcode::GetArrayLen);

    // Async (0xA0-0xA3)
    table[0xA0] = Some(Opcode::AsyncCall);
    table[0xA1] = Some(Opcode::Await);
    table[0xA2] = Some(Opcode::WrapFuture);
    table[0xA3] = Some(Opcode::SpawnTask);

    // Special
    table[0xFF] = Some(Opcode::Halt);

    table
};

/// Decode an opcode byte using the static lookup table.
/// Returns None for invalid opcode bytes.
#[inline(always)]
pub fn decode_opcode(byte: u8) -> Option<Opcode> {
    // SAFETY: `byte` is an opcode byte from the bytecode stream.
    // Preconditions: the compiler emits only valid opcodes and the table
    // length matches the full opcode range, so the index is in-bounds.
    unsafe { *OPCODE_TABLE.get_unchecked(byte as usize) }
}

/// Returns the number of operand bytes following an opcode.
/// Used for instruction-length-aware operations (disassembly, skipping).
#[inline(always)]
pub fn operand_size(opcode: Opcode) -> usize {
    match opcode {
        // u16 operand
        Opcode::Constant
        | Opcode::GetLocal
        | Opcode::SetLocal
        | Opcode::GetGlobal
        | Opcode::SetGlobal
        | Opcode::GetUpvalue
        | Opcode::SetUpvalue
        | Opcode::Array => 2,
        // MakeClosure: two u16 operands (func_const_idx, n_upvalues) = 4 bytes
        Opcode::MakeClosure => 4,
        // i16 operand
        Opcode::Jump | Opcode::JumpIfFalse | Opcode::Loop => 2,
        // u16 + u16 + u8 operand
        Opcode::TraitDispatch => 5,
        // u8 operand
        Opcode::Call => 1,
        // u16 + u16 operand
        Opcode::Struct => 4,
        // No operand
        _ => 0,
    }
}
