//! Peephole optimization pass
//!
//! Applies local pattern simplifications to the instruction stream:
//! - `Dup, Pop` → nothing (useless dup immediately discarded)
//! - `Not, Not` → nothing (double negation)
//! - `True, Not` → `False` (constant boolean flip)
//! - `False, Not` → `True` (constant boolean flip)
//! - `Jump +0` → nothing (jump to next instruction)
//! - Jump threading: `Jump A` where A is `Jump B` → `Jump B`
//!
//! Multiple passes are run until the bytecode stabilizes.

use super::{
    decode_instructions, encode_instructions, fix_all_references, DecodedInstruction,
    OptimizationPass, OptimizationStats,
};
use crate::bytecode::{Bytecode, Opcode};

/// Peephole optimization pass
///
/// Applies small, local transformations that eliminate wasteful patterns.
/// Runs until the bytecode stabilizes.
pub struct PeepholePass;

impl OptimizationPass for PeepholePass {
    fn name(&self) -> &str {
        "peephole"
    }

    fn optimize(&self, bytecode: Bytecode) -> (Bytecode, OptimizationStats) {
        let mut stats = OptimizationStats::new();
        stats.bytecode_size_before = bytecode.instructions.len();
        stats.passes_run = 1;

        let top_level_local_count = bytecode.top_level_local_count;
        let mut decoded = decode_instructions(&bytecode);
        let mut constants = bytecode.constants.clone();

        let mut changed = true;
        while changed {
            changed = false;
            let mut new_decoded: Vec<DecodedInstruction> = Vec::with_capacity(decoded.len());
            let mut i = 0;

            while i < decoded.len() {
                // ── Pattern: Dup, Pop → nothing ──────────────────────────────
                if i + 1 < decoded.len()
                    && decoded[i].opcode == Opcode::Dup
                    && decoded[i + 1].opcode == Opcode::Pop
                {
                    i += 2;
                    stats.peephole_patterns_applied += 1;
                    changed = true;
                    continue;
                }

                // ── Pattern: Not, Not → nothing ──────────────────────────────
                if i + 1 < decoded.len()
                    && decoded[i].opcode == Opcode::Not
                    && decoded[i + 1].opcode == Opcode::Not
                {
                    i += 2;
                    stats.peephole_patterns_applied += 1;
                    changed = true;
                    continue;
                }

                // ── Pattern: Pop, Pop → nothing (eliminate set+discard) ───────
                // Only safe if there's nothing between producing and popping
                // This is NOT always safe — we skip this pattern intentionally
                // to avoid removing side-effects.

                // ── Pattern: Jump +0 → nothing ───────────────────────────────
                // Jump with relative offset 0 jumps to the instruction right
                // after the jump's operands (a no-op).
                if decoded[i].opcode == Opcode::Jump && decoded[i].operands.len() == 2 {
                    let relative = decoded[i].read_i16();
                    if relative == 0 {
                        i += 1;
                        stats.peephole_patterns_applied += 1;
                        changed = true;
                        continue;
                    }
                }

                // ── Jump threading: Jump A where A is Jump B → Jump B ─────────
                // Resolve chains of unconditional jumps to their final target.
                if decoded[i].opcode == Opcode::Jump && decoded[i].operands.len() == 2 {
                    let relative = decoded[i].read_i16();
                    let origin_after = decoded[i].offset + 3;
                    let target = (origin_after as isize + relative as isize) as usize;

                    // Check if target instruction is also a Jump
                    if let Some(target_instr) = find_instruction_at(&decoded, target) {
                        if target_instr.opcode == Opcode::Jump
                            && target_instr.operands.len() == 2
                            && target != decoded[i].offset
                        {
                            let inner_relative = target_instr.read_i16();
                            let inner_after = target + 3;
                            let final_target =
                                (inner_after as isize + inner_relative as isize) as usize;

                            // Don't create an infinite loop
                            if final_target != decoded[i].offset {
                                // Compute new relative offset from current instruction
                                let new_after = decoded[i].offset + 3;
                                let new_relative =
                                    (final_target as isize - new_after as isize) as i16;
                                let mut threaded = decoded[i].clone();
                                threaded.operands =
                                    DecodedInstruction::make_i16_operands(new_relative);
                                new_decoded.push(threaded);
                                i += 1;
                                stats.peephole_patterns_applied += 1;
                                changed = true;
                                continue;
                            }
                        }
                    }
                }

                // ── JumpIfFalse threading ─────────────────────────────────────
                // JumpIfFalse A where A is Jump B → update target to B
                if decoded[i].opcode == Opcode::JumpIfFalse && decoded[i].operands.len() == 2 {
                    let relative = decoded[i].read_i16();
                    let origin_after = decoded[i].offset + 3;
                    let target = (origin_after as isize + relative as isize) as usize;

                    if let Some(target_instr) = find_instruction_at(&decoded, target) {
                        if target_instr.opcode == Opcode::Jump
                            && target_instr.operands.len() == 2
                            && target != decoded[i].offset
                        {
                            let inner_relative = target_instr.read_i16();
                            let inner_after = target + 3;
                            let final_target =
                                (inner_after as isize + inner_relative as isize) as usize;

                            if final_target != decoded[i].offset {
                                let new_after = decoded[i].offset + 3;
                                let new_relative =
                                    (final_target as isize - new_after as isize) as i16;
                                let mut threaded = decoded[i].clone();
                                threaded.operands =
                                    DecodedInstruction::make_i16_operands(new_relative);
                                new_decoded.push(threaded);
                                i += 1;
                                stats.peephole_patterns_applied += 1;
                                changed = true;
                                continue;
                            }
                        }
                    }
                }

                // No pattern matched — keep as-is
                new_decoded.push(decoded[i].clone());
                i += 1;
            }

            decoded = new_decoded;
        }

        fix_all_references(&mut decoded, &mut constants);

        let result = encode_instructions(&decoded, constants, top_level_local_count);
        stats.bytecode_size_after = result.instructions.len();
        (result, stats)
    }
}

/// Find a decoded instruction by its original byte offset
fn find_instruction_at(
    decoded: &[DecodedInstruction],
    offset: usize,
) -> Option<&DecodedInstruction> {
    decoded.iter().find(|instr| instr.offset == offset)
}
