//! Debugger state management.
//!
//! Tracks breakpoints, execution mode, step state, and the last pause event.
//! This is pure state with no VM references – the VM queries this to decide
//! whether to pause or continue.

use std::collections::HashMap;

use crate::debugger::protocol::{Breakpoint, BreakpointId, PauseReason, SourceLocation};

// ── ExecutionMode ─────────────────────────────────────────────────────────────

/// Current execution mode of the debugger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// VM is executing normally.
    Running,
    /// VM is paused (breakpoint hit, step complete, or manual pause).
    Paused,
    /// VM has finished execution (completed or errored).
    Stopped,
}

// ── StepMode ─────────────────────────────────────────────────────────────────

/// Which step operation is in progress.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepMode {
    /// No stepping – run until the next breakpoint.
    None,
    /// Pause after executing exactly one instruction (step-into semantics).
    Into,
    /// Pause when we return to the same or an outer call frame (step-over).
    Over,
    /// Pause when we return from the current call frame (step-out).
    Out,
}

// ── DebuggerState ─────────────────────────────────────────────────────────────

/// Complete mutable state of the Atlas debugger.
///
/// The VM consults this state in its execution hot-path to decide whether to
/// pause at the current instruction.  All breakpoint and step-mode mutations
/// go through this struct.
#[derive(Debug)]
pub struct DebuggerState {
    /// Current execution mode.
    pub mode: ExecutionMode,

    /// All registered breakpoints keyed by their ID.
    breakpoints: HashMap<BreakpointId, Breakpoint>,

    /// Monotonically increasing ID for new breakpoints.
    next_id: BreakpointId,

    /// Active step mode (None means run freely).
    pub step_mode: StepMode,

    /// Call-frame depth at the moment a step operation was initiated.
    /// Used by step-over (pause when depth ≤ start) and step-out
    /// (pause when depth < start).
    pub step_start_frame_depth: usize,

    /// Reason for the most recent pause.
    pub pause_reason: Option<PauseReason>,

    /// Source location of the most recent pause (if source-map is available).
    pub pause_location: Option<SourceLocation>,

    /// Instruction pointer at the most recent pause.
    pub pause_ip: usize,
}

impl DebuggerState {
    /// Create a fresh debugger state (running, no breakpoints, no step mode).
    pub fn new() -> Self {
        Self {
            mode: ExecutionMode::Running,
            breakpoints: HashMap::new(),
            next_id: 1,
            step_mode: StepMode::None,
            step_start_frame_depth: 0,
            pause_reason: None,
            pause_location: None,
            pause_ip: 0,
        }
    }

    // ── Breakpoint management ─────────────────────────────────────────────────

    /// Register a new unverified breakpoint and return its assigned ID.
    pub fn add_breakpoint(&mut self, location: SourceLocation) -> BreakpointId {
        let id = self.next_id;
        self.next_id += 1;
        self.breakpoints.insert(id, Breakpoint::new(id, location));
        id
    }

    /// Verify a breakpoint: bind it to an instruction offset.
    ///
    /// Returns `true` if the breakpoint existed and was updated.
    pub fn verify_breakpoint(&mut self, id: BreakpointId, offset: usize) -> bool {
        if let Some(bp) = self.breakpoints.get_mut(&id) {
            bp.verified = true;
            bp.instruction_offset = Some(offset);
            true
        } else {
            false
        }
    }

    /// Remove a breakpoint by ID, returning the removed entry.
    pub fn remove_breakpoint(&mut self, id: BreakpointId) -> Option<Breakpoint> {
        self.breakpoints.remove(&id)
    }

    /// Remove all registered breakpoints.
    pub fn clear_breakpoints(&mut self) {
        self.breakpoints.clear();
    }

    /// Return all breakpoints sorted by ID.
    pub fn breakpoints(&self) -> Vec<&Breakpoint> {
        let mut bps: Vec<&Breakpoint> = self.breakpoints.values().collect();
        bps.sort_by_key(|bp| bp.id);
        bps
    }

    /// Return all breakpoints as owned values, sorted by ID.
    pub fn breakpoints_owned(&self) -> Vec<Breakpoint> {
        let mut bps: Vec<Breakpoint> = self.breakpoints.values().cloned().collect();
        bps.sort_by_key(|bp| bp.id);
        bps
    }

    /// Look up a single breakpoint by ID.
    pub fn get_breakpoint(&self, id: BreakpointId) -> Option<&Breakpoint> {
        self.breakpoints.get(&id)
    }

    /// Find the first verified breakpoint bound to `offset`.
    pub fn breakpoint_at_offset(&self, offset: usize) -> Option<&Breakpoint> {
        self.breakpoints
            .values()
            .find(|bp| bp.verified && bp.instruction_offset == Some(offset))
    }

    /// Returns `true` if a verified breakpoint is bound to `offset`.
    pub fn has_breakpoint_at_offset(&self, offset: usize) -> bool {
        self.breakpoint_at_offset(offset).is_some()
    }

    /// Total number of registered breakpoints.
    pub fn breakpoint_count(&self) -> usize {
        self.breakpoints.len()
    }

    // ── Execution-mode control ────────────────────────────────────────────────

    /// Transition to Paused mode and record the pause context.
    pub fn pause(&mut self, reason: PauseReason, location: Option<SourceLocation>, ip: usize) {
        self.mode = ExecutionMode::Paused;
        self.pause_reason = Some(reason);
        self.pause_location = location;
        self.pause_ip = ip;
        self.step_mode = StepMode::None; // Clear any active step on pause
    }

    /// Transition from Paused back to Running mode.
    pub fn resume(&mut self) {
        self.mode = ExecutionMode::Running;
        self.pause_reason = None;
        self.pause_location = None;
    }

    /// Transition to Stopped mode (terminal state).
    pub fn stop(&mut self) {
        self.mode = ExecutionMode::Stopped;
        self.step_mode = StepMode::None;
    }

    /// Returns `true` if currently paused.
    pub fn is_paused(&self) -> bool {
        self.mode == ExecutionMode::Paused
    }

    /// Returns `true` if currently running.
    pub fn is_running(&self) -> bool {
        self.mode == ExecutionMode::Running
    }

    /// Returns `true` if execution has stopped.
    pub fn is_stopped(&self) -> bool {
        self.mode == ExecutionMode::Stopped
    }

    // ── Step-mode control ─────────────────────────────────────────────────────

    /// Enter a step mode, recording the current call-frame depth.
    pub fn set_step_mode(&mut self, mode: StepMode, current_frame_depth: usize) {
        self.step_mode = mode;
        self.step_start_frame_depth = current_frame_depth;
        // Resume execution so the VM keeps running until the step condition fires.
        self.mode = ExecutionMode::Running;
        self.pause_reason = None;
    }

    /// Clear the active step mode (revert to free-running).
    pub fn clear_step_mode(&mut self) {
        self.step_mode = StepMode::None;
    }

    // ── Step-pause logic ──────────────────────────────────────────────────────

    /// Decide whether to pause at `ip` for the current step mode.
    ///
    /// `current_frame_depth` is `frames.len()` in the VM.
    pub fn should_pause_for_step(&self, current_frame_depth: usize) -> bool {
        match self.step_mode {
            StepMode::None => false,
            // Step-into: pause at every instruction.
            StepMode::Into => true,
            // Step-over: pause when we're back to the same or shallower frame.
            StepMode::Over => current_frame_depth <= self.step_start_frame_depth,
            // Step-out: pause when we've returned from the start frame.
            StepMode::Out => current_frame_depth < self.step_start_frame_depth,
        }
    }
}

impl Default for DebuggerState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_location(line: u32) -> SourceLocation {
        SourceLocation::new("test.atlas", line, 1)
    }

    // ── Initial state ────────────────────────────────────────────────────────

    #[test]
    fn test_initial_mode_is_running() {
        let state = DebuggerState::new();
        assert!(state.is_running());
        assert!(!state.is_paused());
        assert!(!state.is_stopped());
    }

    #[test]
    fn test_initial_no_breakpoints() {
        let state = DebuggerState::new();
        assert_eq!(state.breakpoint_count(), 0);
        assert!(state.breakpoints().is_empty());
    }

    #[test]
    fn test_initial_step_mode_none() {
        let state = DebuggerState::new();
        assert_eq!(state.step_mode, StepMode::None);
    }

    // ── Breakpoints ──────────────────────────────────────────────────────────

    #[test]
    fn test_add_breakpoint_assigns_sequential_ids() {
        let mut state = DebuggerState::new();
        let id1 = state.add_breakpoint(make_location(5));
        let id2 = state.add_breakpoint(make_location(10));
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
    }

    #[test]
    fn test_add_breakpoint_starts_unverified() {
        let mut state = DebuggerState::new();
        let id = state.add_breakpoint(make_location(5));
        let bp = state.get_breakpoint(id).unwrap();
        assert!(!bp.verified);
        assert!(bp.instruction_offset.is_none());
    }

    #[test]
    fn test_verify_breakpoint() {
        let mut state = DebuggerState::new();
        let id = state.add_breakpoint(make_location(5));
        assert!(state.verify_breakpoint(id, 42));
        let bp = state.get_breakpoint(id).unwrap();
        assert!(bp.verified);
        assert_eq!(bp.instruction_offset, Some(42));
    }

    #[test]
    fn test_verify_nonexistent_breakpoint_returns_false() {
        let mut state = DebuggerState::new();
        assert!(!state.verify_breakpoint(99, 0));
    }

    #[test]
    fn test_has_breakpoint_at_offset() {
        let mut state = DebuggerState::new();
        let id = state.add_breakpoint(make_location(5));
        state.verify_breakpoint(id, 42);
        assert!(state.has_breakpoint_at_offset(42));
        assert!(!state.has_breakpoint_at_offset(43));
    }

    #[test]
    fn test_unverified_breakpoint_not_hit() {
        let mut state = DebuggerState::new();
        state.add_breakpoint(make_location(5)); // not verified
        assert!(!state.has_breakpoint_at_offset(0));
    }

    #[test]
    fn test_remove_breakpoint() {
        let mut state = DebuggerState::new();
        let id = state.add_breakpoint(make_location(5));
        assert!(state.remove_breakpoint(id).is_some());
        assert_eq!(state.breakpoint_count(), 0);
    }

    #[test]
    fn test_remove_nonexistent_breakpoint_returns_none() {
        let mut state = DebuggerState::new();
        assert!(state.remove_breakpoint(99).is_none());
    }

    #[test]
    fn test_clear_breakpoints() {
        let mut state = DebuggerState::new();
        state.add_breakpoint(make_location(1));
        state.add_breakpoint(make_location(2));
        state.add_breakpoint(make_location(3));
        state.clear_breakpoints();
        assert_eq!(state.breakpoint_count(), 0);
    }

    #[test]
    fn test_breakpoints_sorted_by_id() {
        let mut state = DebuggerState::new();
        let id_a = state.add_breakpoint(make_location(10)); // gets id 1
        let id_b = state.add_breakpoint(make_location(5)); // gets id 2
        let id_c = state.add_breakpoint(make_location(7)); // gets id 3
                                                           // breakpoints() returns them sorted ascending by their integer ID
        let ids: Vec<BreakpointId> = state.breakpoints().iter().map(|bp| bp.id).collect();
        assert_eq!(ids, vec![id_a, id_b, id_c]); // sorted: 1, 2, 3
        assert!(ids[0] < ids[1]);
        assert!(ids[1] < ids[2]);
    }

    // ── Execution mode ───────────────────────────────────────────────────────

    #[test]
    fn test_pause_transitions_to_paused() {
        let mut state = DebuggerState::new();
        state.pause(PauseReason::Step, Some(make_location(3)), 10);
        assert!(state.is_paused());
        assert_eq!(state.pause_ip, 10);
        assert_eq!(state.pause_reason, Some(PauseReason::Step));
    }

    #[test]
    fn test_resume_transitions_to_running() {
        let mut state = DebuggerState::new();
        state.pause(PauseReason::ManualPause, None, 0);
        state.resume();
        assert!(state.is_running());
        assert!(state.pause_reason.is_none());
    }

    #[test]
    fn test_stop_transitions_to_stopped() {
        let mut state = DebuggerState::new();
        state.stop();
        assert!(state.is_stopped());
    }

    // ── Step mode ────────────────────────────────────────────────────────────

    #[test]
    fn test_set_step_into_pauses_immediately() {
        let mut state = DebuggerState::new();
        state.set_step_mode(StepMode::Into, 1);
        assert!(state.should_pause_for_step(1)); // any depth
        assert!(state.should_pause_for_step(2));
    }

    #[test]
    fn test_step_over_pauses_at_same_depth() {
        let mut state = DebuggerState::new();
        state.set_step_mode(StepMode::Over, 2);
        assert!(state.should_pause_for_step(2)); // same depth
        assert!(state.should_pause_for_step(1)); // shallower
        assert!(!state.should_pause_for_step(3)); // deeper (inside call)
    }

    #[test]
    fn test_step_out_pauses_at_shallower_depth() {
        let mut state = DebuggerState::new();
        state.set_step_mode(StepMode::Out, 3);
        assert!(state.should_pause_for_step(2)); // returned to outer frame
        assert!(state.should_pause_for_step(1)); // even more outer
        assert!(!state.should_pause_for_step(3)); // same depth – not yet out
        assert!(!state.should_pause_for_step(4)); // deeper
    }

    #[test]
    fn test_no_step_mode_never_pauses() {
        let state = DebuggerState::new();
        assert!(!state.should_pause_for_step(0));
        assert!(!state.should_pause_for_step(5));
    }

    #[test]
    fn test_pause_clears_step_mode() {
        let mut state = DebuggerState::new();
        state.set_step_mode(StepMode::Into, 1);
        state.pause(PauseReason::Step, None, 5);
        assert_eq!(state.step_mode, StepMode::None);
    }
}
