use super::{assert_parity, create_module};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_import_parity_basic_function() {
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
            import { add } from "./math";
            add(10, 20);
        "#,
    );

    assert_parity(&main);
}

#[test]
fn test_import_parity_variable() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "constants",
        "export let PI: number = 3.14159;",
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
            import { PI } from "./constants";
            PI * 2;
        "#,
    );

    assert_parity(&main);
}

#[test]
fn test_import_parity_multiple_imports() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "math",
        r#"
            export fn add(borrow a: number, borrow b: number): number { return a + b; }
            export fn multiply(borrow a: number, borrow b: number): number { return a * b; }
        "#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
            import { add, multiply } from "./math";
            add(2, 3) + multiply(4, 5);
        "#,
    );

    assert_parity(&main);
}

#[test]
fn test_import_parity_chained_imports() {
    let temp_dir = TempDir::new().unwrap();

    create_module(temp_dir.path(), "base", "export let VALUE: number = 100;");

    create_module(
        temp_dir.path(),
        "middle",
        r#"
            import { VALUE } from "./base";
            export fn doubled(): number { return VALUE * 2; }
        "#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
            import { doubled } from "./middle";
            doubled();
        "#,
    );

    assert_parity(&main);
}

#[test]
fn test_import_parity_no_imports() {
    let temp_dir = TempDir::new().unwrap();

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
            fn compute(borrow x: number): number { return x * x + 1; }
            compute(5);
        "#,
    );

    assert_parity(&main);
}

#[test]
fn test_import_parity_atlas_extension() {
    let temp_dir = TempDir::new().unwrap();

    let lib_path = temp_dir.path().join("lib.atlas");
    fs::write(
        &lib_path,
        r#"export fn greet(borrow name: string): string { return "Hello " + name; }"#,
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

    assert_parity(&main_path);
}

#[test]
fn test_h173_namespace_import_function_call() {
    // H-173: `import * as ns` stored as HashMap — ns.fn(args) failed with
    // "No method 'fn' on type HashMap" because member calls bypassed callable-field lookup.
    let temp_dir = TempDir::new().unwrap();

    let lib_path = temp_dir.path().join("utils.atlas");
    fs::write(
        &lib_path,
        r#"
export fn add(borrow a: number, borrow b: number): number { return a + b; }
export fn triple(borrow x: number): number { return x * 3; }
export let PI: number = 3.14159;
"#,
    )
    .unwrap();

    let main_path = temp_dir.path().join("main.atlas");
    fs::write(
        &main_path,
        r#"
import * as utils from "./utils";
str(utils.add(2, 3));
"#,
    )
    .unwrap();

    assert_parity(&main_path);
}
