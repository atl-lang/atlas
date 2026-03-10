//! DiagnosticDescriptor — static per-code metadata and builder API
//!
//! Each AT/AW error code gets one `DiagnosticDescriptor` constant that carries:
//! - the code string
//! - severity level
//! - a human title (used by `atlas explain`)
//! - a message template with named `{key}` holes
//! - optional static help/note text
//! - the diagnostic domain (Parser, Typechecker, etc.)
//!
//! Call sites use `DESCRIPTOR.emit(span)` to obtain a `DiagnosticBuilder`, fill
//! template arguments with `.arg("key", value)`, append contextual help/notes, and
//! call `.build()` to get the final `Diagnostic`.

use std::collections::HashMap;

use crate::{
    diagnostic::{Diagnostic, DiagnosticLevel, SuggestionDiff},
    span::Span,
};

// ── Domain ────────────────────────────────────────────────────────────────────

/// Which compiler sub-system owns this diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticDomain {
    /// Lexer / parser errors (AT1xxx)
    Parser,
    /// Type-checker / binder errors (AT3xxx, AT4xxx)
    Typechecker,
    /// Interpreter / VM runtime errors (AT0xxx)
    Runtime,
    /// Standard-library errors (AT01xx)
    Stdlib,
    /// LSP-specific diagnostics
    Lsp,
}

// ── Descriptor ────────────────────────────────────────────────────────────────

/// Static descriptor attached to every diagnostic code.
///
/// Descriptors are `const` — they live in the data segment and are referenced
/// by pointer. No heap allocation is needed until `emit()` is called.
///
/// `Copy` is intentional: all fields are `&'static str` or plain enums — zero
/// heap allocation, trivially copyable.
#[derive(Copy, Clone)]
pub struct DiagnosticDescriptor {
    /// Error code string, e.g. `"AT1000"`.
    pub code: &'static str,
    /// Severity level (always `Error` for AT codes, `Warning` for AW codes).
    pub level: DiagnosticLevel,
    /// Short human title — shown by `atlas explain <code>`.
    pub title: &'static str,
    /// Message template. Use `{key}` named holes, e.g.
    /// `"expected {expected}, found {found}"`.
    pub message_template: &'static str,
    /// Optional static help line prepended before any call-site help.
    pub static_help: Option<&'static str>,
    /// Optional static note line prepended before any call-site notes.
    pub static_note: Option<&'static str>,
    /// Sub-system that owns this code.
    pub domain: DiagnosticDomain,
}

impl DiagnosticDescriptor {
    /// Begin building a diagnostic at `span`.
    ///
    /// Returns a `DiagnosticBuilder` pre-loaded with this descriptor.
    /// Chain `.arg()`, `.with_help()`, `.with_note()`, and `.with_suggestion_rename()`
    /// calls before finishing with `.build()`.
    pub fn emit(&'static self, span: Span) -> DiagnosticBuilder {
        DiagnosticBuilder {
            desc: self,
            span,
            args: HashMap::new(),
            extra_help: Vec::new(),
            extra_notes: Vec::new(),
            suggestions: Vec::new(),
        }
    }
}

// ── Builder ───────────────────────────────────────────────────────────────────

/// Accumulates per-call-site arguments and extras before producing a `Diagnostic`.
pub struct DiagnosticBuilder {
    /// Reference to the static descriptor for this code.
    pub desc: &'static DiagnosticDescriptor,
    /// Span where the diagnostic is anchored.
    pub span: Span,
    /// Named template arguments, keyed by placeholder name (without braces).
    pub args: HashMap<String, String>,
    /// Extra help lines appended *after* `desc.static_help`.
    pub extra_help: Vec<String>,
    /// Extra note lines appended *after* `desc.static_note`.
    pub extra_notes: Vec<String>,
    /// Structured code-diff suggestions (Rust-style `-old / +new`).
    pub suggestions: Vec<SuggestionDiff>,
}

impl DiagnosticBuilder {
    /// Set a named template argument.
    ///
    /// `key` must match the placeholder in `message_template` (e.g. `"expected"`
    /// for the hole `{expected}`). Keys not present in the template are silently
    /// ignored. Unknown keys in the template are left as-is so callers can detect
    /// missing args during testing.
    pub fn arg(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.args.insert(key.into(), value.into());
        self
    }

    /// Append an extra help line (shown after `static_help` if present).
    pub fn with_help(mut self, text: impl Into<String>) -> Self {
        self.extra_help.push(text.into());
        self
    }

    /// Append an extra note line (shown after `static_note` if present).
    pub fn with_note(mut self, text: impl Into<String>) -> Self {
        self.extra_notes.push(text.into());
        self
    }

    /// Attach a structured code-diff suggestion.
    ///
    /// Delegates to `Diagnostic::with_suggestion_rename` semantics — replaces the
    /// first occurrence of `old_token` in the snippet with `new_token`.  Falls back
    /// to a plain `help:` line when the snippet doesn't contain `old_token`.
    pub fn with_suggestion_rename(
        mut self,
        description: impl Into<String>,
        old_token: impl Into<String>,
        new_token: impl Into<String>,
    ) -> Self {
        self.suggestions.push(SuggestionDiff {
            description: description.into(),
            // Sentinel values: build() will substitute via Diagnostic::with_suggestion_rename
            line_number: 0,
            old_line: old_token.into(),
            new_line: new_token.into(),
            note: None,
        });
        self
    }

    /// Substitute `{key}` holes in `template` using `args`.
    fn substitute(template: &str, args: &HashMap<String, String>) -> String {
        let mut result = template.to_string();
        for (key, value) in args {
            result = result.replace(&format!("{{{}}}", key), value);
        }
        result
    }

    /// Finalise the builder into a `Diagnostic`.
    ///
    /// Substitution order:
    /// 1. Template holes in `message_template` are replaced with `args`.
    /// 2. `static_help` (if any) is prepended to help lines.
    /// 3. `extra_help` lines are appended after.
    /// 4. `static_note` (if any) is prepended to note lines.
    /// 5. `extra_notes` lines are appended after.
    /// 6. Suggestions are applied via `Diagnostic::with_suggestion_rename`.
    pub fn build(self) -> Diagnostic {
        let message = Self::substitute(self.desc.message_template, &self.args);

        let mut diag = match self.desc.level {
            DiagnosticLevel::Error => {
                Diagnostic::error_with_code(self.desc.code, message, self.span)
            }
            DiagnosticLevel::Warning => {
                Diagnostic::warning_with_code(self.desc.code, message, self.span)
            }
            DiagnosticLevel::Hint => Diagnostic::hint_with_code(self.desc.code, message, self.span),
        };

        // Prepend static_help first, then extra_help
        if let Some(sh) = self.desc.static_help {
            diag = diag.with_help(sh);
        }
        for h in self.extra_help {
            diag = diag.with_help(h);
        }

        // Prepend static_note first, then extra_notes
        if let Some(sn) = self.desc.static_note {
            diag = diag.with_note(sn);
        }
        for n in self.extra_notes {
            diag = diag.with_note(n);
        }

        // Apply suggestions
        for sug in self.suggestions {
            diag = diag.with_suggestion_rename(sug.description, &sug.old_line, &sug.new_line);
        }

        diag
    }
}
