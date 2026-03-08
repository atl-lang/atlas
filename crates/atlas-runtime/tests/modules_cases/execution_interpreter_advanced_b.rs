use super::create_module;
use atlas_runtime::{ModuleExecutor, SecurityContext, Value};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_imported_function_preserves_types() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "string_ops",
        r#"
export fn concatStrings(borrow a: string, borrow b: string) -> string {
    return a + b;
}
"#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { concatStrings } from "./string_ops";
concatStrings("Hello", " World");
"#,
    );

    let mut interp = atlas_runtime::Interpreter::new();
    let sec = SecurityContext::allow_all();
    let mut executor = ModuleExecutor::new(&mut interp, &sec, temp_dir.path().to_path_buf());
    let result = executor.execute_module(&main);

    match result {
        Ok(Value::String(s)) => assert_eq!(*s, "Hello World"),
        Ok(v) => panic!("Expected String, got {:?}", v),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}

#[test]
fn test_imported_array_preserves_type() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "data",
        r#"
export let numbers: number[] = [1, 2, 3];
"#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { numbers } from "./data";
len(numbers);
"#,
    );

    let mut interp = atlas_runtime::Interpreter::new();
    let sec = SecurityContext::allow_all();
    let mut executor = ModuleExecutor::new(&mut interp, &sec, temp_dir.path().to_path_buf());
    let result = executor.execute_module(&main);

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 3.0),
        Ok(v) => panic!("Expected Number(3.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}

#[test]
fn test_private_function_not_accessible() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "math",
        r#"
fn private_helper(borrow x: number) -> number {
    return x * 2;
}

export fn public_fn(borrow x: number) -> number {
    return private_helper(x);
}
"#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { private_helper } from "./math";
private_helper(5);
"#,
    );

    let mut interp = atlas_runtime::Interpreter::new();
    let sec = SecurityContext::allow_all();
    let mut executor = ModuleExecutor::new(&mut interp, &sec, temp_dir.path().to_path_buf());
    let result = executor.execute_module(&main);

    // Should fail - private_helper is not exported
    assert!(result.is_err());
}

#[test]
fn test_private_variable_not_accessible() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "config",
        r#"
let SECRET: string = "hidden";
export let PUBLIC: string = "visible";
"#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { SECRET } from "./config";
SECRET;
"#,
    );

    let mut interp = atlas_runtime::Interpreter::new();
    let sec = SecurityContext::allow_all();
    let mut executor = ModuleExecutor::new(&mut interp, &sec, temp_dir.path().to_path_buf());
    let result = executor.execute_module(&main);

    // Should fail - SECRET is not exported
    assert!(result.is_err());
}

#[test]
fn test_import_resolves_atlas_extension() {
    let temp_dir = TempDir::new().unwrap();

    let lib_path = temp_dir.path().join("lib.atlas");
    fs::write(
        &lib_path,
        r#"export fn greet(borrow name: string) -> string { return "Hello " + name; }"#,
    )
    .unwrap();

    let main_path = temp_dir.path().join("main.atlas");
    fs::write(
        &main_path,
        r#"
import { greet } from "./lib";
greet("World");
"#,
    )
    .unwrap();

    let mut interp = atlas_runtime::Interpreter::new();
    let sec = SecurityContext::allow_all();
    let mut executor = ModuleExecutor::new(&mut interp, &sec, temp_dir.path().to_path_buf());
    let result = executor.execute_module(&main_path);

    match result {
        Ok(Value::String(s)) => assert_eq!(*s, "Hello World"),
        Ok(v) => panic!("Expected String, got {:?}", v),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}
