//! Bytecode validator — static analysis before VM execution
//!
//! Performs four checks:
//! 1. **Decode pass** — every byte is a known opcode with enough operand bytes
//! 2. **Jump targets** — all jump/loop destinations are within bounds and land
//!    on a valid opcode boundary
//! 3. **Constant refs** — all constant/global indices are within the pool
//! 4. **Stack depth** — linear walk detects obvious stack underflow
//!
//! Call sites are free to ignore the result; the validator is advisory and does
//! not affect VM execution.

use crate::bytecode::{Bytecode, Opcode};

// ============================================================================
// Public API
// ============================================================================

/// A validation error with the byte offset where it was detected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    /// Byte offset in the instruction stream where the error was detected.
    pub offset: usize,
    /// What went wrong.
    pub kind: ValidationErrorKind,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "offset {:#06x}: {}", self.offset, self.kind)
    }
}

/// Kinds of errors the validator can detect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationErrorKind {
    /// A byte that is not a recognised opcode.
    UnknownOpcode(u8),
    /// An opcode was found but the instruction stream ended before its operands.
    TruncatedInstruction { opcode: &'static str },
    /// A jump/loop target falls outside `[0, instructions.len())`.
    JumpOutOfBounds { target: usize, len: usize },
    /// A jump/loop target does not land on a known opcode boundary.
    JumpMisaligned { target: usize },
    /// A constant-pool or global-name index exceeds the pool size.
    ConstantIndexOutOfBounds { index: usize, pool_size: usize },
    /// Stack depth went negative — a pop with nothing on the stack.
    StackUnderflow { op: &'static str, depth_before: i32 },
    /// The last reachable instruction is neither `Halt` nor `Return`.
    MissingTerminator,
}

impl std::fmt::Display for ValidationErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownOpcode(b) => write!(f, "unknown opcode {:#04x}", b),
            Self::TruncatedInstruction { opcode } => {
                write!(
                    f,
                    "instruction {} is truncated (missing operand bytes)",
                    opcode
                )
            }
            Self::JumpOutOfBounds { target, len } => {
                write!(f, "jump target {} is out of bounds (len={})", target, len)
            }
            Self::JumpMisaligned { target } => {
                write!(
                    f,
                    "jump target {} does not align to an opcode boundary",
                    target
                )
            }
            Self::ConstantIndexOutOfBounds { index, pool_size } => {
                write!(
                    f,
                    "constant index {} out of bounds (pool size={})",
                    index, pool_size
                )
            }
            Self::StackUnderflow { op, depth_before } => {
                write!(
                    f,
                    "stack underflow in {}: depth before = {}",
                    op, depth_before
                )
            }
            Self::MissingTerminator => {
                write!(f, "bytecode does not end with Halt or Return")
            }
        }
    }
}

/// Validate `bytecode`, collecting all errors found.
///
/// Returns `Ok(())` if no issues are found, otherwise `Err(errors)` with every
/// detected problem. Does NOT short-circuit on the first error.
pub fn validate(bytecode: &Bytecode) -> Result<(), Vec<ValidationError>> {
    let mut errors: Vec<ValidationError> = Vec::new();

    // Pass 1: decode into a list of (offset, opcode, operand_value)
    let decoded = decode_instructions(bytecode, &mut errors);

    // Build a set of valid opcode-start offsets for jump-target checks.
    let valid_offsets: std::collections::HashSet<usize> =
        decoded.iter().map(|e| e.offset).collect();

    // Pass 2: validate jump targets
    check_jump_targets(bytecode, &decoded, &valid_offsets, &mut errors);

    // Pass 3: validate constant-pool references
    check_constant_refs(bytecode, &decoded, &mut errors);

    // Pass 4: stack depth simulation
    check_stack_depth(&decoded, &mut errors);

    // Pass 5: termination check
    check_terminator(&decoded, &mut errors);

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

// ============================================================================
// Internal decoded instruction
// ============================================================================

/// A decoded instruction with all relevant data extracted.
#[derive(Debug, Clone)]
struct DecodedInstruction {
    /// Byte offset of the opcode itself.
    offset: usize,
    /// The opcode (None if the byte was unknown — errors already emitted).
    opcode: Option<Opcode>,
    /// Numeric operand value (meaning depends on opcode).
    operand: i64,
}

// ============================================================================
// Pass 1: decode
// ============================================================================

fn decode_instructions(
    bytecode: &Bytecode,
    errors: &mut Vec<ValidationError>,
) -> Vec<DecodedInstruction> {
    let code = &bytecode.instructions;
    let mut decoded = Vec::new();
    let mut ip = 0usize;

    while ip < code.len() {
        let offset = ip;
        let byte = code[ip];
        ip += 1;

        let opcode = match Opcode::try_from(byte) {
            Ok(op) => op,
            Err(_) => {
                errors.push(ValidationError {
                    offset,
                    kind: ValidationErrorKind::UnknownOpcode(byte),
                });
                // Skip 1 byte and continue best-effort decoding
                decoded.push(DecodedInstruction {
                    offset,
                    opcode: None,
                    operand: 0,
                });
                continue;
            }
        };

        let (extra_bytes, operand) = match read_operand(opcode, code, ip) {
            Ok(pair) => pair,
            Err(name) => {
                errors.push(ValidationError {
                    offset,
                    kind: ValidationErrorKind::TruncatedInstruction { opcode: name },
                });
                decoded.push(DecodedInstruction {
                    offset,
                    opcode: Some(opcode),
                    operand: 0,
                });
                break; // Can't continue; don't know where next op starts
            }
        };

        ip += extra_bytes;
        decoded.push(DecodedInstruction {
            offset,
            opcode: Some(opcode),
            operand,
        });
    }

    decoded
}

/// Try to read the operand for `opcode` starting at `ip` in `code`.
///
/// Returns `(extra_bytes, operand_value)` on success, or the opcode name on
/// truncation error.
fn read_operand(opcode: Opcode, code: &[u8], ip: usize) -> Result<(usize, i64), &'static str> {
    match opcode {
        // 2-byte unsigned operand (u16)
        Opcode::Constant
        | Opcode::GetLocal
        | Opcode::SetLocal
        | Opcode::GetGlobal
        | Opcode::SetGlobal
        | Opcode::GetUpvalue
        | Opcode::SetUpvalue
        | Opcode::Array => {
            if ip + 1 >= code.len() {
                return Err(opcode_name(opcode));
            }
            let hi = code[ip] as u16;
            let lo = code[ip + 1] as u16;
            Ok((2, ((hi << 8) | lo) as i64))
        }
        // MakeClosure: 4 bytes (two u16: func_const_idx, n_upvalues)
        Opcode::MakeClosure => {
            if ip + 3 >= code.len() {
                return Err(opcode_name(opcode));
            }
            let hi = code[ip] as u16;
            let lo = code[ip + 1] as u16;
            Ok((4, ((hi << 8) | lo) as i64))
        }
        // 2-byte signed operand (i16)
        Opcode::Jump | Opcode::JumpIfFalse | Opcode::Loop => {
            if ip + 1 >= code.len() {
                return Err(opcode_name(opcode));
            }
            let hi = code[ip] as i16;
            let lo = code[ip + 1] as i16;
            let value = (hi << 8) | lo;
            Ok((2, value as i64))
        }
        // 1-byte operand (u8)
        Opcode::Call => {
            if ip >= code.len() {
                return Err(opcode_name(opcode));
            }
            Ok((1, code[ip] as i64))
        }
        // No operand
        _ => Ok((0, 0)),
    }
}

/// Static name for an opcode (used in error messages).
fn opcode_name(opcode: Opcode) -> &'static str {
    match opcode {
        Opcode::Constant => "Constant",
        Opcode::Null => "Null",
        Opcode::True => "True",
        Opcode::False => "False",
        Opcode::GetLocal => "GetLocal",
        Opcode::SetLocal => "SetLocal",
        Opcode::GetGlobal => "GetGlobal",
        Opcode::SetGlobal => "SetGlobal",
        Opcode::Add => "Add",
        Opcode::Sub => "Sub",
        Opcode::Mul => "Mul",
        Opcode::Div => "Div",
        Opcode::Mod => "Mod",
        Opcode::Negate => "Negate",
        Opcode::Equal => "Equal",
        Opcode::NotEqual => "NotEqual",
        Opcode::Less => "Less",
        Opcode::LessEqual => "LessEqual",
        Opcode::Greater => "Greater",
        Opcode::GreaterEqual => "GreaterEqual",
        Opcode::Not => "Not",
        Opcode::And => "And",
        Opcode::Or => "Or",
        Opcode::Jump => "Jump",
        Opcode::JumpIfFalse => "JumpIfFalse",
        Opcode::Loop => "Loop",
        Opcode::Call => "Call",
        Opcode::Return => "Return",
        Opcode::Array => "Array",
        Opcode::GetIndex => "GetIndex",
        Opcode::SetIndex => "SetIndex",
        Opcode::Pop => "Pop",
        Opcode::Dup => "Dup",
        Opcode::IsOptionSome => "IsOptionSome",
        Opcode::IsOptionNone => "IsOptionNone",
        Opcode::IsResultOk => "IsResultOk",
        Opcode::IsResultErr => "IsResultErr",
        Opcode::ExtractOptionValue => "ExtractOptionValue",
        Opcode::ExtractResultValue => "ExtractResultValue",
        Opcode::IsArray => "IsArray",
        Opcode::GetArrayLen => "GetArrayLen",
        Opcode::Halt => "Halt",
        Opcode::MakeClosure => "MakeClosure",
        Opcode::GetUpvalue => "GetUpvalue",
        Opcode::SetUpvalue => "SetUpvalue",
    }
}

// ============================================================================
// Pass 2: jump targets
// ============================================================================

fn check_jump_targets(
    bytecode: &Bytecode,
    decoded: &[DecodedInstruction],
    valid_offsets: &std::collections::HashSet<usize>,
    errors: &mut Vec<ValidationError>,
) {
    let len = bytecode.instructions.len();

    for instr in decoded {
        let is_jump = matches!(
            instr.opcode,
            Some(Opcode::Jump) | Some(Opcode::JumpIfFalse) | Some(Opcode::Loop)
        );
        if !is_jump {
            continue;
        }

        // The stored offset is relative to the byte AFTER the operand (ip after read).
        // In the VM: ip is advanced past opcode+operand before the jump is applied.
        // operand bytes: 2 bytes (i16)
        let operand_end = instr.offset + 3; // 1 opcode + 2 operand bytes
        let target = operand_end as isize + instr.operand as isize;

        if target < 0 || target as usize >= len {
            errors.push(ValidationError {
                offset: instr.offset,
                kind: ValidationErrorKind::JumpOutOfBounds {
                    target: target.max(0) as usize,
                    len,
                },
            });
            continue;
        }

        let target = target as usize;
        if !valid_offsets.contains(&target) {
            errors.push(ValidationError {
                offset: instr.offset,
                kind: ValidationErrorKind::JumpMisaligned { target },
            });
        }
    }
}

// ============================================================================
// Pass 3: constant-pool references
// ============================================================================

fn check_constant_refs(
    bytecode: &Bytecode,
    decoded: &[DecodedInstruction],
    errors: &mut Vec<ValidationError>,
) {
    let pool_size = bytecode.constants.len();

    for instr in decoded {
        let needs_pool = matches!(
            instr.opcode,
            Some(Opcode::Constant) | Some(Opcode::GetGlobal) | Some(Opcode::SetGlobal)
        );
        if !needs_pool {
            continue;
        }

        let index = instr.operand as usize;
        if index >= pool_size {
            errors.push(ValidationError {
                offset: instr.offset,
                kind: ValidationErrorKind::ConstantIndexOutOfBounds { index, pool_size },
            });
        }
    }
}

// ============================================================================
// Pass 4: stack depth simulation
// ============================================================================

/// Stack depth delta for an opcode.
///
/// Returns `None` for opcodes whose delta depends on their operand at runtime
/// (`Call`, `Array`) — stack tracking is skipped for those.
fn stack_delta(instr: &DecodedInstruction) -> Option<i32> {
    match instr.opcode? {
        // Pushes
        Opcode::Constant
        | Opcode::Null
        | Opcode::True
        | Opcode::False
        | Opcode::GetLocal
        | Opcode::GetGlobal
        | Opcode::GetUpvalue
        | Opcode::Dup => Some(1),

        // Neutral (peek-based or pop-1/push-1)
        Opcode::SetLocal
        | Opcode::SetGlobal
        | Opcode::SetUpvalue
        | Opcode::Negate
        | Opcode::Not
        | Opcode::Jump
        | Opcode::Loop
        | Opcode::IsOptionSome
        | Opcode::IsOptionNone
        | Opcode::IsResultOk
        | Opcode::IsResultErr
        | Opcode::ExtractOptionValue
        | Opcode::ExtractResultValue
        | Opcode::IsArray
        | Opcode::GetArrayLen
        | Opcode::Halt => Some(0),

        // Pop 1
        Opcode::Pop | Opcode::JumpIfFalse => Some(-1),

        // Pop 2, push 1
        Opcode::Add
        | Opcode::Sub
        | Opcode::Mul
        | Opcode::Div
        | Opcode::Mod
        | Opcode::Equal
        | Opcode::NotEqual
        | Opcode::Less
        | Opcode::LessEqual
        | Opcode::Greater
        | Opcode::GreaterEqual
        | Opcode::And
        | Opcode::Or
        | Opcode::GetIndex => Some(-1),

        // Pop 3, push 1 (value assigned back)
        Opcode::SetIndex => Some(-2),

        // Variable-arity — skip (MakeClosure pops n_upvalues, push 1; net depends on operand)
        Opcode::Call | Opcode::Array | Opcode::MakeClosure => None,

        // Return drains the frame — stop tracking
        Opcode::Return => None,
    }
}

fn check_stack_depth(decoded: &[DecodedInstruction], errors: &mut Vec<ValidationError>) {
    let mut depth: i32 = 0;

    for instr in decoded {
        let op_name = instr.opcode.map(opcode_name).unwrap_or("<unknown>");

        match stack_delta(instr) {
            None => {
                // Call/Array/Return — reset depth tracking conservatively.
                // After a Call we know net result is +1 (return value), but arity
                // is unknown statically, so we just reset to a safe minimum.
                if matches!(instr.opcode, Some(Opcode::Return)) {
                    break; // End of this code path
                }
                // For Call/Array: assume depth stays valid, reset to current
                // (don't report spurious underflows after these)
            }
            Some(delta) => {
                if delta < 0 && depth + delta < 0 {
                    errors.push(ValidationError {
                        offset: instr.offset,
                        kind: ValidationErrorKind::StackUnderflow {
                            op: op_name,
                            depth_before: depth,
                        },
                    });
                    // Continue with depth = 0 to catch further errors
                    depth = 0;
                } else {
                    depth += delta;
                }
            }
        }
    }
}

// ============================================================================
// Pass 5: termination
// ============================================================================

fn check_terminator(decoded: &[DecodedInstruction], errors: &mut Vec<ValidationError>) {
    let last = decoded.iter().rev().find(|i| i.opcode.is_some());
    match last {
        None => {
            errors.push(ValidationError {
                offset: 0,
                kind: ValidationErrorKind::MissingTerminator,
            });
        }
        Some(instr) => {
            let is_terminal = matches!(instr.opcode, Some(Opcode::Halt) | Some(Opcode::Return));
            if !is_terminal {
                errors.push(ValidationError {
                    offset: instr.offset,
                    kind: ValidationErrorKind::MissingTerminator,
                });
            }
        }
    }
}

// ============================================================================
// Unit tests
// ============================================================================
