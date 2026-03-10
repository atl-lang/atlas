use super::create_module;
use atlas_runtime::{ModuleExecutor, SecurityContext, Value};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_module_executes_once() {
    let temp_dir = TempDir::new().unwrap();

    // Module with side effect - increments a counter
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

    let mut interp = atlas_runtime::Interpreter::new();
    let sec = SecurityContext::allow_all();
    let mut executor = ModuleExecutor::new(&mut interp, &sec, temp_dir.path().to_path_buf());
    let result = executor.execute_module(&main);

    // Both imports should get count=1 (module executed once)
    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 2.0), // 1 + 1
        Ok(v) => panic!("Expected Number(2.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}
#[test]
fn test_shared_module_executes_once() {
    let temp_dir = TempDir::new().unwrap();

    // Shared module with counter
    create_module(
        temp_dir.path(),
        "shared",
        r#"
export let mut counter: number = 0;
counter = counter + 1;
"#,
    );

    // Module A imports shared
    create_module(
        temp_dir.path(),
        "a",
        r#"
import { counter } from "./shared";
export let A_COUNT: number = counter;
"#,
    );

    // Module B also imports shared
    create_module(
        temp_dir.path(),
        "b",
        r#"
import { counter } from "./shared";
export let B_COUNT: number = counter;
"#,
    );

    // Main imports from both A and B
    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { A_COUNT } from "./a";
import { B_COUNT } from "./b";
A_COUNT + B_COUNT;
"#,
    );

    let mut interp = atlas_runtime::Interpreter::new();
    let sec = SecurityContext::allow_all();
    let mut executor = ModuleExecutor::new(&mut interp, &sec, temp_dir.path().to_path_buf());
    let result = executor.execute_module(&main);

    // shared module executes once, so both get counter=1
    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 2.0), // 1 + 1
        Ok(v) => panic!("Expected Number(2.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}
#[test]
fn test_import_nonexistent_export() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "math",
        "export fn add(borrow a: number, borrow b: number): number { return a + b; }",
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { subtract } from "./math";
subtract(5, 3);
"#,
    );

    let mut interp = atlas_runtime::Interpreter::new();
    let sec = SecurityContext::allow_all();
    let mut executor = ModuleExecutor::new(&mut interp, &sec, temp_dir.path().to_path_buf());
    let result = executor.execute_module(&main);

    assert!(result.is_err());
    if let Err(diagnostics) = result {
        assert!(!diagnostics.is_empty());
        assert!(diagnostics[0].message.contains("not exported"));
    }
}

#[test]
fn test_import_from_nonexistent_module() {
    let temp_dir = TempDir::new().unwrap();

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { foo } from "./nonexistent";
foo;
"#,
    );

    let mut interp = atlas_runtime::Interpreter::new();
    let sec = SecurityContext::allow_all();
    let mut executor = ModuleExecutor::new(&mut interp, &sec, temp_dir.path().to_path_buf());
    let result = executor.execute_module(&main);

    assert!(result.is_err());
    if let Err(diagnostics) = result {
        assert!(!diagnostics.is_empty());
        assert!(
            diagnostics[0].message.contains("not found")
                || diagnostics[0].message.contains("Module not found")
        );
    }
}
#[test]
fn test_circular_import_error() {
    let temp_dir = TempDir::new().unwrap();

    // Create circular dependency: a imports b, b imports a
    create_module(
        temp_dir.path(),
        "a",
        r#"
import { b_val } from "./b";
export let a_val: number = b_val + 1;
"#,
    );

    create_module(
        temp_dir.path(),
        "b",
        r#"
import { a_val } from "./a";
export let b_val: number = a_val + 1;
"#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { a_val } from "./a";
a_val;
"#,
    );

    let mut interp = atlas_runtime::Interpreter::new();
    let sec = SecurityContext::allow_all();
    let mut executor = ModuleExecutor::new(&mut interp, &sec, temp_dir.path().to_path_buf());
    let result = executor.execute_module(&main);

    // Should fail with circular dependency error, NOT infinite loop
    assert!(result.is_err());
    if let Err(diagnostics) = result {
        assert!(!diagnostics.is_empty());
        assert!(
            diagnostics[0].message.contains("Circular")
                || diagnostics[0].message.contains("circular"),
            "Expected circular dependency error, got: {}",
            diagnostics[0].message
        );
    }
}
#[test]
fn test_multiple_imports_from_different_modules() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "math",
        "export fn add(borrow a: number, borrow b: number): number { return a + b; }",
    );

    create_module(
        temp_dir.path(),
        "string_utils",
        r#"export fn greeting(borrow name: string): string { return "Hello, " + name; }"#,
    );

    create_module(
        temp_dir.path(),
        "constants",
        "export let MAX: number = 100;",
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { add } from "./math";
import { greeting } from "./string_utils";
import { MAX } from "./constants";
add(50, 50);
"#,
    );

    let mut interp = atlas_runtime::Interpreter::new();
    let sec = SecurityContext::allow_all();
    let mut executor = ModuleExecutor::new(&mut interp, &sec, temp_dir.path().to_path_buf());
    let result = executor.execute_module(&main);

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 100.0),
        Ok(v) => panic!("Expected Number(100.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}

#[test]
fn test_exported_function_uses_local_helper() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "math",
        r#"
fn helper(borrow x: number): number {
    return x * 2;
}

export fn double(borrow x: number): number {
    return helper(x);
}
"#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { double } from "./math";
double(21);
"#,
    );

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
fn test_exported_function_uses_exported_variable() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "config",
        r#"
export let MULTIPLIER: number = 3;
export fn multiply(borrow x: number): number {
    return x * MULTIPLIER;
}
"#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { multiply, MULTIPLIER } from "./config";
multiply(10);
"#,
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
fn test_relative_import_from_subdirectory() {
    let temp_dir = TempDir::new().unwrap();
    let sub_dir = temp_dir.path().join("lib");
    fs::create_dir(&sub_dir).unwrap();

    create_module(&sub_dir, "helper", "export let VALUE: number = 42;");

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { VALUE } from "./lib/helper";
VALUE;
"#,
    );

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
fn test_parent_directory_import() {
    let temp_dir = TempDir::new().unwrap();
    let sub_dir = temp_dir.path().join("src");
    fs::create_dir(&sub_dir).unwrap();

    create_module(temp_dir.path(), "config", "export let PORT: number = 8080;");

    let main = create_module(
        &sub_dir,
        "server",
        r#"
import { PORT } from "../config";
PORT;
"#,
    );

    let mut interp = atlas_runtime::Interpreter::new();
    let sec = SecurityContext::allow_all();
    let mut executor = ModuleExecutor::new(&mut interp, &sec, temp_dir.path().to_path_buf());
    let result = executor.execute_module(&main);

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 8080.0),
        Ok(v) => panic!("Expected Number(8080.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}
