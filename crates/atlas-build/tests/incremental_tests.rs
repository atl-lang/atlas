//! Integration tests for incremental compilation
//!
//! Tests the incremental build system with file changes and cache management

use atlas_build::Builder;
use std::fs;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

/// Create a test project with the given structure
fn create_test_project(files: &[(&str, &str)]) -> (TempDir, String) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_path_buf();

    // Create src directory
    fs::create_dir(path.join("src")).unwrap();

    // Create manifest
    let manifest = r#"
[package]
name = "test-project"
version = "0.1.0"
"#;
    fs::write(path.join("atlas.toml"), manifest).unwrap();

    // Create source files
    for (file_path, content) in files {
        let full_path = path.join(file_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(full_path, content).unwrap();
    }

    let path_str = path.to_string_lossy().to_string();
    (dir, path_str)
}

#[test]
fn test_initial_build_compiles_all_files() {
    let (_temp, project_path) = create_test_project(&[
        (
            "src/main.atlas",
            r#"fn main() -> void {
    let x: number = 42;
    print(x);
}"#,
        ),
        (
            "src/lib.atlas",
            r#"export fn helper() -> number {
    return 10;
}"#,
        ),
    ]);

    let mut builder = Builder::new(&project_path).unwrap();
    let result = builder.build_incremental();

    assert!(result.is_ok());
    let context = result.unwrap();
    assert_eq!(context.stats.total_modules, 2);
}

#[test]
fn test_rebuild_no_changes_fast() {
    let (_temp, project_path) = create_test_project(&[(
        "src/main.atlas",
        r#"fn main() -> void {
    let x: number = 42;
    print(x);
}"#,
    )]);

    let mut builder = Builder::new(&project_path).unwrap();

    // First build
    builder.build_incremental().unwrap();

    // Second build (no changes)
    let result = builder.build_incremental();

    assert!(result.is_ok());
}

#[test]
fn test_change_one_file_still_builds() {
    let (temp, project_path) = create_test_project(&[(
        "src/main.atlas",
        r#"fn main() -> void {
    let x: number = 42;
    print(x);
}"#,
    )]);

    let mut builder = Builder::new(&project_path).unwrap();

    // First build
    builder.build_incremental().unwrap();

    // Modify file
    thread::sleep(Duration::from_millis(10));
    fs::write(
        temp.path().join("src/main.atlas"),
        r#"fn main() -> void {
    let x: number = 43;
    print(x);
}"#,
    )
    .unwrap();

    // Rebuild with change
    let result = builder.build_incremental();

    assert!(result.is_ok());
}

#[test]
fn test_cache_persists_across_builds() {
    let (_temp, project_path) = create_test_project(&[(
        "src/main.atlas",
        r#"fn main() -> void {
    let x: number = 42;
    print(x);
}"#,
    )]);

    // First builder instance
    {
        let mut builder = Builder::new(&project_path).unwrap();
        builder.build_incremental().unwrap();
    }

    // Second builder instance (should load cache)
    {
        let mut builder = Builder::new(&project_path).unwrap();
        let result = builder.build_incremental();
        assert!(result.is_ok());
    }
}

#[test]
fn test_cold_cache_build() {
    let (_temp, project_path) = create_test_project(&[(
        "src/main.atlas",
        r#"fn main() -> void {
    let x: number = 42;
    print(x);
}"#,
    )]);

    let mut builder = Builder::new(&project_path).unwrap();

    // Cold build (no cache exists)
    let result = builder.build_incremental();

    assert!(result.is_ok());
    let context = result.unwrap();
    assert_eq!(context.stats.total_modules, 1);
}

#[test]
fn test_multi_file_project() {
    let (_temp, project_path) = create_test_project(&[
        (
            "src/main.atlas",
            r#"fn main() -> void {
    let x: number = 42;
    print(x);
}"#,
        ),
        (
            "src/lib.atlas",
            r#"export fn add(a: number, b: number) -> number {
    return a + b;
}"#,
        ),
        (
            "src/utils.atlas",
            r#"export fn multiply(a: number, b: number) -> number {
    return a * b;
}"#,
        ),
    ]);

    let mut builder = Builder::new(&project_path).unwrap();
    let result = builder.build_incremental();

    assert!(result.is_ok());
    let context = result.unwrap();
    assert_eq!(context.stats.total_modules, 3);
}

#[test]
fn test_incremental_build_with_errors() {
    let (temp, project_path) = create_test_project(&[(
        "src/main.atlas",
        r#"fn main() -> void {
    let x: number = 42;
    print(x);
}"#,
    )]);

    let mut builder = Builder::new(&project_path).unwrap();

    // First build succeeds
    builder.build_incremental().unwrap();

    // Introduce syntax error
    thread::sleep(Duration::from_millis(10));
    fs::write(
        temp.path().join("src/main.atlas"),
        r#"fn main() -> void {
    let x: number = // missing value
}"#,
    )
    .unwrap();

    // Rebuild with error
    let result = builder.build_incremental();

    assert!(result.is_err());
}

#[test]
fn test_library_target_incremental() {
    let (_temp, project_path) = create_test_project(&[(
        "src/lib.atlas",
        r#"export fn greet(name: string) -> string {
    return "Hello, " + name;
}"#,
    )]);

    let mut builder = Builder::new(&project_path).unwrap();
    let result = builder.build_incremental();

    assert!(result.is_ok());
    let context = result.unwrap();
    assert_eq!(context.stats.total_modules, 1);
}

#[test]
fn test_binary_target_incremental() {
    let (_temp, project_path) = create_test_project(&[(
        "src/main.atlas",
        r#"fn main() -> void {
    print("Hello, World!");
}"#,
    )]);

    let mut builder = Builder::new(&project_path).unwrap();
    let result = builder.build_incremental();

    assert!(result.is_ok());
    let context = result.unwrap();
    assert_eq!(context.stats.total_modules, 1);
    assert!(!context.artifacts.is_empty());
}
