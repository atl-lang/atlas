//! Standard library file I/O tests (Interpreter)
//!
//! Tests file and directory operations with security checks.

use atlas_runtime::{Atlas, SecurityContext};
use std::fs;
use tempfile::TempDir;

// Helper to create runtime with full filesystem permissions
fn test_runtime_with_io() -> (Atlas, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let mut security = SecurityContext::new();
    security.grant_filesystem_read(temp_dir.path(), true);
    security.grant_filesystem_write(temp_dir.path(), true);
    let runtime = Atlas::new_with_security(security);
    (runtime, temp_dir)
}

// ============================================================================
// readFile tests
// ============================================================================

#[test]
fn test_read_file_basic() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Hello, World!").unwrap();

    let code = format!(r#"readFile("{}")"#, test_file.display());
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(matches!(value, atlas_runtime::Value::String(_)));
}

#[test]
fn test_read_file_utf8() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("utf8.txt");
    fs::write(&test_file, "Hello 你好 🎉").unwrap();

    let code = format!(r#"readFile("{}")"#, test_file.display());
    let result = runtime.eval(&code);

    assert!(result.is_ok());
}

#[test]
fn test_read_file_not_found() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let nonexistent = temp_dir.path().join("does_not_exist.txt");

    let code = format!(r#"readFile("{}")"#, nonexistent.display());
    let result = runtime.eval(&code);

    assert!(result.is_err());
    let diagnostics = result.unwrap_err();
    assert!(diagnostics[0].message.contains("Failed to resolve path"));
}

#[test]
fn test_read_file_permission_denied() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("secret.txt");
    fs::write(&test_file, "secret").unwrap();

    // Runtime with no permissions
    let runtime = Atlas::new();
    let code = format!(r#"readFile("{}")"#, test_file.display());
    let result = runtime.eval(&code);

    assert!(result.is_err());
    let diagnostics = result.unwrap_err();
    assert_eq!(diagnostics[0].code, "AT0300");
    assert!(diagnostics[0].message.contains("Permission denied"));
}

// ============================================================================
// writeFile tests
// ============================================================================

#[test]
fn test_write_file_basic() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("output.txt");

    let code = format!(r#"writeFile("{}", "test content")"#, test_file.display());
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, "test content");
}

#[test]
fn test_write_file_overwrite() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("overwrite.txt");
    fs::write(&test_file, "original").unwrap();

    let code = format!(r#"writeFile("{}", "new content")"#, test_file.display());
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, "new content");
}

#[test]
fn test_write_file_permission_denied() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("output.txt");

    let runtime = Atlas::new();
    let code = format!(r#"writeFile("{}", "content")"#, test_file.display());
    let result = runtime.eval(&code);

    assert!(result.is_err());
    let diagnostics = result.unwrap_err();
    assert_eq!(diagnostics[0].code, "AT0300");
}

// ============================================================================
// appendFile tests
// ============================================================================

#[test]
fn test_append_file_basic() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("append.txt");
    fs::write(&test_file, "line1\n").unwrap();

    let code = format!(r#"appendFile("{}", "line2\n")"#, test_file.display());
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, "line1\nline2\n");
}

#[test]
fn test_append_file_create_if_not_exists() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("new.txt");

    let code = format!(r#"appendFile("{}", "content")"#, test_file.display());
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, "content");
}

// ============================================================================
// fileExists tests
// ============================================================================

#[test]
fn test_file_exists_true() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("exists.txt");
    fs::write(&test_file, "").unwrap();

    let code = format!(r#"fileExists("{}")"#, test_file.display());
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), atlas_runtime::Value::Bool(true)));
}

#[test]
fn test_file_exists_false() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let nonexistent = temp_dir.path().join("does_not_exist.txt");

    let code = format!(r#"fileExists("{}")"#, nonexistent.display());
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), atlas_runtime::Value::Bool(false)));
}

// ============================================================================
// readDir tests
// ============================================================================

#[test]
fn test_read_dir_basic() {
    let (runtime, temp_dir) = test_runtime_with_io();
    fs::write(temp_dir.path().join("file1.txt"), "").unwrap();
    fs::write(temp_dir.path().join("file2.txt"), "").unwrap();

    let code = format!(r#"readDir("{}")"#, temp_dir.path().display());
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), atlas_runtime::Value::Array(_)));
}

#[test]
fn test_read_dir_not_found() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let nonexistent = temp_dir.path().join("nonexistent_dir");

    let code = format!(r#"readDir("{}")"#, nonexistent.display());
    let result = runtime.eval(&code);

    assert!(result.is_err());
}

// ============================================================================
// createDir tests
// ============================================================================

#[test]
fn test_create_dir_basic() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let new_dir = temp_dir.path().join("newdir");

    let code = format!(r#"createDir("{}")"#, new_dir.display());
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    assert!(new_dir.exists());
    assert!(new_dir.is_dir());
}

#[test]
fn test_create_dir_nested() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let nested_dir = temp_dir.path().join("a/b/c");

    let code = format!(r#"createDir("{}")"#, nested_dir.display());
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    assert!(nested_dir.exists());
    assert!(nested_dir.is_dir());
}

// ============================================================================
// removeFile tests
// ============================================================================

#[test]
fn test_remove_file_basic() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("remove.txt");
    fs::write(&test_file, "").unwrap();

    let code = format!(r#"removeFile("{}")"#, test_file.display());
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    assert!(!test_file.exists());
}

#[test]
fn test_remove_file_not_found() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let nonexistent = temp_dir.path().join("does_not_exist.txt");

    let code = format!(r#"removeFile("{}")"#, nonexistent.display());
    let result = runtime.eval(&code);

    assert!(result.is_err());
}

// ============================================================================
// removeDir tests
// ============================================================================

#[test]
fn test_remove_dir_basic() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_dir = temp_dir.path().join("rmdir");
    fs::create_dir(&test_dir).unwrap();

    let code = format!(r#"removeDir("{}")"#, test_dir.display());
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    assert!(!test_dir.exists());
}

#[test]
fn test_remove_dir_not_empty() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_dir = temp_dir.path().join("notempty");
    fs::create_dir(&test_dir).unwrap();
    fs::write(test_dir.join("file.txt"), "").unwrap();

    let code = format!(r#"removeDir("{}")"#, test_dir.display());
    let result = runtime.eval(&code);

    assert!(result.is_err());
    let diagnostics = result.unwrap_err();
    assert!(diagnostics[0]
        .message
        .contains("Failed to remove directory"));
}

// ============================================================================
// fileInfo tests
// ============================================================================

#[test]
fn test_file_info_file() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("info.txt");
    fs::write(&test_file, "test content").unwrap();

    let code = format!(r#"fileInfo("{}")"#, test_file.display());
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    // Result should be a JsonValue object
    assert!(matches!(
        result.unwrap(),
        atlas_runtime::Value::JsonValue(_)
    ));
}

#[test]
fn test_file_info_directory() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_dir = temp_dir.path().join("infodir");
    fs::create_dir(&test_dir).unwrap();

    let code = format!(r#"fileInfo("{}")"#, test_dir.display());
    let result = runtime.eval(&code);

    assert!(result.is_ok());
}

// ============================================================================
// pathJoin tests
// ============================================================================

#[test]
fn test_path_join_basic() {
    let runtime = Atlas::new(); // No permissions needed
    let result = runtime.eval(r#"pathJoin("a", "b", "c")"#);

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), atlas_runtime::Value::String(_)));
}

#[test]
fn test_path_join_single() {
    let runtime = Atlas::new();
    let result = runtime.eval(r#"pathJoin("single")"#);

    assert!(result.is_ok());
}

#[test]
fn test_path_join_no_args() {
    let runtime = Atlas::new();
    let result = runtime.eval(r#"pathJoin()"#);

    assert!(result.is_err());
}
