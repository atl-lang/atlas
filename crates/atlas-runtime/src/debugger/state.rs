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
