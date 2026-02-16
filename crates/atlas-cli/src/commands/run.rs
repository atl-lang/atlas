//! Run command - execute Atlas source files

use anyhow::{Context, Result};
use atlas_runtime::{Atlas, SecurityContext};
use std::fs;

/// Run an Atlas source file
///
/// Compiles and executes the source file, printing the result to stdout.
/// If `json_output` is true, diagnostics are printed in JSON format.
pub fn run(file_path: &str, json_output: bool) -> Result<()> {
    // Read source file
    let source = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read source file: {}", file_path))?;

    // Create runtime with full permissions (like go run, cargo run, python, node, etc.)
    let runtime = Atlas::new_with_security(SecurityContext::allow_all());
    match runtime.eval(&source) {
        Ok(value) => {
            // Print the result value if it's not null
            if !matches!(value, atlas_runtime::Value::Null) {
                println!("{}", value);
            }
            Ok(())
        }
        Err(diagnostics) => {
            // Print all diagnostics
            if json_output {
                // JSON format
                for diag in &diagnostics {
                    println!("{}", diag.to_json_string().unwrap());
                }
            } else {
                // Human-readable format
                eprintln!("Errors occurred while running {}:", file_path);
                for diag in &diagnostics {
                    eprintln!("{}", format_diagnostic(diag, &source));
                }
            }
            Err(anyhow::anyhow!("Failed to execute program"))
        }
    }
}

/// Format a diagnostic for display
fn format_diagnostic(diag: &atlas_runtime::Diagnostic, _source: &str) -> String {
    use atlas_runtime::DiagnosticLevel;

    let level_str = match diag.level {
        DiagnosticLevel::Error => "error",
        DiagnosticLevel::Warning => "warning",
    };

    // Format: line:col: level: message
    format!(
        "{}:{}: {}: {}",
        diag.line, diag.column, level_str, diag.message
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_runtime::{Diagnostic, Span};
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_run_simple_expression() {
        // Create a temporary file with Atlas code
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1 + 2;").unwrap();

        let result = run(temp_file.path().to_str().unwrap(), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_missing_file() {
        let result = run("nonexistent.atl", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_json_output() {
        // Create a temporary file with invalid Atlas code
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "let x: number = \"wrong\";").unwrap();

        let result = run(temp_file.path().to_str().unwrap(), true);
        assert!(result.is_err());
    }

    #[test]
    fn test_format_diagnostic() {
        let source = "let x = 42;";
        let diag = Diagnostic::error("Test error".to_string(), Span::new(0, 3));
        let formatted = format_diagnostic(&diag, source);
        assert!(formatted.contains("error"));
        assert!(formatted.contains("Test error"));
    }
}
