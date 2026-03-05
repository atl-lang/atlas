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
                let source = std::fs::read_to_string(file_path).ok();
                crate::diagnostics::emit_diagnostics_stderr(
                    &diagnostics,
                    source.as_deref(),
                    Some(file_path),
                );
            }
            Err(anyhow::anyhow!("Failed to execute program"))
        }
    }
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
    fn test_format_diagnostic_plain() {
        let diag = Diagnostic::error("Test error".to_string(), Span::new(0, 3));
        let formatted = crate::diagnostics::format_diagnostic_plain(&diag, None, Some("test.atl"));
        assert!(formatted.contains("error"));
        assert!(formatted.contains("Test error"));
    }
}
