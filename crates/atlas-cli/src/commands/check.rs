//! Check command - type-check Atlas source files without executing

use anyhow::{Context, Result};
use atlas_runtime::{Binder, Lexer, Parser, TypeChecker};
use std::fs;

/// Type-check an Atlas source file without executing it
///
/// Performs lexing, parsing, binding, and type-checking, reporting any errors.
/// If `json_output` is true, diagnostics are printed in JSON format.
pub fn run(file_path: &str, json_output: bool) -> Result<()> {
    // Read source file
    let source = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read source file: {}", file_path))?;

    // Lex the source code
    let mut lexer = Lexer::new(&source).with_file(file_path);
    let (tokens, lex_diagnostics) = lexer.tokenize();

    if !lex_diagnostics.is_empty() {
        print_diagnostics(&lex_diagnostics, &source, file_path, json_output);
        return Err(anyhow::anyhow!("Type checking failed"));
    }

    // Parse tokens into AST
    let mut parser = Parser::new(tokens);
    let (ast, parse_diagnostics) = parser.parse();

    if !parse_diagnostics.is_empty() {
        print_diagnostics(&parse_diagnostics, &source, file_path, json_output);
        return Err(anyhow::anyhow!("Type checking failed"));
    }

    // Bind symbols
    let mut binder = Binder::new();
    let (mut symbol_table, bind_diagnostics) = binder.bind(&ast);

    if !bind_diagnostics.is_empty() {
        print_diagnostics(&bind_diagnostics, &source, file_path, json_output);
        return Err(anyhow::anyhow!("Type checking failed"));
    }

    // Type check
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let typecheck_diagnostics = typechecker.check(&ast);

    // Only fail on errors, not warnings
    let has_errors = typecheck_diagnostics.iter().any(|d| d.is_error());
    if has_errors {
        print_diagnostics(&typecheck_diagnostics, &source, file_path, json_output);
        return Err(anyhow::anyhow!("Type checking failed"));
    }

    // Print warnings (they don't block success)
    let warnings: Vec<_> = typecheck_diagnostics
        .iter()
        .filter(|d| d.is_warning())
        .collect();
    if !warnings.is_empty() {
        let warning_diags: Vec<_> = warnings.into_iter().cloned().collect();
        crate::diagnostics::emit_diagnostics_stderr(&warning_diags, Some(&source), Some(file_path));
    }

    // Success!
    println!("{}: No errors found", file_path);
    Ok(())
}

/// Print diagnostics to stderr (or stdout for JSON).
/// JSON output uses the `{"errors": [...], "warnings": [...]}` wrapper format (B14-P06, D-043).
fn print_diagnostics(
    diagnostics: &[atlas_runtime::Diagnostic],
    source: &str,
    file_path: &str,
    json_output: bool,
) {
    if json_output {
        crate::diagnostics::emit_diagnostics_json(diagnostics, Some(source), Some(file_path));
    } else {
        crate::diagnostics::emit_diagnostics_stderr(diagnostics, Some(source), Some(file_path));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_check_valid_file() {
        // Create a temporary file with valid Atlas code
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "let x: number = 42;").unwrap();

        let result = run(temp_file.path().to_str().unwrap(), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_invalid_file() {
        // Create a temporary file with invalid Atlas code
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "let x: number = \"string\";").unwrap();

        let result = run(temp_file.path().to_str().unwrap(), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_missing_file() {
        let result = run("nonexistent.atl", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_json_output() {
        // Create a temporary file with invalid Atlas code
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "let x: number = \"wrong\";").unwrap();

        let result = run(temp_file.path().to_str().unwrap(), true);
        assert!(result.is_err());
    }
}
