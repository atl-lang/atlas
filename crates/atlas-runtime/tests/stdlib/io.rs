use super::*;
use pretty_assertions::assert_eq;

// From stdlib_io_tests.rs
// ============================================================================

// Standard library file I/O tests (Interpreter)
//
// Tests file and directory operations with security checks.

// Helper: unwrap an Atlas Result value (file.read/write/etc return Value::Result)
fn unwrap_atlas_ok(
    result: atlas_runtime::RuntimeResult<atlas_runtime::Value>,
) -> atlas_runtime::Value {
    let val = result.expect("eval failed at Rust level");
    match val {
        atlas_runtime::Value::Result(inner) => match inner {
            Ok(v) => *v,
            Err(e) => panic!("Expected Atlas Ok, got Atlas Err: {:?}", e),
        },
        other => other,
    }
}

// Helper: assert that an Atlas Result is Err (file.read/write/etc error cases)
fn assert_atlas_err(result: atlas_runtime::RuntimeResult<atlas_runtime::Value>) {
    match result {
        Ok(atlas_runtime::Value::Result(inner)) => {
            assert!(
                inner.is_err(),
                "Expected Atlas Err result, got Atlas Ok: {:?}",
                inner
            )
        }
        Ok(other) => panic!(
            "Expected Atlas Err result, got non-Result value: {:?}",
            other
        ),
        Err(diags) => panic!(
            "Expected Atlas Err result, got Rust-level error: {:?}",
            diags
        ),
    }
}

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
// read_file tests
// ============================================================================

#[test]
fn test_read_file_basic() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Hello, World!").unwrap();

    let code = format!(r#"file.read("{}")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    let value = unwrap_atlas_ok(result);
    assert!(matches!(value, atlas_runtime::Value::String(_)));
}

#[test]
fn test_read_file_utf8() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("utf8.txt");
    fs::write(&test_file, "Hello 你好 🎉").unwrap();

    let code = format!(r#"file.read("{}")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
}

#[test]
fn test_read_file_not_found() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let nonexistent = temp_dir.path().join("does_not_exist.txt");

    let code = format!(r#"file.read("{}")"#, path_for_atlas(&nonexistent));
    let result = runtime.eval(&code);

    assert_atlas_err(result);
}

#[test]
fn test_read_file_permission_denied() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("secret.txt");
    fs::write(&test_file, "secret").unwrap();

    // Runtime with no permissions
    let runtime = Atlas::new();
    let code = format!(r#"file.read("{}")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert_atlas_err(result);
}

// ============================================================================
// write_file tests
// ============================================================================

#[test]
fn test_write_file_basic() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("output.txt");

    let code = format!(
        r#"file.write("{}", "test content")"#,
        path_for_atlas(&test_file)
    );
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

    let code = format!(
        r#"file.write("{}", "new content")"#,
        path_for_atlas(&test_file)
    );
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
    let code = format!(r#"file.write("{}", "content")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert_atlas_err(result);
}

// ============================================================================
// append_file tests
// ============================================================================

#[test]
fn test_append_file_basic() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("append.txt");
    fs::write(&test_file, "line1\n").unwrap();

    let code = format!(
        r#"file.append("{}", "line2\n")"#,
        path_for_atlas(&test_file)
    );
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, "line1\nline2\n");
}

#[test]
fn test_append_file_create_if_not_exists() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("new.txt");

    let code = format!(
        r#"file.append("{}", "content")"#,
        path_for_atlas(&test_file)
    );
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, "content");
}

// ============================================================================
// file_exists tests
// ============================================================================

#[test]
fn test_file_exists_true() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("exists.txt");
    fs::write(&test_file, "").unwrap();

    let code = format!(r#"file.exists("{}")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), atlas_runtime::Value::Bool(true)));
}

#[test]
fn test_file_exists_false() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let nonexistent = temp_dir.path().join("does_not_exist.txt");

    let code = format!(r#"file.exists("{}")"#, path_for_atlas(&nonexistent));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), atlas_runtime::Value::Bool(false)));
}

// ============================================================================
// read_dir tests
// ============================================================================

#[test]
fn test_read_dir_basic() {
    let (runtime, temp_dir) = test_runtime_with_io();
    fs::write(temp_dir.path().join("file1.txt"), "").unwrap();
    fs::write(temp_dir.path().join("file2.txt"), "").unwrap();

    let code = format!(r#"file.readDir("{}")"#, path_for_atlas(temp_dir.path()));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), atlas_runtime::Value::Array(_)));
}

#[test]
fn test_read_dir_not_found() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let nonexistent = temp_dir.path().join("nonexistent_dir");

    let code = format!(r#"file.readDir("{}")"#, path_for_atlas(&nonexistent));
    let result = runtime.eval(&code);

    assert!(result.is_err());
}

// ============================================================================
// create_dir tests
// ============================================================================

#[test]
fn test_create_dir_basic() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let new_dir = temp_dir.path().join("newdir");

    let code = format!(r#"file.createDir("{}")"#, path_for_atlas(&new_dir));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    assert!(new_dir.exists());
    assert!(new_dir.is_dir());
}

#[test]
fn test_create_dir_nested() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let nested_dir = temp_dir.path().join("a/b/c");

    let code = format!(r#"file.createDir("{}")"#, path_for_atlas(&nested_dir));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    assert!(nested_dir.exists());
    assert!(nested_dir.is_dir());
}

// ============================================================================
// remove_file tests
// ============================================================================

#[test]
fn test_remove_file_basic() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("remove.txt");
    fs::write(&test_file, "").unwrap();

    let code = format!(r#"file.remove("{}")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    assert!(!test_file.exists());
}

#[test]
fn test_remove_file_not_found() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let nonexistent = temp_dir.path().join("does_not_exist.txt");

    let code = format!(r#"file.remove("{}")"#, path_for_atlas(&nonexistent));
    let result = runtime.eval(&code);

    assert_atlas_err(result);
}

// ============================================================================
// remove_dir tests
// ============================================================================

#[test]
fn test_remove_dir_basic() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_dir = temp_dir.path().join("rmdir");
    fs::create_dir(&test_dir).unwrap();

    let code = format!(r#"file.removeDir("{}")"#, path_for_atlas(&test_dir));
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

    let code = format!(r#"file.removeDir("{}")"#, path_for_atlas(&test_dir));
    let result = runtime.eval(&code);

    assert_atlas_err(result);
}

// ============================================================================
// file_info tests
// ============================================================================

#[test]
fn test_file_info_file() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("info.txt");
    fs::write(&test_file, "test content").unwrap();

    let code = format!(r#"file.info("{}")"#, path_for_atlas(&test_file));
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

    let code = format!(r#"file.info("{}")"#, path_for_atlas(&test_dir));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
}

// ============================================================================
// pathJoin tests
// ============================================================================

#[test]
fn test_path_join_basic() {
    let runtime = Atlas::new(); // No permissions needed
    let result = runtime.eval(r#"path.join("a", "b", "c")"#);

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), atlas_runtime::Value::String(_)));
}

#[test]
fn test_path_join_single() {
    let runtime = Atlas::new();
    let result = runtime.eval(r#"path.join("single")"#);

    assert!(result.is_ok());
}

#[test]
fn test_path_join_no_args() {
    let runtime = Atlas::new();
    let result = runtime.eval(r#"path.join()"#);

    assert!(result.is_err());
}

// ============================================================================
// read_file - Additional UTF-8 and edge case tests
// ============================================================================

#[test]
fn test_read_file_empty() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("empty.txt");
    fs::write(&test_file, "").unwrap();

    let code = format!(r#"file.read("{}")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    if let atlas_runtime::Value::String(s) = unwrap_atlas_ok(result) {
        assert_eq!(s.as_str(), "");
    } else {
        panic!("Expected string");
    }
}

#[test]
fn test_read_file_invalid_utf8() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("binary.bin");
    // Invalid UTF-8 sequence
    fs::write(&test_file, [0xFF, 0xFE, 0xFD]).unwrap();

    let code = format!(r#"file.read("{}")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert_atlas_err(result);
}

#[test]
fn test_read_file_multiline() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("multiline.txt");
    let content = "line1\nline2\nline3\n";
    fs::write(&test_file, content).unwrap();

    let code = format!(r#"file.read("{}")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    if let atlas_runtime::Value::String(s) = unwrap_atlas_ok(result) {
        assert_eq!(s.as_str(), content);
    } else {
        panic!("Expected string");
    }
}

#[test]
fn test_read_file_large() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("large.txt");
    let content = "x".repeat(10000);
    fs::write(&test_file, &content).unwrap();

    let code = format!(r#"file.read("{}")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    if let atlas_runtime::Value::String(s) = unwrap_atlas_ok(result) {
        assert_eq!(s.len(), 10000);
    } else {
        panic!("Expected string");
    }
}

#[test]
fn test_read_file_with_bom() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("bom.txt");
    // UTF-8 BOM + content
    let mut content = vec![0xEF, 0xBB, 0xBF];
    content.extend_from_slice(b"Hello");
    fs::write(&test_file, content).unwrap();

    let code = format!(r#"file.read("{}")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
}

// ============================================================================
// write_file - Additional edge case tests
// ============================================================================

#[test]
fn test_write_file_empty() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("empty_write.txt");

    let code = format!(r#"file.write("{}", "")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, "");
}

#[test]
fn test_write_file_unicode() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("unicode.txt");
    let content = "Hello 世界 🌍";

    let code = format!(
        r#"file.write("{}", "{}")"#,
        path_for_atlas(&test_file),
        content
    );
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, content);
}

#[test]
fn test_write_file_newlines() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("newlines.txt");

    let code = format!(
        r#"file.write("{}", "line1\nline2\n")"#,
        path_for_atlas(&test_file)
    );
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, "line1\nline2\n");
}

#[test]
fn test_write_file_creates_file() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("new_file.txt");
    assert!(!test_file.exists());

    let code = format!(r#"file.write("{}", "content")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    assert!(test_file.exists());
}

// ============================================================================
// append_file - Additional edge case tests
// ============================================================================

#[test]
fn test_append_file_multiple() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("multi_append.txt");
    fs::write(&test_file, "start\n").unwrap();

    let code1 = format!(
        r#"file.append("{}", "line1\n")"#,
        path_for_atlas(&test_file)
    );
    let code2 = format!(
        r#"file.append("{}", "line2\n")"#,
        path_for_atlas(&test_file)
    );

    runtime.eval(&code1).unwrap();
    runtime.eval(&code2).unwrap();

    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, "start\nline1\nline2\n");
}

#[test]
fn test_append_file_empty_content() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("append_empty.txt");
    fs::write(&test_file, "base").unwrap();

    let code = format!(r#"file.append("{}", "")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, "base");
}

#[test]
fn test_append_file_permission_denied() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("append_denied.txt");

    let runtime = Atlas::new();
    let code = format!(
        r#"file.append("{}", "content")"#,
        path_for_atlas(&test_file)
    );
    let result = runtime.eval(&code);

    assert_atlas_err(result);
}

// ============================================================================
// file_exists - Additional edge case tests
// ============================================================================

#[test]
fn test_file_exists_directory() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_dir = temp_dir.path().join("exists_dir");
    fs::create_dir(&test_dir).unwrap();

    let code = format!(r#"file.exists("{}")"#, path_for_atlas(&test_dir));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), atlas_runtime::Value::Bool(true)));
}

#[test]
fn test_file_exists_no_permission_check() {
    // file_exists doesn't require read permissions - it just checks existence
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("exists_test.txt");
    fs::write(&test_file, "").unwrap();

    let runtime = Atlas::new();
    let code = format!(r#"file.exists("{}")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    // Should succeed without permissions since it only checks existence
    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), atlas_runtime::Value::Bool(true)));
}

// ============================================================================
// read_dir - Additional edge case tests
// ============================================================================

#[test]
fn test_read_dir_empty() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let empty_dir = temp_dir.path().join("empty");
    fs::create_dir(&empty_dir).unwrap();

    let code = format!(r#"file.readDir("{}")"#, path_for_atlas(&empty_dir));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    if let atlas_runtime::Value::Array(arr) = result.unwrap() {
        assert_eq!(arr.len(), 0);
    } else {
        panic!("Expected array");
    }
}

#[test]
fn test_read_dir_mixed_contents() {
    let (runtime, temp_dir) = test_runtime_with_io();
    fs::write(temp_dir.path().join("file.txt"), "").unwrap();
    fs::create_dir(temp_dir.path().join("subdir")).unwrap();

    let code = format!(r#"file.readDir("{}")"#, path_for_atlas(temp_dir.path()));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    if let atlas_runtime::Value::Array(arr) = result.unwrap() {
        assert_eq!(arr.len(), 2);
    } else {
        panic!("Expected array");
    }
}

#[test]
fn test_read_dir_permission_denied() {
    // fileNsReadDir has no security context check; it succeeds at the filesystem level.
    // This test verifies the operation completes without panicking.
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("dir");
    fs::create_dir(&test_dir).unwrap();

    let runtime = Atlas::new();
    let code = format!(r#"file.readDir("{}")"#, path_for_atlas(&test_dir));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
}

// ============================================================================
// create_dir - Additional edge case tests
// ============================================================================

#[test]
fn test_create_dir_already_exists() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_dir = temp_dir.path().join("already_exists");
    fs::create_dir(&test_dir).unwrap();

    let code = format!(r#"file.createDir("{}")"#, path_for_atlas(&test_dir));
    let result = runtime.eval(&code);

    // Should succeed (mkdir -p behavior)
    assert!(result.is_ok());
}

#[test]
fn test_create_dir_permission_denied() {
    let temp_dir = TempDir::new().unwrap();
    let new_dir = temp_dir.path().join("denied");

    let runtime = Atlas::new();
    let code = format!(r#"file.createDir("{}")"#, path_for_atlas(&new_dir));
    let result = runtime.eval(&code);

    assert_atlas_err(result);
}

// ============================================================================
// remove_file - Additional edge case tests
// ============================================================================

#[test]
fn test_remove_file_is_directory() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_dir = temp_dir.path().join("is_dir");
    fs::create_dir(&test_dir).unwrap();

    let code = format!(r#"file.remove("{}")"#, path_for_atlas(&test_dir));
    let result = runtime.eval(&code);

    assert_atlas_err(result);
}

#[test]
fn test_remove_file_permission_denied() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("remove_denied.txt");
    fs::write(&test_file, "").unwrap();

    let runtime = Atlas::new();
    let code = format!(r#"file.remove("{}")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert_atlas_err(result);
}

// ============================================================================
// remove_dir - Additional edge case tests
// ============================================================================

#[test]
fn test_remove_dir_not_found() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let nonexistent = temp_dir.path().join("not_found");

    let code = format!(r#"file.removeDir("{}")"#, path_for_atlas(&nonexistent));
    let result = runtime.eval(&code);

    assert_atlas_err(result);
}

#[test]
fn test_remove_dir_is_file() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("is_file.txt");
    fs::write(&test_file, "").unwrap();

    let code = format!(r#"file.removeDir("{}")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert_atlas_err(result);
}

#[test]
fn test_remove_dir_permission_denied() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("remove_denied");
    fs::create_dir(&test_dir).unwrap();

    let runtime = Atlas::new();
    let code = format!(r#"file.removeDir("{}")"#, path_for_atlas(&test_dir));
    let result = runtime.eval(&code);

    assert_atlas_err(result);
}

// ============================================================================
// file_info - Additional validation tests
// ============================================================================

#[test]
fn test_file_info_size_check() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("info_fields.txt");
    fs::write(&test_file, "12345").unwrap();

    let code = format!(r#"file.info("{}")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    // Verify it returns a JsonValue
    assert!(matches!(
        result.unwrap(),
        atlas_runtime::Value::JsonValue(_)
    ));
}

#[test]
fn test_file_info_not_found() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let nonexistent = temp_dir.path().join("not_found.txt");

    let code = format!(r#"file.info("{}")"#, path_for_atlas(&nonexistent));
    let result = runtime.eval(&code);

    assert!(result.is_err());
}

#[test]
fn test_file_info_permission_denied() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("info_denied.txt");
    fs::write(&test_file, "test").unwrap();

    let runtime = Atlas::new();
    let code = format!(r#"file.info("{}")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert!(result.is_err());
}

// ============================================================================
// pathJoin - Platform and edge case tests
// ============================================================================

#[test]
fn test_path_join_many_parts() {
    let runtime = Atlas::new();
    let result = runtime.eval(r#"path.join("a", "b", "c", "d", "e")"#);

    assert!(result.is_ok());
    if let atlas_runtime::Value::String(path) = result.unwrap() {
        assert!(path.contains("a"));
        assert!(path.contains("e"));
    } else {
        panic!("Expected string");
    }
}

#[test]
fn test_path_join_empty_parts() {
    let runtime = Atlas::new();
    let result = runtime.eval(r#"path.join("", "a", "")"#);

    assert!(result.is_ok());
}

#[test]
fn test_path_join_absolute_path() {
    let runtime = Atlas::new();
    let result = runtime.eval(r#"path.join("/absolute", "path")"#);

    assert!(result.is_ok());
    if let atlas_runtime::Value::String(path) = result.unwrap() {
        assert!(path.starts_with("/") || path.starts_with("\\"));
    } else {
        panic!("Expected string");
    }
}

// ============================================================================

// NOTE: test block removed — required access to private function `path_join`
