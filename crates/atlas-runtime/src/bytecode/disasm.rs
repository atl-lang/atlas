//! Bytecode disassembler
//!
//! Converts bytecode back to human-readable assembly-like format.
//! Used for debugging, testing, and `atlas build --disasm` output.

use super::{Bytecode, Opcode};
use std::fmt::Write;

/// Disassemble bytecode to human-readable format
///
/// # Format
/// ```text
/// === Constants ===
/// 0: 42.0
/// 1: "hello"
///
/// === Instructions ===
/// 0000  Constant 0
/// 0003  Add
/// 0004  Halt
/// ```
pub fn disassemble(bytecode: &Bytecode) -> String {
    let mut output = String::new();

    // Constants section
    if !bytecode.constants.is_empty() {
        writeln!(output, "=== Constants ===").unwrap();
        for (idx, constant) in bytecode.constants.iter().enumerate() {
            writeln!(output, "{}: {}", idx, format_value(constant)).unwrap();
        }
        writeln!(output).unwrap();
    }

    // Instructions section
    writeln!(output, "=== Instructions ===").unwrap();
    let mut offset = 0;
    while offset < bytecode.instructions.len() {
        let line = disassemble_instruction(bytecode, &mut offset);
        writeln!(output, "{}", line).unwrap();
    }

    output
}

/// Disassemble a single instruction at the given offset
///
/// Advances offset past the instruction and its operands.
/// Returns formatted instruction string.
fn disassemble_instruction(bytecode: &Bytecode, offset: &mut usize) -> String {
    let start_offset = *offset;

    // Read opcode
    if *offset >= bytecode.instructions.len() {
        return format!("{:04}  <invalid offset>", start_offset);
    }

    let byte = bytecode.instructions[*offset];
    *offset += 1;

    let opcode = match Opcode::try_from(byte) {
        Ok(op) => op,
        Err(_) => return format!("{:04}  <invalid opcode: {:#04x}>", start_offset, byte),
    };

    // Format based on opcode type
    match opcode {
        // Simple opcodes (no operands)
        Opcode::Null
        | Opcode::True
        | Opcode::False
        | Opcode::Add
        | Opcode::Sub
        | Opcode::Mul
        | Opcode::Div
        | Opcode::Mod
        | Opcode::Negate
        | Opcode::Equal
        | Opcode::NotEqual
        | Opcode::Less
        | Opcode::LessEqual
        | Opcode::Greater
        | Opcode::GreaterEqual
        | Opcode::Not
        | Opcode::And
        | Opcode::Or
        | Opcode::Return
        | Opcode::GetIndex
        | Opcode::SetIndex
        | Opcode::GetField
        | Opcode::SetField
        | Opcode::Slice
        | Opcode::SliceFrom
        | Opcode::SliceTo
        | Opcode::SliceFull
        | Opcode::Pop
        | Opcode::Dup
        | Opcode::Dup2
        | Opcode::Rot3
        | Opcode::ToString
        | Opcode::IsOptionSome
        | Opcode::IsOptionNone
        | Opcode::IsResultOk
        | Opcode::IsResultErr
        | Opcode::ExtractOptionValue
        | Opcode::ExtractResultValue
        | Opcode::IsArray
        | Opcode::GetArrayLen
        | Opcode::CheckEnumVariant
        | Opcode::ExtractEnumData
        | Opcode::Await
        | Opcode::WrapFuture
        | Opcode::Halt => {
            format!("{:04}  {:?}", start_offset, opcode)
        }

        // u16 operands (constants, locals, globals, upvalues)
        Opcode::Constant
        | Opcode::GetLocal
        | Opcode::SetLocal
        | Opcode::GetGlobal
        | Opcode::SetGlobal
        | Opcode::GetUpvalue
        | Opcode::SetUpvalue
        | Opcode::Array
        | Opcode::HashMap
        | Opcode::Tuple
        | Opcode::TupleGet => {
            let operand = read_u16(bytecode, offset);
            format!("{:04}  {:?} {}", start_offset, opcode, operand)
        }

        // Struct: two u16 operands (name_idx, field_count)
        Opcode::Struct => {
            let name_idx = read_u16(bytecode, offset);
            let field_count = read_u16(bytecode, offset);
            format!(
                "{:04}  Struct name={} fields={}",
                start_offset, name_idx, field_count
            )
        }

        // MakeClosure: two u16 operands (func_const_idx, n_upvalues)
        Opcode::MakeClosure => {
            let func_idx = read_u16(bytecode, offset);
            let n_upvalues = read_u16(bytecode, offset);
            format!(
                "{:04}  MakeClosure func={} upvalues={}",
                start_offset, func_idx, n_upvalues
            )
        }

        // u8 operand (call arg count, enum variant arg count)
        Opcode::Call | Opcode::EnumVariant | Opcode::Range => {
            let operand = read_u8(bytecode, offset);
            format!("{:04}  {:?} {}", start_offset, opcode, operand)
        }

        // AsyncCall/SpawnTask: u16 fn_const_idx + u8 arg_count
        Opcode::AsyncCall | Opcode::SpawnTask => {
            let fn_const_idx = read_u16(bytecode, offset);
            let arg_count = read_u8(bytecode, offset);
            format!(
                "{:04}  {:?} fn={} args={}",
                start_offset, opcode, fn_const_idx, arg_count
            )
        }

        // TraitDispatch: two u16 operands + u8 arg count
        Opcode::TraitDispatch => {
            let trait_idx = read_u16(bytecode, offset);
            let method_idx = read_u16(bytecode, offset);
            let arg_count = read_u8(bytecode, offset);
            format!(
                "{:04}  TraitDispatch trait={} method={} args={}",
                start_offset, trait_idx, method_idx, arg_count
            )
        }

        // i16 operands (jumps)
        Opcode::Jump | Opcode::JumpIfFalse | Opcode::Loop => {
            let jump_offset = read_i16(bytecode, offset);
            let target = (*offset as i32 + jump_offset as i32) as usize;
            format!(
                "{:04}  {:?} {} (-> {:04})",
                start_offset, opcode, jump_offset, target
            )
        }
    }
}

/// Read u8 operand from bytecode
fn read_u8(bytecode: &Bytecode, offset: &mut usize) -> u8 {
    if *offset >= bytecode.instructions.len() {
        return 0;
    }
    let value = bytecode.instructions[*offset];
    *offset += 1;
    value
}

/// Read u16 operand from bytecode (big-endian)
fn read_u16(bytecode: &Bytecode, offset: &mut usize) -> u16 {
    if *offset + 1 >= bytecode.instructions.len() {
        return 0;
    }
    let high = bytecode.instructions[*offset] as u16;
    let low = bytecode.instructions[*offset + 1] as u16;
    *offset += 2;
    (high << 8) | low
}

/// Read i16 operand from bytecode (big-endian, signed)
fn read_i16(bytecode: &Bytecode, offset: &mut usize) -> i16 {
    read_u16(bytecode, offset) as i16
}

/// Format a Value for constant pool display
fn format_value(value: &crate::value::Value) -> String {
    use crate::value::Value;
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => {
            // Show integers without decimal point
            if n.fract() == 0.0 && n.is_finite() {
                format!("{:.0}", n)
            } else {
                n.to_string()
            }
        }
        Value::String(s) => format!("\"{}\"", s),
        Value::Function(f) => format!("<fn {}({})>", f.name, f.arity),
        Value::Builtin(name) => format!("<builtin {}>", name),
        Value::NativeFunction(_) => "<native fn>".to_string(),
        Value::Array(_) => "<array>".to_string(),
        Value::Range { .. } => "<range>".to_string(),
        Value::JsonValue(_) => "<json>".to_string(),
        Value::Option(_) => "<option>".to_string(),
        Value::Result(_) => "<result>".to_string(),
        Value::HashMap(_) => "<hashmap>".to_string(),
        Value::HashSet(_) => "<hashset>".to_string(),
        Value::Queue(_) => "<queue>".to_string(),
        Value::Stack(_) => "<stack>".to_string(),
        Value::Regex(r) => format!("<regex /{}/>", r.as_str()),
        Value::DateTime(dt) => format!("<datetime {}>", dt.to_rfc3339()),
        Value::HttpRequest(req) => format!("<HttpRequest {} {}>", req.method(), req.url()),
        Value::HttpResponse(res) => format!("<HttpResponse {}>", res.status()),
        Value::ProcessOutput(out) => format!("<ProcessOutput exit={}>", out.exit_code),
        Value::Future(f) => format!("<{}>", f.as_ref()),
        Value::TaskHandle(h) => format!("<TaskHandle #{}>", h.lock().unwrap().id()),
        Value::ChannelSender(_) => "<ChannelSender>".to_string(),
        Value::ChannelReceiver(_) => "<ChannelReceiver>".to_string(),
        Value::AsyncMutex(_) => "<AsyncMutex>".to_string(),
        Value::Watcher(_) => "<Watcher>".to_string(),
        Value::Closure(c) => format!("<fn {}>", c.func.name),
        Value::SharedValue(_) => "<shared>".to_string(),
        Value::Tuple(elems) => format!("<tuple({} elements)>", elems.len()),
        Value::EnumValue {
            enum_name,
            variant_name,
            ..
        } => format!("<{}::{}>", enum_name, variant_name),
    }
}
