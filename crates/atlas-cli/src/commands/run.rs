//! Run command - execute Atlas source files

use anyhow::Result;
use atlas_runtime::{Atlas, SecurityContext};

/// Run an Atlas source file
///
/// Compiles and executes the source file, printing the result to stdout.
/// If `json_output` is true, diagnostics are printed in JSON format.
pub fn run(file_path: &str, json_output: bool) -> Result<()> {
    // Create runtime with full permissions (like go run, cargo run, python, node, etc.)
    let runtime = Atlas::new_with_security(SecurityContext::allow_all());

    // Use eval_file to support module imports
    match runtime.eval_file(file_path) {
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
                    eprintln!("{}", format_diagnostic(diag, file_path));
                }
            }
            Err(anyhow::anyhow!("Failed to execute program"))
        }
    }
}

/// Format a diagnostic for display
fn format_diagnostic(diag: &atlas_runtime::Diagnostic, fallback_file: &str) -> String {
    use atlas_runtime::DiagnosticLevel;

    let level_str = match diag.level {
        DiagnosticLevel::Error => "error",
        DiagnosticLevel::Warning => "warning",
    };

    let file = if diag.file == "<unknown>" || diag.file == "<input>" {
        fallback_file
    } else {
        diag.file.as_str()
    };

    // Format: file:line:col: level: message
    let mut output = format!(
        "{}:{}:{}: {}: {}",
        file, diag.line, diag.column, level_str, diag.message
    );

    if !diag.stack_trace.is_empty() {
        for frame in &diag.stack_trace {
            let frame_file = if frame.file == "<unknown>" || frame.file == "<input>" {
                fallback_file
            } else {
                frame.file.as_str()
            };
            output.push_str(&format!(
                "\n  at {} ({}:{}:{})",
                frame.function, frame_file, frame.line, frame.column
            ));
        }
    }

    output
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
        let diag = Diagnostic::error("Test error".to_string(), Span::new(0, 3));
        let formatted = format_diagnostic(&diag, "test.atl");
        assert!(formatted.contains("error"));
        assert!(formatted.contains("Test error"));
    }
}
