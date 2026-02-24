//! Dead code elimination optimization pass
//!
//! Removes instructions that can never be executed:
//! - Code after unconditional jumps/returns (when not a jump target)
//! - Any instruction not reachable from the entry point
//!
//! Algorithm:
//! 1. Decode bytecode into instruction list
//! 2. BFS from offset 0, following normal execution and jump targets
//! 3. Remove all instructions not in the reachable set
//! 4. Fix all jump offsets and function references to account for removals

use super::{
    decode_instructions, encode_instructions, fix_all_references, DecodedInstruction,
    OptimizationPass, OptimizationStats,
};
use crate::bytecode::{Bytecode, Opcode};
use std::collections::{HashMap, HashSet, VecDeque};

/// Dead code elimination pass
///
/// Performs a BFS reachability analysis from instruction offset 0 and removes
/// any instructions that cannot be reached during normal execution.
pub struct DeadCodeEliminationPass;

impl OptimizationPass for DeadCodeEliminationPass {
    fn name(&self) -> &str {
        "dead-code-elimination"
    }

    fn optimize(&self, bytecode: Bytecode) -> (Bytecode, OptimizationStats) {
        let mut stats = OptimizationStats::new();
        stats.bytecode_size_before = bytecode.instructions.len();
        stats.passes_run = 1;

        if bytecode.instructions.is_empty() {
            stats.bytecode_size_after = 0;
            return (bytecode, stats);
        }

        let top_level_local_count = bytecode.top_level_local_count;
        let decoded = decode_instructions(&bytecode);
        if decoded.is_empty() {
            stats.bytecode_size_after = bytecode.instructions.len();
            return (bytecode, stats);
        }

        // Build offset → instruction index map for efficient lookup
        let offset_to_idx: HashMap<usize, usize> = decoded
            .iter()
            .enumerate()
            .map(|(i, instr)| (instr.offset, i))
            .collect();

        // BFS reachability analysis
        let reachable = compute_reachable(&decoded, &offset_to_idx, &bytecode);

        // Count dead instructions
        let dead_count = decoded.len() - reachable.len();
        if dead_count == 0 {
            // Nothing to remove
            stats.bytecode_size_after = bytecode.instructions.len();
            return (bytecode, stats);
        }

        stats.dead_instructions_removed = dead_count;

        // Keep only reachable instructions (preserve order)
        let mut live: Vec<DecodedInstruction> = decoded
            .into_iter()
            .filter(|instr| reachable.contains(&instr.offset))
            .collect();

        let mut constants = bytecode.constants;
        fix_all_references(&mut live, &mut constants);

        let result = encode_instructions(&live, constants, top_level_local_count);
        stats.bytecode_size_after = result.instructions.len();
        (result, stats)
    }
}

/// BFS reachability analysis starting from offset 0.
///
/// Returns the set of byte offsets of reachable instructions.
fn compute_reachable(
    decoded: &[DecodedInstruction],
    offset_to_idx: &HashMap<usize, usize>,
    bytecode: &Bytecode,
) -> HashSet<usize> {
    let mut reachable: HashSet<usize> = HashSet::new();
    let mut queue: VecDeque<usize> = VecDeque::new();

    // Seed with entry point
    if !decoded.is_empty() {
        queue.push_back(decoded[0].offset);
    }

    // Also seed with all function body entry points (they're called indirectly)
    for constant in &bytecode.constants {
        if let crate::value::Value::Function(ref func) = constant {
            if func.bytecode_offset > 0 {
                queue.push_back(func.bytecode_offset);
            }
        }
    }

    while let Some(offset) = queue.pop_front() {
        if reachable.contains(&offset) {
            continue;
        }

        let idx = match offset_to_idx.get(&offset) {
            Some(&i) => i,
            None => continue, // Invalid offset, skip
        };

        let instr = &decoded[idx];
        reachable.insert(offset);

        match instr.opcode {
            // Unconditional jump: only successor is the jump target
            Opcode::Jump => {
                if instr.operands.len() == 2 {
                    let relative = instr.read_i16();
                    let target = (offset as isize + 3 + relative as isize) as usize;
                    queue.push_back(target);
                }
                // NO fallthrough for unconditional jump
            }

            // Loop (backward jump): only successor is the loop target
            Opcode::Loop => {
                if instr.operands.len() == 2 {
                    let relative = instr.read_i16();
                    let target = (offset as isize + 3 + relative as isize) as usize;
                    queue.push_back(target);
                }
                // NO fallthrough
            }

            // Conditional jump: both fallthrough and jump target are successors
            Opcode::JumpIfFalse => {
                if instr.operands.len() == 2 {
                    let relative = instr.read_i16();
                    let target = (offset as isize + 3 + relative as isize) as usize;
                    queue.push_back(target);
                }
                // Also fallthrough to next instruction
                let next_offset = offset + instr.byte_size();
                queue.push_back(next_offset);
            }

            // Terminators: no successors
            Opcode::Return | Opcode::Halt => {}

            // All other instructions: fallthrough to next
            _ => {
                let next_offset = offset + instr.byte_size();
                queue.push_back(next_offset);
            }
        }
    }

    reachable
}
