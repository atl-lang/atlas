//! Integration tests for AST dump command

use insta::assert_snapshot;
use std::fs;
use tempfile::TempDir;

/// Helper to create a temporary file with content and run ast command
fn run_ast_dump(source: &str) -> String {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.atl");
    fs::write(&file_path, source).unwrap();

    let output = assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .arg("ast")
        .arg(file_path.to_str().unwrap())
        .output()
        .unwrap();

    assert!(output.status.success(), "Command failed: {:?}", output);
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn test_ast_dump_simple_literal() {
    let source = "42;";
    let json = run_ast_dump(source);
    assert_snapshot!(json);
}

#[test]
fn test_ast_dump_variable_declaration() {
    let source = "let x: number = 42;";
    let json = run_ast_dump(source);
    assert_snapshot!(json);
}

#[test]
fn test_ast_dump_function_declaration() {
    let source = r#"
fn add(a: number, b: number) -> number {
    return a + b;
}
"#;
    let json = run_ast_dump(source);
    assert_snapshot!(json);
}

#[test]
fn test_ast_dump_if_statement() {
    let source = r#"
if (true) {
    let x: number = 1;
}
"#;
    let json = run_ast_dump(source);
    assert_snapshot!(json);
}

#[test]
fn test_ast_dump_while_loop() {
    let source = r#"
let i: number = 0;
while (i < 10) {
    i = i + 1;
}
"#;
    let json = run_ast_dump(source);
    assert_snapshot!(json);
}

#[test]
fn test_ast_dump_array_literal() {
    let source = "let arr: number[] = [1, 2, 3];";
    let json = run_ast_dump(source);
    assert_snapshot!(json);
}

#[test]
fn test_ast_dump_binary_expression() {
    let source = "let x: number = 1 + 2 * 3;";
    let json = run_ast_dump(source);
    assert_snapshot!(json);
}

#[test]
fn test_ast_dump_function_call() {
    let source = r#"
fn greet(name: string) -> string {
    return name;
}
let msg: string = greet("world");
"#;
    let json = run_ast_dump(source);
    assert_snapshot!(json);
}

#[test]
fn test_ast_dump_string_literal() {
    let source = r#"let msg: string = "hello";"#;
    let json = run_ast_dump(source);
    assert_snapshot!(json);
}

#[test]
fn test_ast_dump_boolean_literal() {
    let source = "let flag: bool = true;";
    let json = run_ast_dump(source);
    assert_snapshot!(json);
}
