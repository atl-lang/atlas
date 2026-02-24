//! Advanced breakpoint management for the Atlas debugger.
//!
//! Provides conditional breakpoints, hit counts, enable/disable toggling,
//! and log points (breakpoints that log instead of pausing).

use std::collections::HashMap;

use crate::debugger::protocol::{Breakpoint, BreakpointId, SourceLocation};

// ── BreakpointCondition ──────────────────────────────────────────────────────

/// A condition that must be true for a breakpoint to fire.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum BreakpointCondition {
    /// Always fire (unconditional breakpoint).
    #[default]
    Always,
    /// Fire only when the given Atlas expression evaluates to a truthy value.
    Expression(String),
    /// Fire only when the hit count reaches this threshold.
    HitCount(u64),
    /// Fire when hit count is a multiple of this value.
    HitCountMultiple(u64),
}

// ── BreakpointEntry ──────────────────────────────────────────────────────────

/// Extended breakpoint with condition, hit count, and enable/disable state.
#[derive(Debug, Clone)]
pub struct BreakpointEntry {
    /// The protocol-level breakpoint (id, location, verified, offset).
    pub breakpoint: Breakpoint,
    /// Whether this breakpoint is enabled (can be toggled without removing).
    pub enabled: bool,
    /// Condition that must be met for the breakpoint to fire.
    pub condition: BreakpointCondition,
    /// Number of times execution has reached this breakpoint's offset.
    pub hit_count: u64,
    /// Optional log message (log point) — if set, logs instead of pausing.
    pub log_message: Option<String>,
}

impl BreakpointEntry {
    /// Create a new unconditional, enabled breakpoint entry.
    pub fn new(breakpoint: Breakpoint) -> Self {
        Self {
            breakpoint,
            enabled: true,
            condition: BreakpointCondition::Always,
            hit_count: 0,
            log_message: None,
        }
    }

    /// Create a conditional breakpoint entry.
    pub fn with_condition(breakpoint: Breakpoint, condition: BreakpointCondition) -> Self {
        Self {
            breakpoint,
            enabled: true,
            condition,
            hit_count: 0,
            log_message: None,
        }
    }

    /// Create a log point (logs message instead of pausing).
    pub fn log_point(breakpoint: Breakpoint, message: String) -> Self {
        Self {
            breakpoint,
            enabled: true,
            condition: BreakpointCondition::Always,
            hit_count: 0,
            log_message: Some(message),
        }
    }

    /// Returns true if this breakpoint is a log point.
    pub fn is_log_point(&self) -> bool {
        self.log_message.is_some()
    }

    /// Increment hit count and return whether this breakpoint should fire.
    ///
    /// Returns `ShouldFire::Pause` for normal breakpoints,
    /// `ShouldFire::Log(msg)` for log points, or `ShouldFire::Skip`
    /// if the condition is not met.
    pub fn check_and_increment(&mut self) -> ShouldFire {
        if !self.enabled {
            return ShouldFire::Skip;
        }
        if !self.breakpoint.verified {
            return ShouldFire::Skip;
        }

        self.hit_count += 1;

        let condition_met = match &self.condition {
            BreakpointCondition::Always => true,
            BreakpointCondition::Expression(_) => {
                // Expression conditions require evaluation by the caller.
                // Return true here; the caller handles expression evaluation.
                true
            }
            BreakpointCondition::HitCount(target) => self.hit_count >= *target,
            BreakpointCondition::HitCountMultiple(n) => *n > 0 && self.hit_count.is_multiple_of(*n),
        };

        if !condition_met {
            return ShouldFire::Skip;
        }

        if let Some(ref msg) = self.log_message {
            ShouldFire::Log(msg.clone())
        } else {
            ShouldFire::Pause
        }
    }

    /// Reset the hit count to zero.
    pub fn reset_hit_count(&mut self) {
        self.hit_count = 0;
    }
}

// ── ShouldFire ───────────────────────────────────────────────────────────────

/// The result of checking whether a breakpoint should fire.
#[derive(Debug, Clone, PartialEq)]
pub enum ShouldFire {
    /// The breakpoint should pause execution.
    Pause,
    /// The breakpoint should log a message without pausing.
    Log(String),
    /// The breakpoint should be skipped (condition not met or disabled).
    Skip,
}

// ── BreakpointManager ────────────────────────────────────────────────────────

/// Manages all breakpoints with advanced features.
///
/// Wraps the simpler `DebuggerState` breakpoint storage with condition
/// evaluation, hit counting, and log point support.
#[derive(Debug, Default)]
pub struct BreakpointManager {
    /// Breakpoint entries keyed by ID.
    entries: HashMap<BreakpointId, BreakpointEntry>,
    /// Next ID to assign.
    next_id: BreakpointId,
    /// Reverse index: instruction offset → breakpoint IDs.
    offset_index: HashMap<usize, Vec<BreakpointId>>,
    /// Log messages accumulated during execution.
    log_output: Vec<String>,
}

impl BreakpointManager {
    /// Create a new empty breakpoint manager.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            next_id: 1,
            offset_index: HashMap::new(),
            log_output: Vec::new(),
        }
    }

    /// Add a simple breakpoint and return its ID.
    pub fn add(&mut self, location: SourceLocation) -> BreakpointId {
        let id = self.next_id;
        self.next_id += 1;
        let bp = Breakpoint::new(id, location);
        self.entries.insert(id, BreakpointEntry::new(bp));
        id
    }

    /// Add a conditional breakpoint.
    pub fn add_conditional(
        &mut self,
        location: SourceLocation,
        condition: BreakpointCondition,
    ) -> BreakpointId {
        let id = self.next_id;
        self.next_id += 1;
        let bp = Breakpoint::new(id, location);
        self.entries
            .insert(id, BreakpointEntry::with_condition(bp, condition));
        id
    }

    /// Add a log point.
    pub fn add_log_point(&mut self, location: SourceLocation, message: String) -> BreakpointId {
        let id = self.next_id;
        self.next_id += 1;
        let bp = Breakpoint::new(id, location);
        self.entries
            .insert(id, BreakpointEntry::log_point(bp, message));
        id
    }

    /// Verify (bind) a breakpoint to an instruction offset.
    pub fn verify(&mut self, id: BreakpointId, offset: usize) -> bool {
        if let Some(entry) = self.entries.get_mut(&id) {
            entry.breakpoint.verified = true;
            entry.breakpoint.instruction_offset = Some(offset);
            self.offset_index.entry(offset).or_default().push(id);
            true
        } else {
            false
        }
    }

    /// Remove a breakpoint by ID.
    pub fn remove(&mut self, id: BreakpointId) -> Option<BreakpointEntry> {
        if let Some(entry) = self.entries.remove(&id) {
            // Clean up offset index
            if let Some(offset) = entry.breakpoint.instruction_offset {
                if let Some(ids) = self.offset_index.get_mut(&offset) {
                    ids.retain(|&bid| bid != id);
                    if ids.is_empty() {
                        self.offset_index.remove(&offset);
                    }
                }
            }
            Some(entry)
        } else {
            None
        }
    }

    /// Remove all breakpoints.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.offset_index.clear();
    }

    /// Enable a breakpoint by ID.
    pub fn enable(&mut self, id: BreakpointId) -> bool {
        if let Some(entry) = self.entries.get_mut(&id) {
            entry.enabled = true;
            true
        } else {
            false
        }
    }

    /// Disable a breakpoint by ID (keeps it registered but won't fire).
    pub fn disable(&mut self, id: BreakpointId) -> bool {
        if let Some(entry) = self.entries.get_mut(&id) {
            entry.enabled = false;
            true
        } else {
            false
        }
    }

    /// Set a condition on an existing breakpoint.
    pub fn set_condition(&mut self, id: BreakpointId, condition: BreakpointCondition) -> bool {
        if let Some(entry) = self.entries.get_mut(&id) {
            entry.condition = condition;
            true
        } else {
            false
        }
    }

    /// Get a breakpoint entry by ID.
    pub fn get(&self, id: BreakpointId) -> Option<&BreakpointEntry> {
        self.entries.get(&id)
    }

    /// Get a mutable breakpoint entry by ID.
    pub fn get_mut(&mut self, id: BreakpointId) -> Option<&mut BreakpointEntry> {
        self.entries.get_mut(&id)
    }

    /// Check if any breakpoint exists at the given offset.
    pub fn has_breakpoint_at(&self, offset: usize) -> bool {
        self.offset_index.contains_key(&offset)
    }

    /// Check all breakpoints at the given offset. Returns the action to take.
    ///
    /// Increments hit counts and evaluates conditions.
    pub fn check_offset(&mut self, offset: usize) -> ShouldFire {
        let ids = match self.offset_index.get(&offset) {
            Some(ids) => ids.clone(),
            None => return ShouldFire::Skip,
        };

        for id in ids {
            if let Some(entry) = self.entries.get_mut(&id) {
                match entry.check_and_increment() {
                    ShouldFire::Pause => return ShouldFire::Pause,
                    ShouldFire::Log(msg) => {
                        self.log_output.push(msg);
                        // Log points don't pause — continue checking
                    }
                    ShouldFire::Skip => {}
                }
            }
        }

        ShouldFire::Skip
    }

    /// Drain accumulated log output.
    pub fn drain_log_output(&mut self) -> Vec<String> {
        std::mem::take(&mut self.log_output)
    }

    /// Total number of breakpoints.
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Number of enabled breakpoints.
    pub fn enabled_count(&self) -> usize {
        self.entries.values().filter(|e| e.enabled).count()
    }

    /// Get all entries sorted by ID.
    pub fn all_entries(&self) -> Vec<&BreakpointEntry> {
        let mut entries: Vec<&BreakpointEntry> = self.entries.values().collect();
        entries.sort_by_key(|e| e.breakpoint.id);
        entries
    }

    /// Get all protocol-level breakpoints sorted by ID.
    pub fn all_breakpoints(&self) -> Vec<Breakpoint> {
        self.all_entries()
            .iter()
            .map(|e| e.breakpoint.clone())
            .collect()
    }

    /// Reset all hit counts.
    pub fn reset_all_hit_counts(&mut self) {
        for entry in self.entries.values_mut() {
            entry.reset_hit_count();
        }
    }
}
