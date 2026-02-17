//! Source map - bidirectional mapping between instruction offsets and source locations.
//!
//! Converts `DebugSpan` entries from bytecode into (file, line, column) source locations
//! and maintains both forward (offset → location) and reverse (location → offset) indexes.

use std::collections::HashMap;

use crate::bytecode::DebugSpan;
use crate::debugger::protocol::SourceLocation;

// ── Line offset computation ───────────────────────────────────────────────────

/// Compute the byte offset of each line's start in `source`.
///
/// Returns a `Vec` where `result[i]` is the byte offset of line `i+1` (0-indexed).
/// Line 1 always starts at offset 0.
pub fn compute_line_offsets(source: &str) -> Vec<usize> {
    let mut offsets = vec![0usize]; // line 1 starts at 0
    for (i, ch) in source.char_indices() {
        if ch == '\n' {
            offsets.push(i + 1);
        }
    }
    offsets
}

/// Convert a byte offset to a 1-based `(line, column)` pair.
///
/// Uses the pre-computed `line_offsets` table from [`compute_line_offsets`].
pub fn byte_offset_to_line_column(offset: usize, line_offsets: &[usize]) -> (u32, u32) {
    // Binary search for the greatest line start ≤ offset.
    let line_index = match line_offsets.binary_search(&offset) {
        Ok(i) => i,      // offset is exactly at a line start
        Err(i) => i - 1, // offset is within line i-1 (0-based)
    };
    let line_start = line_offsets[line_index];
    let column = offset.saturating_sub(line_start);
    ((line_index + 1) as u32, (column + 1) as u32)
}

// ── SourceMap ─────────────────────────────────────────────────────────────────

/// Bidirectional mapping between instruction offsets and source locations.
///
/// Built from bytecode [`DebugSpan`] entries.  Optionally accepts source text
/// to compute 1-based line/column numbers from byte offsets; without source text
/// all entries get line=1, column=1.
#[derive(Debug, Default)]
pub struct SourceMap {
    /// offset → source location
    offset_to_location: HashMap<usize, SourceLocation>,
    /// (file, line) → sorted list of instruction offsets
    line_to_offsets: HashMap<(String, u32), Vec<usize>>,
    /// (file, line, column) → first matching instruction offset
    location_to_offset: HashMap<(String, u32, u32), usize>,
}

impl SourceMap {
    /// Create an empty source map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a source map from bytecode `DebugSpan` entries.
    ///
    /// * `file` – source file name; use an empty string for anonymous source.
    /// * `source` – optional source text used to convert byte offsets to line/column.
    ///   When `None`, all entries get line 1, column 1.
    pub fn from_debug_spans(spans: &[DebugSpan], file: &str, source: Option<&str>) -> Self {
        let mut map = Self::new();

        let line_offsets: Vec<usize> = match source {
            Some(src) => compute_line_offsets(src),
            None => vec![0],
        };

        for debug_span in spans {
            let (line, column) = if source.is_some() {
                byte_offset_to_line_column(debug_span.span.start, &line_offsets)
            } else {
                (1u32, 1u32)
            };

            let location = SourceLocation {
                file: file.to_string(),
                line,
                column,
            };

            map.insert(debug_span.instruction_offset, location);
        }

        map
    }

    /// Insert a single offset → location mapping.
    pub fn insert(&mut self, offset: usize, location: SourceLocation) {
        // line → offsets  (allow multiple instructions on the same line)
        let line_key = (location.file.clone(), location.line);
        let entry = self.line_to_offsets.entry(line_key).or_default();
        if !entry.contains(&offset) {
            entry.push(offset);
            entry.sort_unstable();
        }

        // (file, line, col) → first offset (first insertion wins)
        let loc_key = (location.file.clone(), location.line, location.column);
        self.location_to_offset.entry(loc_key).or_insert(offset);

        // offset → location (last write wins – use for updates)
        self.offset_to_location.insert(offset, location);
    }

    /// Get the source location for an instruction offset.
    ///
    /// If there is no exact match, returns the location of the closest *preceding*
    /// instruction so callers always get a meaningful result.
    pub fn location_for_offset(&self, offset: usize) -> Option<&SourceLocation> {
        if let Some(loc) = self.offset_to_location.get(&offset) {
            return Some(loc);
        }

        // Closest preceding key
        let best = self
            .offset_to_location
            .keys()
            .copied()
            .filter(|&o| o <= offset)
            .max()?;

        self.offset_to_location.get(&best)
    }

    /// Get the first instruction offset for an exact `(file, line, column)` match.
    pub fn offset_for_location(&self, file: &str, line: u32, column: u32) -> Option<usize> {
        let key = (file.to_string(), line, column);
        self.location_to_offset.get(&key).copied()
    }

    /// Get all instruction offsets for a source line, sorted ascending.
    pub fn offsets_for_line(&self, file: &str, line: u32) -> Vec<usize> {
        let key = (file.to_string(), line);
        self.line_to_offsets.get(&key).cloned().unwrap_or_default()
    }

    /// Get the first (lowest) instruction offset for a source line.
    pub fn first_offset_for_line(&self, file: &str, line: u32) -> Option<usize> {
        self.offsets_for_line(file, line).into_iter().next()
    }

    /// Returns `true` if the map contains no entries.
    pub fn is_empty(&self) -> bool {
        self.offset_to_location.is_empty()
    }

    /// Total number of mapped instruction offsets.
    pub fn len(&self) -> usize {
        self.offset_to_location.len()
    }

    /// All mapped instruction offsets, sorted ascending.
    pub fn all_offsets(&self) -> Vec<usize> {
        let mut offsets: Vec<usize> = self.offset_to_location.keys().copied().collect();
        offsets.sort_unstable();
        offsets
    }

    /// All distinct source files referenced by this map.
    pub fn files(&self) -> Vec<String> {
        let mut files: Vec<String> = self
            .offset_to_location
            .values()
            .map(|l| l.file.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        files.sort();
        files
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::Span;

    fn make_spans(pairs: &[(usize, usize, usize)]) -> Vec<DebugSpan> {
        // pairs: (instruction_offset, span_start, span_end)
        pairs
            .iter()
            .map(|&(off, start, end)| DebugSpan {
                instruction_offset: off,
                span: Span::new(start, end),
            })
            .collect()
    }

    // ── compute_line_offsets ─────────────────────────────────────────────────

    #[test]
    fn test_line_offsets_empty() {
        let offsets = compute_line_offsets("");
        assert_eq!(offsets, vec![0]);
    }

    #[test]
    fn test_line_offsets_single_line() {
        let offsets = compute_line_offsets("hello");
        assert_eq!(offsets, vec![0]);
    }

    #[test]
    fn test_line_offsets_two_lines() {
        let offsets = compute_line_offsets("hi\nbye");
        assert_eq!(offsets, vec![0, 3]);
    }

    #[test]
    fn test_line_offsets_trailing_newline() {
        let offsets = compute_line_offsets("line1\n");
        assert_eq!(offsets, vec![0, 6]);
    }

    // ── byte_offset_to_line_column ───────────────────────────────────────────

    #[test]
    fn test_byte_offset_start_of_line1() {
        let src = "abc\ndef";
        let offsets = compute_line_offsets(src);
        assert_eq!(byte_offset_to_line_column(0, &offsets), (1, 1));
    }

    #[test]
    fn test_byte_offset_middle_of_line1() {
        let src = "abc\ndef";
        let offsets = compute_line_offsets(src);
        assert_eq!(byte_offset_to_line_column(2, &offsets), (1, 3));
    }

    #[test]
    fn test_byte_offset_start_of_line2() {
        let src = "abc\ndef";
        let offsets = compute_line_offsets(src);
        // 'def' starts at offset 4
        assert_eq!(byte_offset_to_line_column(4, &offsets), (2, 1));
    }

    #[test]
    fn test_byte_offset_middle_of_line2() {
        let src = "abc\ndef";
        let offsets = compute_line_offsets(src);
        assert_eq!(byte_offset_to_line_column(5, &offsets), (2, 2));
    }

    // ── SourceMap::from_debug_spans ──────────────────────────────────────────

    #[test]
    fn test_from_debug_spans_no_source() {
        let spans = make_spans(&[(0, 0, 3), (4, 3, 6)]);
        let map = SourceMap::from_debug_spans(&spans, "test.atlas", None);
        assert_eq!(map.len(), 2);
        // Without source, all locations get line=1, col=1
        let loc = map.location_for_offset(0).unwrap();
        assert_eq!(loc.line, 1);
        assert_eq!(loc.column, 1);
    }

    #[test]
    fn test_from_debug_spans_with_source() {
        // "let x = 1;\nlet y = 2;\n"
        let src = "let x = 1;\nlet y = 2;\n";
        let spans = make_spans(&[
            (0, 0, 10),  // "let x = 1;" → line 1
            (3, 11, 21), // "let y = 2;" → line 2
        ]);
        let map = SourceMap::from_debug_spans(&spans, "main.atlas", Some(src));
        assert_eq!(map.len(), 2);

        let loc0 = map.location_for_offset(0).unwrap();
        assert_eq!(loc0.line, 1);
        assert_eq!(loc0.column, 1);

        let loc1 = map.location_for_offset(3).unwrap();
        assert_eq!(loc1.line, 2);
        assert_eq!(loc1.column, 1);
    }

    // ── SourceMap lookups ────────────────────────────────────────────────────

    #[test]
    fn test_location_for_offset_exact() {
        let mut map = SourceMap::new();
        map.insert(10, SourceLocation::new("a.atlas", 3, 1));
        assert_eq!(map.location_for_offset(10).unwrap().line, 3);
    }

    #[test]
    fn test_location_for_offset_closest_preceding() {
        let mut map = SourceMap::new();
        map.insert(0, SourceLocation::new("a.atlas", 1, 1));
        map.insert(10, SourceLocation::new("a.atlas", 5, 1));
        // Offset 7 is between 0 and 10; should return line 1 (closest preceding)
        assert_eq!(map.location_for_offset(7).unwrap().line, 1);
    }

    #[test]
    fn test_location_for_offset_none_before() {
        let mut map = SourceMap::new();
        map.insert(10, SourceLocation::new("a.atlas", 5, 1));
        // Offset 5 is before the only entry
        assert!(map.location_for_offset(5).is_none());
    }

    #[test]
    fn test_offset_for_location_exact() {
        let mut map = SourceMap::new();
        map.insert(42, SourceLocation::new("a.atlas", 7, 3));
        assert_eq!(map.offset_for_location("a.atlas", 7, 3), Some(42));
    }

    #[test]
    fn test_offset_for_location_missing() {
        let map = SourceMap::new();
        assert_eq!(map.offset_for_location("a.atlas", 7, 3), None);
    }

    #[test]
    fn test_offsets_for_line_multiple() {
        let mut map = SourceMap::new();
        map.insert(0, SourceLocation::new("a.atlas", 1, 1));
        map.insert(2, SourceLocation::new("a.atlas", 1, 5));
        map.insert(5, SourceLocation::new("a.atlas", 2, 1));
        let line1 = map.offsets_for_line("a.atlas", 1);
        assert_eq!(line1, vec![0, 2]);
    }

    #[test]
    fn test_first_offset_for_line() {
        let mut map = SourceMap::new();
        map.insert(3, SourceLocation::new("a.atlas", 2, 1));
        map.insert(1, SourceLocation::new("a.atlas", 2, 5));
        assert_eq!(map.first_offset_for_line("a.atlas", 2), Some(1));
    }

    #[test]
    fn test_all_offsets_sorted() {
        let mut map = SourceMap::new();
        map.insert(10, SourceLocation::new("a.atlas", 1, 1));
        map.insert(3, SourceLocation::new("a.atlas", 1, 5));
        map.insert(7, SourceLocation::new("a.atlas", 2, 1));
        assert_eq!(map.all_offsets(), vec![3, 7, 10]);
    }

    #[test]
    fn test_files() {
        let mut map = SourceMap::new();
        map.insert(0, SourceLocation::new("a.atlas", 1, 1));
        map.insert(5, SourceLocation::new("b.atlas", 1, 1));
        let mut files = map.files();
        files.sort();
        assert_eq!(files, vec!["a.atlas", "b.atlas"]);
    }

    #[test]
    fn test_empty_map() {
        let map = SourceMap::new();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
        assert!(map.all_offsets().is_empty());
    }
}
