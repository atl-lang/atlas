use super::create_module;
use atlas_runtime::{ModuleExecutor, SecurityContext, Value};
use tempfile::TempDir;

// Module Execution Tests (BLOCKER 04-D)
// Tests for runtime module execution in interpreter.

#[test]
fn test_single_module_no_imports() {
    let temp_dir = TempDir::new().unwrap();
    let main = create_module(temp_dir.path(), "main", "let x: number = 42;\nx;");

    let mut interp = atlas_runtime::Interpreter::new();
    let sec = SecurityContext::allow_all();
    let mut executor = ModuleExecutor::new(&mut interp, &sec, temp_dir.path().to_path_buf());
    let result = executor.execute_module(&main);

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 42.0),
        Ok(v) => panic!("Expected Number(42.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}
#[test]
fn test_single_module_with_function() {
    let temp_dir = TempDir::new().unwrap();
    let main = create_module(
        temp_dir.path(),
        "main",
        "fn add(borrow a: number, borrow b: number) -> number { return a + b; }\nadd(10, 20);",
    );

    let mut interp = atlas_runtime::Interpreter::new();
    let sec = SecurityContext::allow_all();
    let mut executor = ModuleExecutor::new(&mut interp, &sec, temp_dir.path().to_path_buf());
    let result = executor.execute_module(&main);

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 30.0),
        Ok(v) => panic!("Expected Number(30.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}
#[test]
fn test_module_with_export_function() {
    let temp_dir = TempDir::new().unwrap();
    let math = create_module(
        temp_dir.path(),
        "math",
        "export fn multiply(borrow a: number, borrow b: number) -> number { return a * b; }",
    );

    let mut interp = atlas_runtime::Interpreter::new();
    let sec = SecurityContext::allow_all();
    let mut executor = ModuleExecutor::new(&mut interp, &sec, temp_dir.path().to_path_buf());
    let result = executor.execute_module(&math);

    assert!(result.is_ok());
}

#[test]
fn test_module_with_export_variable() {
    let temp_dir = TempDir::new().unwrap();
    let constants = create_module(
        temp_dir.path(),
        "constants",
        "export let PI: number = 3.14159;",
    );

    let mut interp = atlas_runtime::Interpreter::new();
    let sec = SecurityContext::allow_all();
    let mut executor = ModuleExecutor::new(&mut interp, &sec, temp_dir.path().to_path_buf());
    let result = executor.execute_module(&constants);

    assert!(result.is_ok());
}
#[test]
fn test_import_single_function() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "math",
        "export fn add(borrow a: number, borrow b: number) -> number { return a + b; }",
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { add } from "./math";
add(5, 7);
"#,
    );

    let mut interp = atlas_runtime::Interpreter::new();
    let sec = SecurityContext::allow_all();
    let mut executor = ModuleExecutor::new(&mut interp, &sec, temp_dir.path().to_path_buf());
    let result = executor.execute_module(&main);

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 12.0),
        Ok(v) => panic!("Expected Number(12.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}

#[test]
fn test_import_multiple_functions() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "math",
        r#"
export fn add(borrow a: number, borrow b: number) -> number { return a + b; }
export fn sub(borrow a: number, borrow b: number) -> number { return a - b; }
"#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { add, sub } from "./math";
let sum: number = add(10, 5);
let diff: number = sub(10, 5);
sum + diff;
"#,
    );

    let mut interp = atlas_runtime::Interpreter::new();
    let sec = SecurityContext::allow_all();
    let mut executor = ModuleExecutor::new(&mut interp, &sec, temp_dir.path().to_path_buf());
    let result = executor.execute_module(&main);

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 20.0), // 15 + 5
        Ok(v) => panic!("Expected Number(20.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}

#[test]
fn test_import_variable() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "constants",
        "export let SCALE: number = 4.2;",
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { SCALE } from "./constants";
SCALE * 2;
"#,
    );

    let mut interp = atlas_runtime::Interpreter::new();
    let sec = SecurityContext::allow_all();
    let mut executor = ModuleExecutor::new(&mut interp, &sec, temp_dir.path().to_path_buf());
    let result = executor.execute_module(&main);

    match result {
        Ok(Value::Number(n)) => assert!((n - 8.4).abs() < 0.00001),
        Ok(v) => panic!("Expected Number(8.4), got {:?}", v),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}

#[test]
fn test_import_mixed_function_and_variable() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "utils",
        r#"
export let SCALE: number = 10;
export fn scale(borrow x: number) -> number { return x * SCALE; }
"#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { SCALE, scale } from "./utils";
scale(5);
"#,
    );

    let mut interp = atlas_runtime::Interpreter::new();
    let sec = SecurityContext::allow_all();
    let mut executor = ModuleExecutor::new(&mut interp, &sec, temp_dir.path().to_path_buf());
    let result = executor.execute_module(&main);

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 50.0),
        Ok(v) => panic!("Expected Number(50.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}

#[test]
fn test_dependency_chain_two_levels() {
    let temp_dir = TempDir::new().unwrap();

    create_module(temp_dir.path(), "base", "export let VALUE: number = 100;");

    create_module(
        temp_dir.path(),
        "middle",
        r#"
import { VALUE } from "./base";
export let DOUBLED: number = VALUE * 2;
"#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { DOUBLED } from "./middle";
DOUBLED;
"#,
    );

    let mut interp = atlas_runtime::Interpreter::new();
    let sec = SecurityContext::allow_all();
    let mut executor = ModuleExecutor::new(&mut interp, &sec, temp_dir.path().to_path_buf());
    let result = executor.execute_module(&main);

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 200.0),
        Ok(v) => panic!("Expected Number(200.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}

#[test]
fn test_dependency_chain_three_levels() {
    let temp_dir = TempDir::new().unwrap();

    create_module(temp_dir.path(), "a", "export let X: number = 1;");

    create_module(
        temp_dir.path(),
        "b",
        r#"
import { X } from "./a";
export let Y: number = X + 10;
"#,
    );

    create_module(
        temp_dir.path(),
        "c",
        r#"
import { Y } from "./b";
export let Z: number = Y + 100;
"#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { Z } from "./c";
Z;
"#,
    );

    let mut interp = atlas_runtime::Interpreter::new();
    let sec = SecurityContext::allow_all();
    let mut executor = ModuleExecutor::new(&mut interp, &sec, temp_dir.path().to_path_buf());
    let result = executor.execute_module(&main);

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 111.0), // 1 + 10 + 100
        Ok(v) => panic!("Expected Number(111.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}

#[test]
fn test_diamond_dependency() {
    let temp_dir = TempDir::new().unwrap();

    // Base module
    create_module(temp_dir.path(), "base", "export let VALUE: number = 10;");

    // Left branch
    create_module(
        temp_dir.path(),
        "left",
        r#"
import { VALUE } from "./base";
export let LEFT: number = VALUE + 1;
"#,
    );

    // Right branch
    create_module(
        temp_dir.path(),
        "right",
        r#"
import { VALUE } from "./base";
export let RIGHT: number = VALUE + 2;
"#,
    );

    // Main imports from both branches
    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { LEFT } from "./left";
import { RIGHT } from "./right";
LEFT + RIGHT;
"#,
    );

    let mut interp = atlas_runtime::Interpreter::new();
    let sec = SecurityContext::allow_all();
    let mut executor = ModuleExecutor::new(&mut interp, &sec, temp_dir.path().to_path_buf());
    let result = executor.execute_module(&main);

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 23.0), // 11 + 12
        Ok(v) => panic!("Expected Number(23.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}
