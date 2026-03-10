//! Diagnostic normalization for stable golden tests
//!
//! Normalizes diagnostics by stripping non-deterministic data like
//! absolute paths, timestamps, and machine-specific information.

use crate::diagnostic::{Diagnostic, RelatedLocation};
use std::collections::HashMap;
use std::path::Path;

/// Normalize a diagnostic for golden testing
pub fn normalize_diagnostic_for_testing(diag: &Diagnostic) -> Diagnostic {
    let mut normalized = diag.clone();

    // Normalize file path to relative or placeholder
    normalized.file = normalize_path(&diag.file);

    // Normalize related locations
    normalized.related = diag
        .related
        .iter()
        .map(|rel| RelatedLocation {
            file: normalize_path(&rel.file),
            line: rel.line,
            column: rel.column,
            length: rel.length,
            message: rel.message.clone(),

            snippet: String::new(),
            label: String::new(),
            is_occurrence: false,
        })
        .collect();

    // Normalize stack trace entries
    normalized.stack_trace = diag
        .stack_trace
        .iter()
        .map(|frame| crate::diagnostic::StackTraceFrame {
            function: frame.function.clone(),
            file: normalize_path(&frame.file),
            line: frame.line,
            column: frame.column,
        })
        .collect();

    normalized
}

/// Normalize a file path for testing
///
/// Converts absolute paths to just the filename, or uses placeholders
/// for common test paths like "<input>", "<unknown>", etc.
fn normalize_path(path: &str) -> String {
    // Keep special paths as-is
    if path.starts_with('<') && path.ends_with('>') {
        return path.to_string();
    }

    // For absolute paths, try to make them relative to current dir
    if Path::new(path).is_absolute() {
        // First try to strip current directory prefix
        if let Ok(current_dir) = std::env::current_dir() {
            if let Ok(relative) = Path::new(path).strip_prefix(&current_dir) {
                return relative.display().to_string();
            }
        }

        // If that fails, just use the filename
        if let Some(filename) = Path::new(path).file_name() {
            return filename.to_string_lossy().to_string();
        }
    }

    // Return as-is if already relative or can't be normalized
    path.to_string()
}

/// Normalize a collection of diagnostics
pub fn normalize_diagnostics_for_testing(diags: &[Diagnostic]) -> Vec<Diagnostic> {
    diags.iter().map(normalize_diagnostic_for_testing).collect()
}

/// Group diagnostics with identical (code, message) into consolidated entries.
///
/// When the same error fires at multiple locations (e.g., `[]Person` used in 4 places),
/// this folds all occurrences into a single diagnostic whose `related` list carries
/// the extra locations as grouped occurrences (`is_occurrence: true`).
///
/// Single-occurrence diagnostics pass through unchanged.
/// Different messages (even same code) stay as separate diagnostics.
pub fn group_by_message(diagnostics: Vec<Diagnostic>) -> Vec<Diagnostic> {
    // Preserve order of first occurrence for each group key.
    let mut order: Vec<String> = Vec::new();
    let mut groups: HashMap<String, Vec<Diagnostic>> = HashMap::new();

    for diag in diagnostics {
        let key = format!("{}:{}", diag.code, diag.message);
        if !groups.contains_key(&key) {
            order.push(key.clone());
        }
        groups.entry(key).or_default().push(diag);
    }

    let mut result = Vec::new();
    for key in order {
        let Some(mut group) = groups.remove(&key) else {
            continue;
        };
        if group.len() == 1 {
            result.push(group.remove(0));
            continue;
        }

        // Multiple occurrences — fold into first as primary, rest become related occurrences.
        let mut primary = group.remove(0);
        for extra in group {
            primary.related.push(RelatedLocation {
                file: extra.file,
                line: extra.line,
                column: extra.column,
                length: extra.length,
                message: extra.label.clone(),
                snippet: extra.snippet,
                label: extra.label,
                is_occurrence: true,
            });
        }
        result.push(primary);
    }

    result
}
