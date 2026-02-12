# Diagnostic System

Builder pattern for structured error reporting.

## Diagnostic Structure

```rust
// diagnostic.rs
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub diag_version: u32,
    pub level: DiagnosticLevel,
    pub code: String,
    pub message: String,
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub length: usize,
    pub snippet: String,
    pub label: Option<String>,
    pub notes: Vec<String>,
    pub related: Vec<RelatedSpan>,
    pub help: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiagnosticLevel {
    Error,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedSpan {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub length: usize,
    pub message: String,
}
```

## Builder Pattern

```rust
impl Diagnostic {
    pub fn error(code: &str, message: &str, span: Span) -> Self {
        Self {
            diag_version: 1,
            level: DiagnosticLevel::Error,
            code: code.to_string(),
            message: message.to_string(),
            file: "<input>".to_string(),
            line: span.line,
            column: span.column,
            length: span.len(),
            snippet: String::new(),
            label: None,
            notes: Vec::new(),
            related: Vec::new(),
            help: None,
        }
    }

    pub fn warning(code: &str, message: &str, span: Span) -> Self {
        let mut diag = Self::error(code, message, span);
        diag.level = DiagnosticLevel::Warning;
        diag
    }

    pub fn with_label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    pub fn with_note(mut self, note: &str) -> Self {
        self.notes.push(note.to_string());
        self
    }

    pub fn with_help(mut self, help: &str) -> Self {
        self.help = Some(help.to_string());
        self
    }

    pub fn with_related(mut self, span: Span, message: &str) -> Self {
        self.related.push(RelatedSpan {
            file: "<input>".to_string(),
            line: span.line,
            column: span.column,
            length: span.len(),
            message: message.to_string(),
        });
        self
    }
}
```

## Formatting

```rust
impl Diagnostic {
    pub fn to_human(&self, source: &str) -> String {
        let level_str = match self.level {
            DiagnosticLevel::Error => "error",
            DiagnosticLevel::Warning => "warning",
        };

        let mut output = format!(
            "{}[{}]: {}\n  --> {}:{}:{}\n",
            level_str, self.code, self.message,
            self.file, self.line, self.column
        );

        // Extract snippet
        let lines: Vec<&str> = source.lines().collect();
        if self.line > 0 && (self.line as usize) <= lines.len() {
            let line_text = lines[self.line as usize - 1];
            output.push_str(&format!("{:>4} | {}\n", self.line, line_text));

            // Caret line
            let spaces = " ".repeat(self.column as usize - 1);
            let carets = "^".repeat(self.length.max(1));
            if let Some(label) = &self.label {
                output.push_str(&format!("     | {}{} {}\n", spaces, carets, label));
            } else {
                output.push_str(&format!("     | {}{}\n", spaces, carets));
            }
        }

        for note in &self.notes {
            output.push_str(&format!("note: {}\n", note));
        }

        if let Some(help) = &self.help {
            output.push_str(&format!("help: {}\n", help));
        }

        output
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }
}
```

## Usage Example

```rust
// In typechecker:
self.diagnostics.push(
    Diagnostic::error(
        "AT0001",
        "Type mismatch",
        span
    )
    .with_label(&format!("expected {}, found {}", expected, found))
    .with_note(&format!("variable declared as {}", expected))
    .with_help("convert the value or change the variable type")
);
```
