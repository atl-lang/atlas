//! Diagnostic normalization for stable golden tests
//!
//! Normalizes diagnostics by stripping non-deterministic data like
//! absolute paths, timestamps, and machine-specific information.

use crate::diagnostic::{Diagnostic, RelatedLocation};
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
