//! Diagnostic system for errors and warnings
//!
//! All errors and warnings flow through the unified Diagnostic type,
//! ensuring consistent formatting across compiler, interpreter, and VM.

pub mod error_codes;
pub mod formatter;
pub mod normalizer;
pub mod warnings;

use crate::span::{source_for_file, Span};
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
    /// Hint — informational, non-actionable (e.g., AI scaffolding notes)
    #[serde(rename = "hint")]
    Hint,
}

impl fmt::Display for DiagnosticLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiagnosticLevel::Error => write!(f, "error"),
            DiagnosticLevel::Warning => write!(f, "warning"),
            DiagnosticLevel::Hint => write!(f, "hint"),
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

/// Stack trace frame for runtime errors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StackTraceFrame {
    /// Function name.
    pub function: String,
    /// File path.
    pub file: String,
    /// Line number (1-based).
    pub line: usize,
    /// Column number (1-based).
    pub column: usize,
}

/// A structured code-diff suggestion (Rust-style `-old / +new` lines).
///
/// Shown when Atlas knows exactly what token to replace (e.g. "did you mean `len`?").
/// Rendered in `to_human_string()` as:
/// ```text
/// help: did you mean `len`?
///   6 - s.lenght();
///   6 + s.len();
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuggestionDiff {
    /// Short description (e.g. `"did you mean \`len\`?"`)
    pub description: String,
    /// Source line number (1-based)
    pub line_number: usize,
    /// Original source line (the typo)
    pub old_line: String,
    /// Fixed source line (the correction)
    pub new_line: String,
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
    /// Runtime stack trace (optional)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub stack_trace: Vec<StackTraceFrame>,
    /// Suggested fixes (optional, multiple allowed)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub help: Vec<String>,
    /// When true, this diagnostic is a secondary/cascade error and should be
    /// visually subordinated (D-043: cascade suppression). Omitted from JSON if false.
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub is_secondary: bool,
    /// Structured code-diff suggestions (H-195). Rendered as `-old / +new` lines.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub suggestions: Vec<SuggestionDiff>,
}

impl Diagnostic {
    /// Create a new error diagnostic with code
    pub fn error_with_code(
        code: impl Into<String>,
        message: impl Into<String>,
        span: Span,
    ) -> Self {
        let (line, column, snippet) =
            source_context_for_span(span).unwrap_or((1, span.start + 1, String::new()));
        Self {
            diag_version: DIAG_VERSION,
            level: DiagnosticLevel::Error,
            code: code.into(),
            message: message.into(),
            file: span.file().to_string(),
            line,
            column,
            length: span.end.saturating_sub(span.start),
            snippet,
            label: "".to_string(),
            notes: Vec::new(),
            related: Vec::new(),
            stack_trace: Vec::new(),
            help: Vec::new(),
            is_secondary: false,
            suggestions: Vec::new(),
        }
    }

    /// Create a new warning diagnostic with code
    pub fn warning_with_code(
        code: impl Into<String>,
        message: impl Into<String>,
        span: Span,
    ) -> Self {
        let (line, column, snippet) =
            source_context_for_span(span).unwrap_or((1, span.start + 1, String::new()));
        Self {
            diag_version: DIAG_VERSION,
            level: DiagnosticLevel::Warning,
            code: code.into(),
            message: message.into(),
            file: span.file().to_string(),
            line,
            column,
            length: span.end.saturating_sub(span.start),
            snippet,
            label: String::new(),
            notes: Vec::new(),
            related: Vec::new(),
            stack_trace: Vec::new(),
            help: Vec::new(),
            is_secondary: false,
            suggestions: Vec::new(),
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

    /// Add a help message (multiple calls produce multiple help lines)
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help.push(help.into());
        self
    }

    /// Add a related location
    pub fn with_related_location(mut self, location: RelatedLocation) -> Self {
        let mut location = location;
        if (location.file == "<input>" || location.file == "<unknown>")
            && self.file != "<unknown>"
            && self.file != "<input>"
        {
            location.file = self.file.clone();
        }
        self.related.push(location);
        self
    }

    /// Add a stack trace
    pub fn with_stack_trace(mut self, stack_trace: Vec<StackTraceFrame>) -> Self {
        self.stack_trace = stack_trace;
        self
    }

    /// Add a code-diff suggestion (H-195: Rust-style `-old / +new`).
    ///
    /// Replaces the first occurrence of `old_token` with `new_token` in the diagnostic's
    /// existing snippet to build the diff lines. If the snippet doesn't contain `old_token`
    /// (e.g. multi-line expression), falls back to a plain `help:` line.
    pub fn with_suggestion_rename(
        mut self,
        description: impl Into<String>,
        old_token: &str,
        new_token: &str,
    ) -> Self {
        let description = description.into();
        if !self.snippet.is_empty() && self.snippet.contains(old_token) {
            let new_line = self.snippet.replacen(old_token, new_token, 1);
            self.suggestions.push(SuggestionDiff {
                description,
                line_number: self.line,
                old_line: self.snippet.clone(),
                new_line,
            });
        } else {
            // Fallback: plain help text when snippet doesn't contain the token
            self.help.push(format!("{}: `{}`", description, new_token));
        }
        self
    }

    /// Mark this diagnostic as a secondary/cascade error (D-043).
    /// Secondary diagnostics are visually subordinated in output and omitted from
    /// JSON when `is_secondary` is false.
    pub fn as_secondary(mut self) -> Self {
        self.is_secondary = true;
        self
    }

    /// Create a new hint diagnostic with code
    pub fn hint_with_code(code: impl Into<String>, message: impl Into<String>, span: Span) -> Self {
        let (line, column, snippet) =
            source_context_for_span(span).unwrap_or((1, span.start + 1, String::new()));
        Self {
            diag_version: DIAG_VERSION,
            level: DiagnosticLevel::Hint,
            code: code.into(),
            message: message.into(),
            file: span.file().to_string(),
            line,
            column,
            length: span.end.saturating_sub(span.start),
            snippet,
            label: String::new(),
            notes: Vec::new(),
            related: Vec::new(),
            stack_trace: Vec::new(),
            help: Vec::new(),
            is_secondary: false,
            suggestions: Vec::new(),
        }
    }

    /// Check if this diagnostic is an error (not a warning)
    pub fn is_error(&self) -> bool {
        matches!(self.level, DiagnosticLevel::Error)
    }

    /// Check if this diagnostic is a warning
    pub fn is_warning(&self) -> bool {
        matches!(self.level, DiagnosticLevel::Warning)
    }

    /// Check if this diagnostic is a hint
    pub fn is_hint(&self) -> bool {
        matches!(self.level, DiagnosticLevel::Hint)
    }

    /// Format as human-readable string
    ///
    /// Clean Atlas format — no Rust chrome (no `-->`, no `|` gutters, no `= ` prefixes).
    ///
    /// ```text
    /// error[AT0001]: Type mismatch
    /// path/to/file.atl:12:9
    /// 12: let x: str = 42;
    ///             ^ type mismatch
    /// help: expected `str`, found `int`
    /// note: declared as `str` on this line
    /// ```
    pub fn to_human_string(&self) -> String {
        let mut output = String::new();

        // Line 1: error[CODE]: message
        // Secondary diagnostics are prefixed with `note:` to subordinate them visually (D-043).
        if self.is_secondary {
            output.push_str(&format!(
                "note[{}] (secondary): {}\n",
                self.code, self.message
            ));
        } else {
            output.push_str(&format!(
                "{}[{}]: {}\n",
                self.level, self.code, self.message
            ));
        }

        // Line 2: path:line:col
        output.push_str(&format!("{}:{}:{}\n", self.file, self.line, self.column));

        // Lines 3-4: source snippet with caret
        if !self.snippet.is_empty() {
            let line_prefix = format!("{}: ", self.line);
            output.push_str(&format!("{}{}\n", line_prefix, self.snippet));

            if self.length > 0 {
                // Caret indented to match the source character position
                let caret_indent = " ".repeat(line_prefix.len() + self.column.saturating_sub(1));
                let carets = "^".repeat(self.length.max(1));
                output.push_str(&format!("{}{}", caret_indent, carets));
                if !self.label.is_empty() {
                    output.push_str(&format!(" {}", self.label));
                }
                output.push('\n');
            }
        }

        // Stack trace (before help/notes — shows execution path first)
        for frame in &self.stack_trace {
            output.push_str(&format!(
                "  at {} ({}:{}:{})\n",
                frame.function, frame.file, frame.line, frame.column
            ));
        }

        // Help lines (actionable fixes — what to write/change)
        for help in &self.help {
            output.push_str(&format!("help: {}\n", help));
        }

        // Suggestion diffs — Rust-style `-old / +new` code blocks (H-195)
        for sug in &self.suggestions {
            output.push_str(&format!("help: {}\n", sug.description));
            output.push_str(&format!("  {} - {}\n", sug.line_number, sug.old_line));
            output.push_str(&format!("  {} + {}\n", sug.line_number, sug.new_line));
        }

        // Note lines (context/explanation + related locations)
        for note in &self.notes {
            output.push_str(&format!("note: {}\n", note));
        }
        for related in &self.related {
            output.push_str(&format!(
                "note: see {}:{}:{}: {}\n",
                related.file, related.line, related.column, related.message
            ));
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

fn source_context_for_span(span: Span) -> Option<(usize, usize, String)> {
    let source = source_for_file(span.file)?;
    let (line, column) =
        crate::diagnostic::formatter::offset_to_line_col(source.as_ref(), span.start);
    let snippet =
        crate::diagnostic::formatter::extract_snippet(source.as_ref(), line).unwrap_or_default();
    Some((line, column, snippet))
}

/// Sort diagnostics by level (errors first), then by location
pub fn sort_diagnostics(diagnostics: &mut [Diagnostic]) {
    diagnostics.sort_by(|a, b| {
        // Errors before warnings before hints
        match (a.level, b.level) {
            (DiagnosticLevel::Error, DiagnosticLevel::Warning | DiagnosticLevel::Hint) => {
                std::cmp::Ordering::Less
            }
            (DiagnosticLevel::Warning | DiagnosticLevel::Hint, DiagnosticLevel::Error) => {
                std::cmp::Ordering::Greater
            }
            (DiagnosticLevel::Warning, DiagnosticLevel::Hint) => std::cmp::Ordering::Less,
            (DiagnosticLevel::Hint, DiagnosticLevel::Warning) => std::cmp::Ordering::Greater,
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
