//! Source location tracking and span utilities

use serde::{Deserialize, Serialize};

/// Represents a location in source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    /// Starting byte offset
    pub start: usize,
    /// Ending byte offset (exclusive)
    pub end: usize,
}

impl Span {
    /// Create a new span
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Create a dummy span for testing
    pub fn dummy() -> Self {
        Self { start: 0, end: 0 }
    }

    /// Get the length of this span
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Check if this span is empty
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Check if this span contains a byte offset
    pub fn contains(&self, offset: usize) -> bool {
        offset >= self.start && offset < self.end
    }

    /// Check if this span overlaps with another span
    pub fn overlaps(&self, other: Span) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// Combine two spans into one encompassing span
    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    /// Extend this span to include another span
    pub fn extend(&mut self, other: Span) {
        self.start = self.start.min(other.start);
        self.end = self.end.max(other.end);
    }

    /// Create a span that starts at the end of this span
    pub fn after(&self) -> Span {
        Span {
            start: self.end,
            end: self.end,
        }
    }
}
