use super::{create_module, execute_with_vm};
use atlas_runtime::Value;
use tempfile::TempDir;

// Module Execution VM Tests (BLOCKER 04-D - VM Parity)

#[test]
fn test_vm_single_module_no_imports() {
    let temp_dir = TempDir::new().unwrap();
    let main = create_module(temp_dir.path(), "main", "let x: number = 42;\nx;");

    let result = execute_with_vm(&main, temp_dir.path());

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 42.0),
        Ok(v) => panic!("Expected Number(42.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {}", e),
    }
}

#[test]
fn test_vm_import_single_function() {
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

    let result = execute_with_vm(&main, temp_dir.path());

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 12.0),
        Ok(v) => panic!("Expected Number(12.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {}", e),
    }
}

#[test]
fn test_vm_import_variable() {
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

    let result = execute_with_vm(&main, temp_dir.path());

    match result {
        Ok(Value::Number(n)) => assert!((n - 8.4).abs() < 0.00001),
        Ok(v) => panic!("Expected Number(8.4), got {:?}", v),
        Err(e) => panic!("Execution failed: {}", e),
    }
}

#[test]
fn test_vm_dependency_chain() {
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

    let result = execute_with_vm(&main, temp_dir.path());

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 200.0),
        Ok(v) => panic!("Expected Number(200.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {}", e),
    }
}

#[test]
fn test_vm_module_executes_once() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "counter",
        r#"
export let mut count: number = 0;
count = count + 1;
"#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { count } from "./counter";
let first: number = count;
import { count } from "./counter";
let second: number = count;
first + second;
"#,
    );

    let result = execute_with_vm(&main, temp_dir.path());

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 2.0), // 1 + 1
        Ok(v) => panic!("Expected Number(2.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {}", e),
    }
}

#[test]
fn test_vm_export_function_and_variable() {
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
import { scale } from "./utils";
scale(5);
"#,
    );

    let result = execute_with_vm(&main, temp_dir.path());

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 50.0),
        Ok(v) => panic!("Expected Number(50.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {}", e),
    }
}

#[test]
fn test_vm_diamond_dependency() {
    let temp_dir = TempDir::new().unwrap();

    create_module(temp_dir.path(), "base", "export let VALUE: number = 10;");

    create_module(
        temp_dir.path(),
        "left",
        r#"
import { VALUE } from "./base";
export let LEFT: number = VALUE + 1;
"#,
    );

    create_module(
        temp_dir.path(),
        "right",
        r#"
import { VALUE } from "./base";
export let RIGHT: number = VALUE + 2;
"#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { LEFT } from "./left";
import { RIGHT } from "./right";
LEFT + RIGHT;
"#,
    );

    let result = execute_with_vm(&main, temp_dir.path());

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 23.0), // 11 + 12
        Ok(v) => panic!("Expected Number(23.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {}", e),
    }
}

#[test]
fn test_vm_multiple_imports() {
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

    let result = execute_with_vm(&main, temp_dir.path());

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 20.0), // 15 + 5
        Ok(v) => panic!("Expected Number(20.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {}", e),
    }
}

#[test]
fn test_vm_string_export() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "config",
        r#"export let NAME: string = "Atlas";"#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { NAME } from "./config";
NAME;
"#,
    );

    let result = execute_with_vm(&main, temp_dir.path());

    match result {
        Ok(Value::String(s)) => assert_eq!(*s, "Atlas"),
        Ok(v) => panic!("Expected String(Atlas), got {:?}", v),
        Err(e) => panic!("Execution failed: {}", e),
    }
}

#[test]
fn test_vm_boolean_export() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "flags",
        r#"export let DEBUG: bool = true;"#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { DEBUG } from "./flags";
DEBUG;
"#,
    );

    let result = execute_with_vm(&main, temp_dir.path());

    match result {
        Ok(Value::Bool(b)) => assert!(b),
        Ok(v) => panic!("Expected Bool(true), got {:?}", v),
        Err(e) => panic!("Execution failed: {}", e),
    }
}
