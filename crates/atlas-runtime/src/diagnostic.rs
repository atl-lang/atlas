//! Diagnostic system for errors and warnings
//!
//! All errors and warnings flow through the unified Diagnostic type,
//! ensuring consistent formatting across compiler, interpreter, and VM.

pub mod error_codes;
pub mod formatter;
pub mod normalizer;
pub mod warnings;

use crate::span::Span;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Diagnostic schema version
pub const DIAG_VERSION: u32 = 1;

/// Severity level of a diagnostic
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticLevel {
    /// Fatal error that prevents compilation
    #[serde(rename = "error")]
    Error,
    /// Warning that doesn't prevent compilation
    #[serde(rename = "warning")]
    Warning,
}

impl fmt::Display for DiagnosticLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiagnosticLevel::Error => write!(f, "error"),
            DiagnosticLevel::Warning => write!(f, "warning"),
        }
    }
}

/// Secondary location for related diagnostic information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelatedLocation {
    /// File path
    pub file: String,
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
    /// Length of the span
    pub length: usize,
    /// Description of this location
    pub message: String,
}

/// A diagnostic message (error or warning)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Diagnostic schema version
    pub diag_version: u32,
    /// Severity level
    pub level: DiagnosticLevel,
    /// Error code (e.g., "AT0001")
    pub code: String,
    /// Main diagnostic message
    pub message: String,
    /// File path
    pub file: String,
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
    /// Length of error span
    pub length: usize,
    /// Source line string
    pub snippet: String,
    /// Short label for caret range
    pub label: String,
    /// Additional notes (optional)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub notes: Vec<String>,
    /// Related locations (optional)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub related: Vec<RelatedLocation>,
    /// Suggested fix (optional)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub help: Option<String>,
}

impl Diagnostic {
    /// Create a new error diagnostic with code
    pub fn error_with_code(
        code: impl Into<String>,
        message: impl Into<String>,
        span: Span,
    ) -> Self {
        Self {
            diag_version: DIAG_VERSION,
            level: DiagnosticLevel::Error,
            code: code.into(),
            message: message.into(),
            file: "<unknown>".to_string(),
            line: 1,
            column: span.start + 1,
            length: span.end.saturating_sub(span.start),
            snippet: "".to_string(),
            label: "".to_string(),
            notes: Vec::new(),
            related: Vec::new(),
            help: None,
        }
    }

    /// Create a new warning diagnostic with code
    pub fn warning_with_code(
        code: impl Into<String>,
        message: impl Into<String>,
        span: Span,
    ) -> Self {
        Self {
            diag_version: DIAG_VERSION,
            level: DiagnosticLevel::Warning,
            code: code.into(),
            message: message.into(),
            file: "<unknown>".to_string(),
            line: 1,
            column: span.start + 1,
            length: span.end.saturating_sub(span.start),
            snippet: String::new(),
            label: String::new(),
            notes: Vec::new(),
            related: Vec::new(),
            help: None,
        }
    }

    /// Create a new error diagnostic (uses generic error code)
    pub fn error(message: impl Into<String>, span: Span) -> Self {
        Self::error_with_code("AT9999", message, span)
    }

    /// Create a new warning diagnostic (uses generic warning code)
    pub fn warning(message: impl Into<String>, span: Span) -> Self {
        Self::warning_with_code("AW9999", message, span)
    }

    /// Set the file path
    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.file = file.into();
        self
    }

    /// Set the line number
    pub fn with_line(mut self, line: usize) -> Self {
        self.line = line;
        self
    }

    /// Set the snippet (source line)
    pub fn with_snippet(mut self, snippet: impl Into<String>) -> Self {
        self.snippet = snippet.into();
        self
    }

    /// Set the label (caret description)
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Add a note
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Add a help message
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Add a related location
    pub fn with_related_location(mut self, location: RelatedLocation) -> Self {
        self.related.push(location);
        self
    }

    /// Format as human-readable string
    pub fn to_human_string(&self) -> String {
        let mut output = String::new();

        // Header: error[AT0001]: Type mismatch
        output.push_str(&format!(
            "{}[{}]: {}\n",
            self.level, self.code, self.message
        ));

        // Location: --> path/to/file.atl:12:9
        output.push_str(&format!(
            "  --> {}:{}:{}\n",
            self.file, self.line, self.column
        ));

        // Snippet with caret
        if !self.snippet.is_empty() {
            output.push_str("   |\n");
            output.push_str(&format!("{:>2} | {}\n", self.line, self.snippet));

            // Caret line
            if self.length > 0 {
                let padding = " ".repeat(self.column - 1);
                let carets = "^".repeat(self.length);
                output.push_str(&format!("   | {}{}", padding, carets));

                if !self.label.is_empty() {
                    output.push_str(&format!(" {}", self.label));
                }
                output.push('\n');
            }
        }

        // Notes
        for note in &self.notes {
            output.push_str(&format!("   = note: {}\n", note));
        }

        // Related locations
        for related in &self.related {
            output.push_str(&format!(
                "   = note: related location at {}:{}:{}: {}\n",
                related.file, related.line, related.column, related.message
            ));
        }

        // Help
        if let Some(help) = &self.help {
            output.push_str(&format!("   = help: {}\n", help));
        }

        output
    }

    /// Format as JSON string
    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Format as compact JSON string
    pub fn to_json_compact(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Sort diagnostics by level (errors first), then by location
pub fn sort_diagnostics(diagnostics: &mut [Diagnostic]) {
    diagnostics.sort_by(|a, b| {
        // Errors before warnings
        match (a.level, b.level) {
            (DiagnosticLevel::Error, DiagnosticLevel::Warning) => std::cmp::Ordering::Less,
            (DiagnosticLevel::Warning, DiagnosticLevel::Error) => std::cmp::Ordering::Greater,
            _ => {
                // Same level: sort by file, line, column
                a.file
                    .cmp(&b.file)
                    .then(a.line.cmp(&b.line))
                    .then(a.column.cmp(&b.column))
            }
        }
    });
}
