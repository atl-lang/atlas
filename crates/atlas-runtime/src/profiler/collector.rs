//! Profile data collector
//!
//! Records execution statistics during VM runs: instruction counts,
//! per-location hotness, stack depth extremes, and function call counts.

use crate::bytecode::Opcode;
use std::collections::HashMap;

/// Execution statistics collected during a VM run
#[derive(Debug, Clone)]
pub struct ProfileCollector {
    /// Total instructions executed
    total_instructions: u64,
    /// Count per opcode (opcode byte → count)
    instruction_counts: HashMap<u8, u64>,
    /// Count per instruction location (IP → count)
    location_counts: HashMap<usize, u64>,
    /// Opcode recorded at each IP (for hotspot labelling)
    location_opcodes: HashMap<usize, u8>,
    /// Maximum call stack depth observed
    max_stack_depth: usize,
    /// Maximum value stack depth observed
    max_value_stack_depth: usize,
    /// Total function calls recorded
    function_calls: u64,
    /// Calls per named function
    function_call_counts: HashMap<String, u64>,
}

impl ProfileCollector {
    /// Create a new, empty collector
    pub fn new() -> Self {
        Self {
            total_instructions: 0,
            instruction_counts: HashMap::new(),
            location_counts: HashMap::new(),
            location_opcodes: HashMap::new(),
            max_stack_depth: 0,
            max_value_stack_depth: 0,
            function_calls: 0,
            function_call_counts: HashMap::new(),
        }
    }

    /// Record an instruction execution at a specific IP
    pub fn record_instruction(&mut self, opcode: Opcode, ip: usize) {
        self.total_instructions += 1;
        let byte = opcode as u8;
        *self.instruction_counts.entry(byte).or_insert(0) += 1;
        *self.location_counts.entry(ip).or_insert(0) += 1;
        self.location_opcodes.entry(ip).or_insert(byte);
    }

    /// Record an instruction without a specific location (backward compat)
    pub fn record_instruction_opcode(&mut self, opcode: Opcode) {
        self.total_instructions += 1;
        let byte = opcode as u8;
        *self.instruction_counts.entry(byte).or_insert(0) += 1;
    }

    /// Update the observed call stack depth
    pub fn update_frame_depth(&mut self, depth: usize) {
        if depth > self.max_stack_depth {
            self.max_stack_depth = depth;
        }
    }

    /// Update the observed value stack depth
    pub fn update_value_stack_depth(&mut self, depth: usize) {
        if depth > self.max_value_stack_depth {
            self.max_value_stack_depth = depth;
        }
    }

    /// Record a named function call
    pub fn record_function_call(&mut self, name: &str) {
        self.function_calls += 1;
        *self
            .function_call_counts
            .entry(name.to_string())
            .or_insert(0) += 1;
    }

    /// Reset all counters
    pub fn reset(&mut self) {
        self.total_instructions = 0;
        self.instruction_counts.clear();
        self.location_counts.clear();
        self.location_opcodes.clear();
        self.max_stack_depth = 0;
        self.max_value_stack_depth = 0;
        self.function_calls = 0;
        self.function_call_counts.clear();
    }

    // --- Accessors ---

    /// Total instructions executed
    pub fn total_instructions(&self) -> u64 {
        self.total_instructions
    }

    /// Count for a specific opcode
    pub fn instruction_count(&self, opcode: Opcode) -> u64 {
        self.instruction_counts
            .get(&(opcode as u8))
            .copied()
            .unwrap_or(0)
    }

    /// All per-opcode counts (opcode byte → count)
    pub fn instruction_counts(&self) -> &HashMap<u8, u64> {
        &self.instruction_counts
    }

    /// All per-location counts (IP → count)
    pub fn location_counts(&self) -> &HashMap<usize, u64> {
        &self.location_counts
    }

    /// The opcode recorded at a specific IP (if any)
    pub fn opcode_at(&self, ip: usize) -> Option<Opcode> {
        self.location_opcodes
            .get(&ip)
            .and_then(|&b| Opcode::try_from(b).ok())
    }

    /// Maximum call frame stack depth
    pub fn max_stack_depth(&self) -> usize {
        self.max_stack_depth
    }

    /// Maximum value stack depth
    pub fn max_value_stack_depth(&self) -> usize {
        self.max_value_stack_depth
    }

    /// Total named function calls
    pub fn function_calls(&self) -> u64 {
        self.function_calls
    }

    /// Per-function call counts
    pub fn function_call_counts(&self) -> &HashMap<String, u64> {
        &self.function_call_counts
    }

    /// Top N opcodes by execution count (sorted descending)
    pub fn top_opcodes(&self, n: usize) -> Vec<(Opcode, u64)> {
        let mut pairs: Vec<(Opcode, u64)> = self
            .instruction_counts
            .iter()
            .filter_map(|(&byte, &count)| Opcode::try_from(byte).ok().map(|op| (op, count)))
            .collect();
        pairs.sort_by(|a, b| b.1.cmp(&a.1));
        pairs.truncate(n);
        pairs
    }

    /// Top N hot locations by execution count (sorted descending)
    pub fn top_locations(&self, n: usize) -> Vec<(usize, u64)> {
        let mut pairs: Vec<(usize, u64)> = self
            .location_counts
            .iter()
            .map(|(&ip, &c)| (ip, c))
            .collect();
        pairs.sort_by(|a, b| b.1.cmp(&a.1));
        pairs.truncate(n);
        pairs
    }
}

impl Default for ProfileCollector {
    fn default() -> Self {
        Self::new()
    }
}
