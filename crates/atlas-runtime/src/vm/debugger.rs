//! VM debugger hooks
//!
//! Provides optional debugging capabilities for step-through execution,
//! breakpoints, and state inspection. Disabled by default for production use.

use crate::bytecode::Opcode;
use std::collections::HashSet;

/// Debugger action to take after a hook callback
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugAction {
    /// Continue normal execution
    Continue,
    /// Step to next instruction
    Step,
    /// Halt execution (for breakpoint pause)
    Pause,
}

/// Frame-depth-aware step condition for the VM debugger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum StepCondition {
    /// No stepping active.
    None,
    /// Pause at every instruction (step-into semantics).
    Always,
    /// Pause when call-frame depth ≤ the recorded start depth (step-over).
    OverDepth(usize),
    /// Pause when call-frame depth < the recorded start depth (step-out).
    OutDepth(usize),
}

impl StepCondition {
    /// Returns `true` if this condition fires at `current_depth`.
    pub fn fires(&self, current_depth: usize) -> bool {
        match self {
            Self::None => false,
            Self::Always => true,
            Self::OverDepth(start) => current_depth <= *start,
            Self::OutDepth(start) => current_depth < *start,
        }
    }
}

/// Debugger hook trait for custom debugging callbacks
///
/// Implement this trait to create custom debugging tools.
pub trait DebugHook {
    /// Called before each instruction is executed
    ///
    /// # Arguments
    /// * `ip` - Current instruction pointer
    /// * `opcode` - Opcode about to be executed
    ///
    /// # Returns
    /// The action to take (Continue, Step, or Pause)
    fn before_instruction(&mut self, ip: usize, opcode: Opcode) -> DebugAction;

    /// Called after each instruction is executed
    ///
    /// # Arguments
    /// * `ip` - Instruction pointer after execution
    fn after_instruction(&mut self, ip: usize);
}

/// VM debugger for step-through execution and breakpoints
///
/// Provides debugging capabilities without affecting production performance.
/// Disabled by default.
#[derive(Debug, Clone)]
pub struct Debugger {
    /// Whether debugging is enabled
    enabled: bool,
    /// Breakpoints (instruction offsets)
    breakpoints: HashSet<usize>,
    /// Whether VM is currently paused at a breakpoint
    paused: bool,
    /// Simple step mode: pause after each instruction (legacy)
    step_mode: bool,
    /// Frame-depth-aware step condition (overrides step_mode when not None)
    step_condition: StepCondition,
}

impl Debugger {
    /// Create a new debugger (disabled by default)
    pub fn new() -> Self {
        Self {
            enabled: false,
            breakpoints: HashSet::new(),
            paused: false,
            step_mode: false,
            step_condition: StepCondition::None,
        }
    }

    /// Create a new debugger with debugging enabled
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            breakpoints: HashSet::new(),
            paused: false,
            step_mode: false,
            step_condition: StepCondition::None,
        }
    }

    /// Enable debugging
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable debugging
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check if debugging is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Set a breakpoint at the given instruction offset
    pub fn set_breakpoint(&mut self, offset: usize) {
        self.breakpoints.insert(offset);
    }

    /// Remove a breakpoint at the given instruction offset
    pub fn remove_breakpoint(&mut self, offset: usize) {
        self.breakpoints.remove(&offset);
    }

    /// Clear all breakpoints
    pub fn clear_breakpoints(&mut self) {
        self.breakpoints.clear();
    }

    /// Check if there's a breakpoint at the given offset
    pub fn has_breakpoint(&self, offset: usize) -> bool {
        self.breakpoints.contains(&offset)
    }

    /// Get all breakpoints
    pub fn breakpoints(&self) -> &HashSet<usize> {
        &self.breakpoints
    }

    /// Enable step mode (pause after each instruction)
    pub fn enable_step_mode(&mut self) {
        self.step_mode = true;
    }

    /// Disable step mode
    pub fn disable_step_mode(&mut self) {
        self.step_mode = false;
    }

    /// Check if step mode is enabled
    pub fn is_step_mode(&self) -> bool {
        self.step_mode
    }

    /// Check if VM is currently paused
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Resume execution from a paused state
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Get the active step condition.
    pub fn step_condition(&self) -> StepCondition {
        self.step_condition
    }

    /// Set a frame-depth-aware step condition.
    ///
    /// When set, this overrides the simple `step_mode` flag.
    pub fn set_step_condition(&mut self, condition: StepCondition) {
        self.step_condition = condition;
        // Keep legacy step_mode in sync for introspection.
        self.step_mode = !matches!(condition, StepCondition::None);
    }

    /// Clear the active step condition (revert to free-running).
    pub fn clear_step_condition(&mut self) {
        self.step_condition = StepCondition::None;
        self.step_mode = false;
    }

    /// Hook called before instruction execution
    ///
    /// Returns the action to take (Continue, Step, or Pause).
    pub fn before_instruction(&mut self, ip: usize, _opcode: Opcode) -> DebugAction {
        if !self.enabled {
            return DebugAction::Continue;
        }

        // Check for breakpoint
        if self.has_breakpoint(ip) {
            self.paused = true;
            return DebugAction::Pause;
        }

        // Check step mode
        if self.step_mode {
            return DebugAction::Step;
        }

        DebugAction::Continue
    }

    /// Frame-depth-aware variant of `before_instruction`.
    ///
    /// This is the preferred hook when running with the full debugger infrastructure,
    /// as it supports step-over and step-out semantics.
    pub fn before_instruction_with_depth(
        &mut self,
        ip: usize,
        _opcode: Opcode,
        frame_depth: usize,
    ) -> DebugAction {
        if !self.enabled {
            return DebugAction::Continue;
        }

        // Breakpoints always take priority
        if self.has_breakpoint(ip) {
            self.paused = true;
            return DebugAction::Pause;
        }

        // Frame-depth-aware step condition overrides simple step_mode
        if !matches!(self.step_condition, StepCondition::None) {
            if self.step_condition.fires(frame_depth) {
                return DebugAction::Pause;
            }
            return DebugAction::Continue;
        }

        // Legacy simple step mode
        if self.step_mode {
            return DebugAction::Step;
        }

        DebugAction::Continue
    }

    /// Hook called after instruction execution
    pub fn after_instruction(&mut self, _ip: usize) {
        // Future: Could track execution history, etc.
    }
}

impl Default for Debugger {
    fn default() -> Self {
        Self::new()
    }
}
