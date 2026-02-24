//! Constant folding optimization pass
//!
//! Evaluates constant expressions at compile time:
//! - Binary arithmetic: `Constant(a), Constant(b), Op` → `Constant(a op b)`
//! - Unary negation: `Constant(n), Negate` → `Constant(-n)`
//! - Boolean not: `Constant(bool), Not` → `True`/`False`
//! - Literal not: `True/False, Not` → `False/True`
//!
//! Multiple passes are run until the bytecode stabilizes.

use super::{
    decode_instructions, encode_instructions, fix_all_references, DecodedInstruction,
    OptimizationPass, OptimizationStats,
};
use crate::bytecode::{Bytecode, Opcode};
use crate::value::Value;

/// Constant folding optimization pass
///
/// Repeatedly scans the instruction stream looking for constant expressions
/// and folds them into a single constant.  Runs until stable (fixed point).
pub struct ConstantFoldingPass;

impl OptimizationPass for ConstantFoldingPass {
    fn name(&self) -> &str {
        "constant-folding"
    }

    fn optimize(&self, bytecode: Bytecode) -> (Bytecode, OptimizationStats) {
        let mut stats = OptimizationStats::new();
        stats.bytecode_size_before = bytecode.instructions.len();
        stats.passes_run = 1;

        let top_level_local_count = bytecode.top_level_local_count;
        let mut constants = bytecode.constants.clone();
        let mut decoded = decode_instructions(&bytecode);

        let mut changed = true;
        while changed {
            changed = false;
            let mut new_decoded: Vec<DecodedInstruction> = Vec::with_capacity(decoded.len());
            let mut i = 0;

            while i < decoded.len() {
                // ── Pattern: Constant(a), Constant(b), BinaryOp ───────────────
                if i + 2 < decoded.len()
                    && decoded[i].opcode == Opcode::Constant
                    && decoded[i + 1].opcode == Opcode::Constant
                {
                    let a_idx = decoded[i].read_u16() as usize;
                    let b_idx = decoded[i + 1].read_u16() as usize;
                    let op = decoded[i + 2].opcode;

                    if a_idx < constants.len() && b_idx < constants.len() && is_foldable_binary(op)
                    {
                        if let Some(result) = fold_binary(&constants[a_idx], &constants[b_idx], op)
                        {
                            let new_idx = constants.len() as u16;
                            constants.push(result);
                            let span = decoded[i].span.or(decoded[i + 2].span);
                            new_decoded.push(DecodedInstruction {
                                offset: decoded[i].offset,
                                opcode: Opcode::Constant,
                                operands: DecodedInstruction::make_u16_operands(new_idx),
                                span,
                            });
                            i += 3;
                            stats.constants_folded += 1;
                            changed = true;
                            continue;
                        }
                    }
                }

                // ── Pattern: Constant(n), Negate ──────────────────────────────
                if i + 1 < decoded.len()
                    && decoded[i].opcode == Opcode::Constant
                    && decoded[i + 1].opcode == Opcode::Negate
                {
                    let a_idx = decoded[i].read_u16() as usize;
                    if a_idx < constants.len() {
                        if let Value::Number(n) = constants[a_idx] {
                            let new_idx = constants.len() as u16;
                            constants.push(Value::Number(-n));
                            let span = decoded[i].span;
                            new_decoded.push(DecodedInstruction {
                                offset: decoded[i].offset,
                                opcode: Opcode::Constant,
                                operands: DecodedInstruction::make_u16_operands(new_idx),
                                span,
                            });
                            i += 2;
                            stats.constants_folded += 1;
                            changed = true;
                            continue;
                        }
                    }
                }

                // ── Pattern: Constant(bool), Not ──────────────────────────────
                if i + 1 < decoded.len()
                    && decoded[i].opcode == Opcode::Constant
                    && decoded[i + 1].opcode == Opcode::Not
                {
                    let a_idx = decoded[i].read_u16() as usize;
                    if a_idx < constants.len() {
                        if let Value::Bool(b) = constants[a_idx] {
                            let new_opcode = if b { Opcode::False } else { Opcode::True };
                            let span = decoded[i].span;
                            new_decoded.push(DecodedInstruction {
                                offset: decoded[i].offset,
                                opcode: new_opcode,
                                operands: Vec::new(),
                                span,
                            });
                            i += 2;
                            stats.constants_folded += 1;
                            changed = true;
                            continue;
                        }
                    }
                }

                // ── Pattern: True, Not → False ────────────────────────────────
                if i + 1 < decoded.len()
                    && decoded[i].opcode == Opcode::True
                    && decoded[i + 1].opcode == Opcode::Not
                {
                    let span = decoded[i].span;
                    new_decoded.push(DecodedInstruction {
                        offset: decoded[i].offset,
                        opcode: Opcode::False,
                        operands: Vec::new(),
                        span,
                    });
                    i += 2;
                    stats.constants_folded += 1;
                    changed = true;
                    continue;
                }

                // ── Pattern: False, Not → True ────────────────────────────────
                if i + 1 < decoded.len()
                    && decoded[i].opcode == Opcode::False
                    && decoded[i + 1].opcode == Opcode::Not
                {
                    let span = decoded[i].span;
                    new_decoded.push(DecodedInstruction {
                        offset: decoded[i].offset,
                        opcode: Opcode::True,
                        operands: Vec::new(),
                        span,
                    });
                    i += 2;
                    stats.constants_folded += 1;
                    changed = true;
                    continue;
                }

                // ── Pattern: Null, Not ────────────────────────────────────────
                // null is falsy, so !null = true
                if i + 1 < decoded.len()
                    && decoded[i].opcode == Opcode::Null
                    && decoded[i + 1].opcode == Opcode::Not
                {
                    let span = decoded[i].span;
                    new_decoded.push(DecodedInstruction {
                        offset: decoded[i].offset,
                        opcode: Opcode::True,
                        operands: Vec::new(),
                        span,
                    });
                    i += 2;
                    stats.constants_folded += 1;
                    changed = true;
                    continue;
                }

                // No pattern matched — keep instruction as-is
                new_decoded.push(decoded[i].clone());
                i += 1;
            }

            decoded = new_decoded;
        }

        // Fix jump targets and function offsets after structural changes
        fix_all_references(&mut decoded, &mut constants);

        let result = encode_instructions(&decoded, constants, top_level_local_count);
        stats.bytecode_size_after = result.instructions.len();
        (result, stats)
    }
}

/// Returns true if `op` is a binary opcode that constant folding can evaluate
fn is_foldable_binary(op: Opcode) -> bool {
    matches!(
        op,
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
    )
}

/// Attempt to fold a binary operation on two constant values.
/// Returns `None` if the operation is not supported, or would produce a
/// runtime error (e.g., division by zero).
fn fold_binary(a: &Value, b: &Value, op: Opcode) -> Option<Value> {
    match (a, b) {
        (Value::Number(an), Value::Number(bn)) => match op {
            Opcode::Add => Some(Value::Number(an + bn)),
            Opcode::Sub => Some(Value::Number(an - bn)),
            Opcode::Mul => Some(Value::Number(an * bn)),
            Opcode::Div => {
                // Preserve runtime semantics: don't fold division by zero
                if *bn == 0.0 {
                    None
                } else {
                    Some(Value::Number(an / bn))
                }
            }
            Opcode::Mod => {
                if *bn == 0.0 {
                    None
                } else {
                    Some(Value::Number(an % bn))
                }
            }
            Opcode::Equal => Some(Value::Bool((an - bn).abs() < f64::EPSILON)),
            Opcode::NotEqual => Some(Value::Bool((an - bn).abs() >= f64::EPSILON)),
            Opcode::Less => Some(Value::Bool(an < bn)),
            Opcode::LessEqual => Some(Value::Bool(an <= bn)),
            Opcode::Greater => Some(Value::Bool(an > bn)),
            Opcode::GreaterEqual => Some(Value::Bool(an >= bn)),
            _ => None,
        },
        (Value::Bool(ab), Value::Bool(bb)) => match op {
            Opcode::Equal => Some(Value::Bool(ab == bb)),
            Opcode::NotEqual => Some(Value::Bool(ab != bb)),
            _ => None,
        },
        _ => None,
    }
}
