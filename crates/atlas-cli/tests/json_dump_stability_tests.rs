//! Integration tests for JSON dump stability
//!
//! These tests ensure that AST and typecheck JSON dumps are:
//! - Deterministic (same input always produces same output)
//! - Properly formatted (no trailing spaces, stable indentation)
//! - Version-tagged (includes ast_version/typecheck_version fields)

use serde_json::Value;
use std::fs;
use tempfile::TempDir;

/// Helper to run ast dump command
fn run_ast_dump(source: &str) -> String {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.atl");
    fs::write(&file_path, source).unwrap();

    let output = assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .arg("ast")
        .arg(file_path.to_str().unwrap())
        .output()
        .unwrap();

    assert!(output.status.success(), "AST dump command failed");
    String::from_utf8(output.stdout).unwrap()
}

/// Helper to run typecheck dump command
fn run_typecheck_dump(source: &str) -> String {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.atl");
    fs::write(&file_path, source).unwrap();

    let output = assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .arg("typecheck")
        .arg(file_path.to_str().unwrap())
        .output()
        .unwrap();

    assert!(output.status.success(), "Typecheck dump command failed");
    String::from_utf8(output.stdout).unwrap()
}

/// Verify JSON has no trailing whitespace on any line
fn assert_no_trailing_whitespace(json: &str) {
    for (idx, line) in json.lines().enumerate() {
        assert!(
            !line.ends_with(' ') && !line.ends_with('\t'),
            "Line {} has trailing whitespace: {:?}",
            idx + 1,
            line
        );
    }
}

/// Verify JSON uses consistent indentation (2 spaces)
fn assert_consistent_indentation(json: &str) {
    for (idx, line) in json.lines().enumerate() {
        if line.is_empty() {
            continue;
        }
        let leading_spaces = line.len() - line.trim_start().len();
        if leading_spaces > 0 {
            assert!(
                leading_spaces % 2 == 0,
                "Line {} has inconsistent indentation (not multiple of 2): {}",
                idx + 1,
                leading_spaces
            );
        }
    }
}

/// Verify JSON can be parsed and is valid
fn assert_valid_json(json: &str) {
    serde_json::from_str::<Value>(json).expect("JSON should be valid");
}

/// Verify version field is present
fn assert_has_version_field(json: &str, field_name: &str) {
    let value: Value = serde_json::from_str(json).unwrap();
    assert!(
        value.get(field_name).is_some(),
        "JSON should have {} field",
        field_name
    );
}

// ============================================================================
// AST Dump Stability Tests
// ============================================================================

#[test]
fn test_ast_dump_deterministic_simple() {
    let source = "let x: number = 42;";
    let output1 = run_ast_dump(source);
    let output2 = run_ast_dump(source);
    assert_eq!(output1, output2, "AST dump should be deterministic");
}

#[test]
fn test_ast_dump_deterministic_complex() {
    let source = r#"
fn factorial(n: number) -> number {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}

let x: number = factorial(5);
"#;
    let output1 = run_ast_dump(source);
    let output2 = run_ast_dump(source);
    assert_eq!(output1, output2, "AST dump should be deterministic");
}

#[test]
fn test_ast_dump_no_trailing_whitespace() {
    let source = r#"
let x: number = 42;
let y: string = "hello";
"#;
    let output = run_ast_dump(source);
    assert_no_trailing_whitespace(&output);
}

#[test]
fn test_ast_dump_consistent_indentation() {
    let source = r#"
fn add(a: number, b: number) -> number {
    return a + b;
}
"#;
    let output = run_ast_dump(source);
    assert_consistent_indentation(&output);
}

#[test]
fn test_ast_dump_valid_json() {
    let source = "let x: number = 42;";
    let output = run_ast_dump(source);
    assert_valid_json(&output);
}

#[test]
fn test_ast_dump_has_version_field() {
    let source = "let x: number = 42;";
    let output = run_ast_dump(source);
    assert_has_version_field(&output, "ast_version");
}

#[test]
fn test_ast_dump_deterministic_with_arrays() {
    let source = r#"
let arr: number[] = [1, 2, 3, 4, 5];
let nested: number[][] = [[1, 2], [3, 4]];
"#;
    let output1 = run_ast_dump(source);
    let output2 = run_ast_dump(source);
    assert_eq!(output1, output2, "AST dump should be deterministic");
}

#[test]
fn test_ast_dump_deterministic_with_loops() {
    let source = r#"
let i: number = 0;
while (i < 10) {
    i = i + 1;
}
"#;
    let output1 = run_ast_dump(source);
    let output2 = run_ast_dump(source);
    assert_eq!(output1, output2, "AST dump should be deterministic");
}

// ============================================================================
// Typecheck Dump Stability Tests
// ============================================================================

#[test]
fn test_typecheck_dump_deterministic_simple() {
    let source = "let x: number = 42;";
    let output1 = run_typecheck_dump(source);
    let output2 = run_typecheck_dump(source);
    assert_eq!(output1, output2, "Typecheck dump should be deterministic");
}

#[test]
fn test_typecheck_dump_deterministic_complex() {
    let source = r#"
fn factorial(n: number) -> number {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}

let x: number = factorial(5);
"#;
    let output1 = run_typecheck_dump(source);
    let output2 = run_typecheck_dump(source);
    assert_eq!(output1, output2, "Typecheck dump should be deterministic");
}

#[test]
fn test_typecheck_dump_no_trailing_whitespace() {
    let source = r#"
let x: number = 42;
let y: string = "hello";
"#;
    let output = run_typecheck_dump(source);
    assert_no_trailing_whitespace(&output);
}

#[test]
fn test_typecheck_dump_consistent_indentation() {
    let source = r#"
fn add(a: number, b: number) -> number {
    return a + b;
}
"#;
    let output = run_typecheck_dump(source);
    assert_consistent_indentation(&output);
}

#[test]
fn test_typecheck_dump_valid_json() {
    let source = "let x: number = 42;";
    let output = run_typecheck_dump(source);
    assert_valid_json(&output);
}

#[test]
fn test_typecheck_dump_has_version_field() {
    let source = "let x: number = 42;";
    let output = run_typecheck_dump(source);
    assert_has_version_field(&output, "typecheck_version");
}

#[test]
fn test_typecheck_dump_deterministic_with_multiple_scopes() {
    let source = r#"
fn helper(b: number) -> number {
    return b * 2;
}

fn outer(a: number) -> number {
    let x: number = helper(5);
    if (a > 0) {
        let y: number = a + x;
        return y;
    }
    return x;
}
"#;
    let output1 = run_typecheck_dump(source);
    let output2 = run_typecheck_dump(source);
    assert_eq!(output1, output2, "Typecheck dump should be deterministic");
}

#[test]
fn test_typecheck_dump_deterministic_with_arrays() {
    let source = r#"
let arr: number[] = [1, 2, 3];
let first: number = arr[0];
"#;
    let output1 = run_typecheck_dump(source);
    let output2 = run_typecheck_dump(source);
    assert_eq!(output1, output2, "Typecheck dump should be deterministic");
}

// ============================================================================
// Cross-format Consistency Tests
// ============================================================================

#[test]
fn test_both_dumps_use_same_formatting() {
    let source = "let x: number = 42;";
    let ast_output = run_ast_dump(source);
    let typecheck_output = run_typecheck_dump(source);

    // Both should have no trailing whitespace
    assert_no_trailing_whitespace(&ast_output);
    assert_no_trailing_whitespace(&typecheck_output);

    // Both should have consistent indentation
    assert_consistent_indentation(&ast_output);
    assert_consistent_indentation(&typecheck_output);

    // Both should be valid JSON
    assert_valid_json(&ast_output);
    assert_valid_json(&typecheck_output);
}

#[test]
fn test_determinism_with_string_escapes() {
    let source = r#"let msg: string = "hello\nworld";"#;
    let ast1 = run_ast_dump(source);
    let ast2 = run_ast_dump(source);
    assert_eq!(ast1, ast2, "AST dump should be deterministic with escapes");

    let tc1 = run_typecheck_dump(source);
    let tc2 = run_typecheck_dump(source);
    assert_eq!(
        tc1, tc2,
        "Typecheck dump should be deterministic with escapes"
    );
}

#[test]
fn test_determinism_with_boolean_expressions() {
    let source = r#"
let a: bool = true;
let b: bool = false;
let c: bool = a && b || !a;
"#;
    let ast1 = run_ast_dump(source);
    let ast2 = run_ast_dump(source);
    assert_eq!(ast1, ast2, "AST dump should be deterministic with booleans");

    let tc1 = run_typecheck_dump(source);
    let tc2 = run_typecheck_dump(source);
    assert_eq!(
        tc1, tc2,
        "Typecheck dump should be deterministic with booleans"
    );
}
