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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_creation() {
        let span = Span::new(0, 10);
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 10);
    }

    #[test]
    fn test_span_merge() {
        let span1 = Span::new(0, 5);
        let span2 = Span::new(3, 10);
        let merged = span1.merge(span2);
        assert_eq!(merged.start, 0);
        assert_eq!(merged.end, 10);
    }

    #[test]
    fn test_span_len() {
        let span = Span::new(5, 10);
        assert_eq!(span.len(), 5);

        let empty = Span::new(5, 5);
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn test_span_is_empty() {
        let empty = Span::new(5, 5);
        assert!(empty.is_empty());

        let not_empty = Span::new(5, 10);
        assert!(!not_empty.is_empty());
    }

    #[test]
    fn test_span_contains() {
        let span = Span::new(5, 10);
        assert!(span.contains(5));
        assert!(span.contains(7));
        assert!(span.contains(9));
        assert!(!span.contains(10));
        assert!(!span.contains(4));
        assert!(!span.contains(15));
    }

    #[test]
    fn test_span_overlaps() {
        let span1 = Span::new(0, 5);
        let span2 = Span::new(3, 8);
        let span3 = Span::new(10, 15);

        assert!(span1.overlaps(span2));
        assert!(span2.overlaps(span1));
        assert!(!span1.overlaps(span3));
        assert!(!span3.overlaps(span1));
    }

    #[test]
    fn test_span_extend() {
        let mut span = Span::new(5, 10);
        span.extend(Span::new(3, 8));
        assert_eq!(span.start, 3);
        assert_eq!(span.end, 10);

        span.extend(Span::new(7, 15));
        assert_eq!(span.start, 3);
        assert_eq!(span.end, 15);
    }

    #[test]
    fn test_span_after() {
        let span = Span::new(5, 10);
        let after = span.after();
        assert_eq!(after.start, 10);
        assert_eq!(after.end, 10);
        assert!(after.is_empty());
    }
}
