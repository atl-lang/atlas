//! Source-line-aware stepping for the Atlas debugger.
//!
//! Extends the basic instruction-level stepping with line-level awareness,
//! ensuring step-over/step-into/step-out operate on source lines rather
//! than individual bytecode instructions.

use crate::debugger::protocol::{PauseReason, SourceLocation};
use crate::debugger::source_map::SourceMap;

// ── StepRequest ──────────────────────────────────────────────────────────────

/// A step operation requested by the debugger client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepRequest {
    /// Step into: pause at the next instruction (descend into calls).
    Into,
    /// Step over: pause at the next source line at the same or shallower depth.
    Over,
    /// Step out: pause when the current function returns.
    Out,
    /// Run to a specific source line.
    RunToLine { file: String, line: u32 },
    /// Run to a specific instruction offset.
    RunToOffset(usize),
}

// ── StepTracker ──────────────────────────────────────────────────────────────

/// Tracks step state for source-line-aware stepping.
///
/// The VM calls `should_pause` before each instruction. The tracker determines
/// whether execution should stop based on the current step request, frame depth,
/// and source line changes.
#[derive(Debug)]
pub struct StepTracker {
    /// The active step request (None = free running).
    active_request: Option<StepRequest>,
    /// Frame depth when the step was initiated.
    start_frame_depth: usize,
    /// Source line at the start of the step (for line-level step-over).
    start_line: Option<u32>,
    /// Source file at the start of the step.
    start_file: Option<String>,
    /// Number of instructions executed since the step started.
    instructions_since_step: u64,
    /// Maximum instructions before force-pausing (safety limit).
    max_instructions: u64,
}

impl StepTracker {
    /// Create a new step tracker with no active step request.
    pub fn new() -> Self {
        Self {
            active_request: None,
            start_frame_depth: 0,
            start_line: None,
            start_file: None,
            instructions_since_step: 0,
            max_instructions: 1_000_000,
        }
    }

    /// Start a step operation.
    ///
    /// `frame_depth`: current call frame depth.
    /// `current_location`: the source location where the step starts.
    pub fn begin_step(
        &mut self,
        request: StepRequest,
        frame_depth: usize,
        current_location: Option<&SourceLocation>,
    ) {
        self.start_frame_depth = frame_depth;
        self.start_line = current_location.map(|l| l.line);
        self.start_file = current_location.map(|l| l.file.clone());
        self.instructions_since_step = 0;
        self.active_request = Some(request);
    }

    /// Cancel the active step request.
    pub fn cancel(&mut self) {
        self.active_request = None;
        self.instructions_since_step = 0;
    }

    /// Returns `true` if a step operation is active.
    pub fn is_stepping(&self) -> bool {
        self.active_request.is_some()
    }

    /// Get the active step request.
    pub fn active_request(&self) -> Option<&StepRequest> {
        self.active_request.as_ref()
    }

    /// Get the frame depth when the step started.
    pub fn start_depth(&self) -> usize {
        self.start_frame_depth
    }

    /// Get instructions executed since step started.
    pub fn instructions_executed(&self) -> u64 {
        self.instructions_since_step
    }

    /// Set the maximum instruction limit for safety.
    pub fn set_max_instructions(&mut self, max: u64) {
        self.max_instructions = max;
    }

    /// Determine whether execution should pause at this instruction.
    ///
    /// Returns `Some(PauseReason)` if the step condition is met, `None` otherwise.
    ///
    /// Parameters:
    /// - `ip`: current instruction pointer
    /// - `frame_depth`: current call frame depth
    /// - `source_map`: for resolving IP to source location
    pub fn should_pause(
        &mut self,
        ip: usize,
        frame_depth: usize,
        source_map: &SourceMap,
    ) -> Option<PauseReason> {
        let request = match &self.active_request {
            Some(r) => r.clone(),
            None => return None,
        };

        self.instructions_since_step += 1;

        // Safety limit: force pause after too many instructions
        if self.instructions_since_step > self.max_instructions {
            self.active_request = None;
            return Some(PauseReason::Step);
        }

        let current_location = source_map.location_for_offset(ip);
        let current_line = current_location.map(|l| l.line);
        let current_file = current_location.map(|l| l.file.as_str());

        match request {
            StepRequest::Into => {
                // Step-into: pause at the first instruction on a different source line,
                // OR immediately if we have no line info.
                if self.start_line.is_none() || current_line.is_none() {
                    self.active_request = None;
                    return Some(PauseReason::Step);
                }
                let on_new_line = current_line != self.start_line
                    || current_file.map(|f| f.to_string()) != self.start_file;
                if on_new_line {
                    self.active_request = None;
                    return Some(PauseReason::Step);
                }
                None
            }
            StepRequest::Over => {
                // Step-over: pause at next source line at same or shallower depth.
                if frame_depth > self.start_frame_depth {
                    return None; // Still inside a call — keep running
                }
                // At same or shallower depth — check for line change
                if self.start_line.is_none() || current_line.is_none() {
                    self.active_request = None;
                    return Some(PauseReason::Step);
                }
                let on_new_line = current_line != self.start_line
                    || current_file.map(|f| f.to_string()) != self.start_file;
                if on_new_line {
                    self.active_request = None;
                    return Some(PauseReason::Step);
                }
                None
            }
            StepRequest::Out => {
                // Step-out: pause when frame depth decreases below start.
                if frame_depth < self.start_frame_depth {
                    self.active_request = None;
                    return Some(PauseReason::Step);
                }
                None
            }
            StepRequest::RunToLine { ref file, line } => {
                if let Some(loc) = current_location {
                    if loc.line == line && loc.file == *file {
                        self.active_request = None;
                        return Some(PauseReason::Step);
                    }
                }
                None
            }
            StepRequest::RunToOffset(target) => {
                if ip == target {
                    self.active_request = None;
                    return Some(PauseReason::Step);
                }
                None
            }
        }
    }
}

impl Default for StepTracker {
    fn default() -> Self {
        Self::new()
    }
}
