//! Hotspot detection for JIT compilation
//!
//! Bridges the VM profiler's hotspot data with JIT compilation decisions.
//! Identifies functions that have been called enough times to warrant
//! compilation to native code.

use atlas_runtime::bytecode::{Bytecode, Opcode};
use std::collections::HashMap;

/// Tracks function execution counts and identifies compilation candidates
#[derive(Debug)]
pub struct HotspotTracker {
    /// Execution count per function (keyed by bytecode offset)
    function_counts: HashMap<usize, u64>,
    /// Threshold for JIT compilation
    threshold: u64,
    /// Functions already compiled (don't recompile)
    compiled: HashMap<usize, bool>,
}

impl HotspotTracker {
    /// Create a new tracker with the given compilation threshold
    pub fn new(threshold: u64) -> Self {
        Self {
            function_counts: HashMap::new(),
            threshold,
            compiled: HashMap::new(),
        }
    }

    /// Record a function call at the given bytecode offset
    pub fn record_call(&mut self, function_offset: usize) {
        *self.function_counts.entry(function_offset).or_insert(0) += 1;
    }

    /// Get the execution count for a function
    pub fn call_count(&self, function_offset: usize) -> u64 {
        self.function_counts
            .get(&function_offset)
            .copied()
            .unwrap_or(0)
    }

    /// Check if a function is hot enough for JIT compilation
    pub fn is_hot(&self, function_offset: usize) -> bool {
        self.call_count(function_offset) >= self.threshold && !self.is_compiled(function_offset)
    }

    /// Mark a function as compiled
    pub fn mark_compiled(&mut self, function_offset: usize) {
        self.compiled.insert(function_offset, true);
    }

    /// Check if a function has already been compiled
    pub fn is_compiled(&self, function_offset: usize) -> bool {
        self.compiled
            .get(&function_offset)
            .copied()
            .unwrap_or(false)
    }

    /// Get all hot functions that need compilation, sorted by call count (highest first)
    pub fn pending_compilations(&self) -> Vec<HotFunction> {
        let mut hot: Vec<HotFunction> = self
            .function_counts
            .iter()
            .filter(|(&offset, &count)| count >= self.threshold && !self.is_compiled(offset))
            .map(|(&offset, &count)| HotFunction { offset, count })
            .collect();
        hot.sort_by(|a, b| b.count.cmp(&a.count));
        hot
    }

    /// Get the compilation threshold
    pub fn threshold(&self) -> u64 {
        self.threshold
    }

    /// Set a new compilation threshold
    pub fn set_threshold(&mut self, threshold: u64) {
        self.threshold = threshold;
    }

    /// Reset all tracking data
    pub fn reset(&mut self) {
        self.function_counts.clear();
        self.compiled.clear();
    }

    /// Total number of tracked functions
    pub fn tracked_count(&self) -> usize {
        self.function_counts.len()
    }

    /// Number of compiled functions
    pub fn compiled_count(&self) -> usize {
        self.compiled.len()
    }
}

/// A function identified as hot (candidate for JIT compilation)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotFunction {
    /// Bytecode offset where the function starts
    pub offset: usize,
    /// Number of times the function has been called
    pub count: u64,
}

/// Extract function boundaries from bytecode
///
/// Scans bytecode for function entry points (Call targets) and returns
/// a map of offset -> function info.
pub fn extract_function_boundaries(bytecode: &Bytecode) -> Vec<FunctionBoundary> {
    let mut boundaries = Vec::new();
    let instructions = &bytecode.instructions;
    let mut ip = 0;

    // Scan for function-like patterns: sequences ending in Return
    // In Atlas bytecode, functions are defined via Constant opcodes that
    // reference FunctionRef values. We track where functions start/end.
    let mut function_start: Option<usize> = None;

    while ip < instructions.len() {
        let byte = instructions[ip];
        if let Ok(opcode) = Opcode::try_from(byte) {
            match opcode {
                // Function constants indicate a function definition nearby
                Opcode::Constant => {
                    let idx = if ip + 2 < instructions.len() {
                        ((instructions[ip + 1] as u16) << 8) | (instructions[ip + 2] as u16)
                    } else {
                        0
                    };
                    // Check if the constant is a FunctionRef
                    if let Some(atlas_runtime::value::Value::Function(fref)) =
                        bytecode.constants.get(idx as usize)
                    {
                        if fref.bytecode_offset > 0 {
                            function_start = Some(fref.bytecode_offset);
                        }
                    }
                    ip += 3;
                }
                Opcode::Return => {
                    if let Some(start) = function_start.take() {
                        boundaries.push(FunctionBoundary { start, end: ip + 1 });
                    }
                    ip += 1;
                }
                // Skip operands for opcodes that have them
                Opcode::GetLocal
                | Opcode::SetLocal
                | Opcode::GetGlobal
                | Opcode::SetGlobal
                | Opcode::Jump
                | Opcode::JumpIfFalse
                | Opcode::Loop
                | Opcode::Array => {
                    ip += 3; // opcode + u16
                }
                Opcode::Call => {
                    ip += 2; // opcode + u8
                }
                _ => {
                    ip += 1;
                }
            }
        } else {
            ip += 1;
        }
    }

    boundaries
}

/// A function's bytecode range
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionBoundary {
    /// Start offset in bytecode
    pub start: usize,
    /// End offset (exclusive)
    pub end: usize,
}

impl FunctionBoundary {
    /// Size in bytes
    pub fn size(&self) -> usize {
        self.end.saturating_sub(self.start)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_new() {
        let tracker = HotspotTracker::new(100);
        assert_eq!(tracker.threshold(), 100);
        assert_eq!(tracker.tracked_count(), 0);
        assert_eq!(tracker.compiled_count(), 0);
    }

    #[test]
    fn test_record_call() {
        let mut tracker = HotspotTracker::new(10);
        tracker.record_call(42);
        assert_eq!(tracker.call_count(42), 1);
        tracker.record_call(42);
        assert_eq!(tracker.call_count(42), 2);
        assert_eq!(tracker.call_count(99), 0);
    }

    #[test]
    fn test_is_hot() {
        let mut tracker = HotspotTracker::new(3);
        tracker.record_call(10);
        tracker.record_call(10);
        assert!(!tracker.is_hot(10));
        tracker.record_call(10);
        assert!(tracker.is_hot(10));
    }

    #[test]
    fn test_compiled_not_hot() {
        let mut tracker = HotspotTracker::new(2);
        tracker.record_call(10);
        tracker.record_call(10);
        assert!(tracker.is_hot(10));
        tracker.mark_compiled(10);
        assert!(!tracker.is_hot(10));
        assert!(tracker.is_compiled(10));
    }

    #[test]
    fn test_pending_compilations() {
        let mut tracker = HotspotTracker::new(2);
        // Function at offset 10: called 5 times
        for _ in 0..5 {
            tracker.record_call(10);
        }
        // Function at offset 20: called 3 times
        for _ in 0..3 {
            tracker.record_call(20);
        }
        // Function at offset 30: called 1 time (below threshold)
        tracker.record_call(30);

        let pending = tracker.pending_compilations();
        assert_eq!(pending.len(), 2);
        assert_eq!(pending[0].offset, 10); // highest first
        assert_eq!(pending[0].count, 5);
        assert_eq!(pending[1].offset, 20);
        assert_eq!(pending[1].count, 3);
    }

    #[test]
    fn test_reset() {
        let mut tracker = HotspotTracker::new(2);
        tracker.record_call(10);
        tracker.mark_compiled(10);
        tracker.reset();
        assert_eq!(tracker.tracked_count(), 0);
        assert_eq!(tracker.compiled_count(), 0);
    }
}
