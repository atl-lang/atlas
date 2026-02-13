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
    /// Step mode: pause after each instruction
    step_mode: bool,
}

impl Debugger {
    /// Create a new debugger (disabled by default)
    pub fn new() -> Self {
        Self {
            enabled: false,
            breakpoints: HashSet::new(),
            paused: false,
            step_mode: false,
        }
    }

    /// Create a new debugger with debugging enabled
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            breakpoints: HashSet::new(),
            paused: false,
            step_mode: false,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debugger_new() {
        let debugger = Debugger::new();
        assert!(!debugger.is_enabled());
        assert!(!debugger.is_paused());
        assert!(!debugger.is_step_mode());
    }

    #[test]
    fn test_debugger_enabled() {
        let debugger = Debugger::enabled();
        assert!(debugger.is_enabled());
    }

    #[test]
    fn test_enable_disable() {
        let mut debugger = Debugger::new();
        assert!(!debugger.is_enabled());

        debugger.enable();
        assert!(debugger.is_enabled());

        debugger.disable();
        assert!(!debugger.is_enabled());
    }

    #[test]
    fn test_breakpoints() {
        let mut debugger = Debugger::enabled();

        assert!(!debugger.has_breakpoint(10));
        debugger.set_breakpoint(10);
        assert!(debugger.has_breakpoint(10));

        debugger.set_breakpoint(20);
        assert_eq!(debugger.breakpoints().len(), 2);

        debugger.remove_breakpoint(10);
        assert!(!debugger.has_breakpoint(10));
        assert!(debugger.has_breakpoint(20));

        debugger.clear_breakpoints();
        assert_eq!(debugger.breakpoints().len(), 0);
    }

    #[test]
    fn test_step_mode() {
        let mut debugger = Debugger::enabled();
        assert!(!debugger.is_step_mode());

        debugger.enable_step_mode();
        assert!(debugger.is_step_mode());

        debugger.disable_step_mode();
        assert!(!debugger.is_step_mode());
    }

    #[test]
    fn test_before_instruction_disabled() {
        let mut debugger = Debugger::new(); // disabled
        let action = debugger.before_instruction(10, Opcode::Add);
        assert_eq!(action, DebugAction::Continue);
    }

    #[test]
    fn test_before_instruction_breakpoint() {
        let mut debugger = Debugger::enabled();
        debugger.set_breakpoint(10);

        let action = debugger.before_instruction(10, Opcode::Add);
        assert_eq!(action, DebugAction::Pause);
        assert!(debugger.is_paused());
    }

    #[test]
    fn test_before_instruction_step_mode() {
        let mut debugger = Debugger::enabled();
        debugger.enable_step_mode();

        let action = debugger.before_instruction(10, Opcode::Add);
        assert_eq!(action, DebugAction::Step);
    }

    #[test]
    fn test_before_instruction_no_breakpoint() {
        let mut debugger = Debugger::enabled();
        debugger.set_breakpoint(20);

        let action = debugger.before_instruction(10, Opcode::Add);
        assert_eq!(action, DebugAction::Continue);
        assert!(!debugger.is_paused());
    }

    #[test]
    fn test_resume() {
        let mut debugger = Debugger::enabled();
        debugger.set_breakpoint(10);
        debugger.before_instruction(10, Opcode::Add);
        assert!(debugger.is_paused());

        debugger.resume();
        assert!(!debugger.is_paused());
    }

    #[test]
    fn test_default() {
        let debugger = Debugger::default();
        assert!(!debugger.is_enabled());
    }
}
