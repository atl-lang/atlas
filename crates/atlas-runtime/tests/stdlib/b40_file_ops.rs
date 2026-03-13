//! B40: File operations tests (file.rename, file.copy)
//!
//! Tests for H-282 (file.rename) and H-283 (file.copy)

use atlas_runtime::runtime::Atlas;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::value::Value;
use std::fs;
use tempfile::TempDir;

/// Helper to escape path for Atlas string
fn path_for_atlas(path: &std::path::Path) -> String {
    path.to_string_lossy().replace('\\', "\\\\")
}

// ============================================================================
// file.rename Tests (H-282)
// ============================================================================

#[test]
fn test_file_rename_basic() {
    let temp_dir = TempDir::new().unwrap();
    let src = temp_dir.path().join("source.txt");
    let dst = temp_dir.path().join("dest.txt");
    fs::write(&src, "content").unwrap();

    let code = format!(
        r#"file.rename("{}", "{}")"#,
        path_for_atlas(&src),
        path_for_atlas(&dst)
    );

    let mut security = SecurityContext::new();
    security.grant_filesystem_write(temp_dir.path(), true);
    security.grant_filesystem_read(temp_dir.path(), true);
    let runtime = Atlas::new_with_security(security);
    let result = runtime.eval(&code).unwrap();

    // Should return Ok(null)
    assert!(matches!(result, Value::Result(Ok(_))));
    assert!(!src.exists());
    assert!(dst.exists());
    assert_eq!(fs::read_to_string(&dst).unwrap(), "content");
}

#[test]
fn test_file_rename_nonexistent_source() {
    let temp_dir = TempDir::new().unwrap();
    let src = temp_dir.path().join("nonexistent.txt");
    let dst = temp_dir.path().join("dest.txt");

    let code = format!(
        r#"file.rename("{}", "{}")"#,
        path_for_atlas(&src),
        path_for_atlas(&dst)
    );

    let mut security = SecurityContext::new();
    security.grant_filesystem_write(temp_dir.path(), true);
    security.grant_filesystem_read(temp_dir.path(), true);
    let runtime = Atlas::new_with_security(security);
    let result = runtime.eval(&code).unwrap();

    // Should return Err
    assert!(matches!(result, Value::Result(Err(_))));
}

#[test]
fn test_file_rename_to_different_name() {
    let temp_dir = TempDir::new().unwrap();
    let src = temp_dir.path().join("old_name.txt");
    let dst = temp_dir.path().join("new_name.txt");
    fs::write(&src, "data").unwrap();

    let code = format!(
        r#"
        let result = file.rename("{}", "{}");
        match result {{
            Ok(_) => "success",
            Err(e) => e
        }}
        "#,
        path_for_atlas(&src),
        path_for_atlas(&dst)
    );

    let mut security = SecurityContext::new();
    security.grant_filesystem_write(temp_dir.path(), true);
    security.grant_filesystem_read(temp_dir.path(), true);
    let runtime = Atlas::new_with_security(security);
    let result = runtime.eval(&code).unwrap();

    assert_eq!(result.to_string(), "success");
}

#[test]
fn test_file_rename_preserves_content() {
    let temp_dir = TempDir::new().unwrap();
    let src = temp_dir.path().join("src.txt");
    let dst = temp_dir.path().join("dst.txt");
    let content = "Important data that must be preserved";
    fs::write(&src, content).unwrap();

    let code = format!(
        r#"
        file.rename("{}", "{}")?;
        file.read("{}")
        "#,
        path_for_atlas(&src),
        path_for_atlas(&dst),
        path_for_atlas(&dst)
    );

    let mut security = SecurityContext::new();
    security.grant_filesystem_write(temp_dir.path(), true);
    security.grant_filesystem_read(temp_dir.path(), true);
    let runtime = Atlas::new_with_security(security);
    let result = runtime.eval(&code).unwrap();

    // Result type wrapping
    if let Value::Result(Ok(inner)) = result {
        assert_eq!(inner.to_string(), content);
    } else {
        assert_eq!(result.to_string(), content);
    }
}

// ============================================================================
// file.copy Tests (H-283)
// ============================================================================

#[test]
fn test_file_copy_basic() {
    let temp_dir = TempDir::new().unwrap();
    let src = temp_dir.path().join("source.txt");
    let dst = temp_dir.path().join("copy.txt");
    fs::write(&src, "original content").unwrap();

    let code = format!(
        r#"file.copy("{}", "{}")"#,
        path_for_atlas(&src),
        path_for_atlas(&dst)
    );

    let mut security = SecurityContext::new();
    security.grant_filesystem_write(temp_dir.path(), true);
    security.grant_filesystem_read(temp_dir.path(), true);
    let runtime = Atlas::new_with_security(security);
    let result = runtime.eval(&code).unwrap();

    // Should return Ok(null)
    assert!(matches!(result, Value::Result(Ok(_))));
    // Source should still exist
    assert!(src.exists());
    // Destination should exist with same content
    assert!(dst.exists());
    assert_eq!(fs::read_to_string(&dst).unwrap(), "original content");
}

#[test]
fn test_file_copy_nonexistent_source() {
    let temp_dir = TempDir::new().unwrap();
    let src = temp_dir.path().join("does_not_exist.txt");
    let dst = temp_dir.path().join("dest.txt");

    let code = format!(
        r#"file.copy("{}", "{}")"#,
        path_for_atlas(&src),
        path_for_atlas(&dst)
    );

    let mut security = SecurityContext::new();
    security.grant_filesystem_write(temp_dir.path(), true);
    security.grant_filesystem_read(temp_dir.path(), true);
    let runtime = Atlas::new_with_security(security);
    let result = runtime.eval(&code).unwrap();

    // Should return Err
    assert!(matches!(result, Value::Result(Err(_))));
}

#[test]
fn test_file_copy_preserves_original() {
    let temp_dir = TempDir::new().unwrap();
    let src = temp_dir.path().join("src.txt");
    let dst = temp_dir.path().join("dst.txt");
    let content = "Keep this content";
    fs::write(&src, content).unwrap();

    let code = format!(
        r#"
        file.copy("{}", "{}")?;
        file.read("{}")
        "#,
        path_for_atlas(&src),
        path_for_atlas(&dst),
        path_for_atlas(&src) // Read source after copy
    );

    let mut security = SecurityContext::new();
    security.grant_filesystem_write(temp_dir.path(), true);
    security.grant_filesystem_read(temp_dir.path(), true);
    let runtime = Atlas::new_with_security(security);
    let result = runtime.eval(&code).unwrap();

    // Source should still have original content
    if let Value::Result(Ok(inner)) = result {
        assert_eq!(inner.to_string(), content);
    } else {
        assert_eq!(result.to_string(), content);
    }
}

#[test]
fn test_file_copy_overwrites_existing() {
    let temp_dir = TempDir::new().unwrap();
    let src = temp_dir.path().join("src.txt");
    let dst = temp_dir.path().join("dst.txt");
    fs::write(&src, "new content").unwrap();
    fs::write(&dst, "old content").unwrap();

    let code = format!(
        r#"
        file.copy("{}", "{}")?;
        file.read("{}")
        "#,
        path_for_atlas(&src),
        path_for_atlas(&dst),
        path_for_atlas(&dst)
    );

    let mut security = SecurityContext::new();
    security.grant_filesystem_write(temp_dir.path(), true);
    security.grant_filesystem_read(temp_dir.path(), true);
    let runtime = Atlas::new_with_security(security);
    let result = runtime.eval(&code).unwrap();

    // Destination should have new content
    if let Value::Result(Ok(inner)) = result {
        assert_eq!(inner.to_string(), "new content");
    } else {
        assert_eq!(result.to_string(), "new content");
    }
}

#[test]
fn test_file_copy_large_file() {
    let temp_dir = TempDir::new().unwrap();
    let src = temp_dir.path().join("large.txt");
    let dst = temp_dir.path().join("large_copy.txt");

    // Create a larger file (10KB)
    let content: String = "x".repeat(10 * 1024);
    fs::write(&src, &content).unwrap();

    let code = format!(
        r#"
        file.copy("{}", "{}")?;
        len(file.read("{}")?)
        "#,
        path_for_atlas(&src),
        path_for_atlas(&dst),
        path_for_atlas(&dst)
    );

    let mut security = SecurityContext::new();
    security.grant_filesystem_write(temp_dir.path(), true);
    security.grant_filesystem_read(temp_dir.path(), true);
    let runtime = Atlas::new_with_security(security);
    let result = runtime.eval(&code).unwrap();

    // Length should match
    assert_eq!(result, Value::Number(10240.0));
}

// ============================================================================
// Combined operations
// ============================================================================

#[test]
fn test_file_copy_then_rename() {
    let temp_dir = TempDir::new().unwrap();
    let src = temp_dir.path().join("original.txt");
    let copy = temp_dir.path().join("copy.txt");
    let renamed = temp_dir.path().join("renamed.txt");
    fs::write(&src, "data").unwrap();

    let code = format!(
        r#"
        file.copy("{}", "{}")?;
        file.rename("{}", "{}")?;
        file.exists("{}")
        "#,
        path_for_atlas(&src),
        path_for_atlas(&copy),
        path_for_atlas(&copy),
        path_for_atlas(&renamed),
        path_for_atlas(&renamed)
    );

    let mut security = SecurityContext::new();
    security.grant_filesystem_write(temp_dir.path(), true);
    security.grant_filesystem_read(temp_dir.path(), true);
    let runtime = Atlas::new_with_security(security);
    let result = runtime.eval(&code).unwrap();

    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_file_rename_then_copy() {
    let temp_dir = TempDir::new().unwrap();
    let src = temp_dir.path().join("start.txt");
    let renamed = temp_dir.path().join("middle.txt");
    let copied = temp_dir.path().join("end.txt");
    fs::write(&src, "test data").unwrap();

    let code = format!(
        r#"
        file.rename("{}", "{}")?;
        file.copy("{}", "{}")?;
        file.read("{}")
        "#,
        path_for_atlas(&src),
        path_for_atlas(&renamed),
        path_for_atlas(&renamed),
        path_for_atlas(&copied),
        path_for_atlas(&copied)
    );

    let mut security = SecurityContext::new();
    security.grant_filesystem_write(temp_dir.path(), true);
    security.grant_filesystem_read(temp_dir.path(), true);
    let runtime = Atlas::new_with_security(security);
    let result = runtime.eval(&code).unwrap();

    if let Value::Result(Ok(inner)) = result {
        assert_eq!(inner.to_string(), "test data");
    } else {
        assert_eq!(result.to_string(), "test data");
    }
}

// ============================================================================
// Async file operations (B40-P07)
// ============================================================================

#[test]
fn test_file_read_async_returns_future() {
    let temp_dir = TempDir::new().unwrap();
    let src = temp_dir.path().join("test.txt");
    fs::write(&src, "async content").unwrap();

    let code = format!(
        r#"
        let f = file.readAsync("{}");
        reflect.typeOf(f)
        "#,
        path_for_atlas(&src)
    );

    let mut security = SecurityContext::new();
    security.grant_filesystem_read(temp_dir.path(), true);
    let runtime = Atlas::new_with_security(security);
    let result = runtime.eval(&code).unwrap();

    assert_eq!(result.to_string(), "Future");
}

#[test]
fn test_file_write_async_returns_future() {
    let temp_dir = TempDir::new().unwrap();
    let dst = temp_dir.path().join("async_write.txt");

    let code = format!(
        r#"
        let f = file.writeAsync("{}", "data");
        reflect.typeOf(f)
        "#,
        path_for_atlas(&dst)
    );

    let mut security = SecurityContext::new();
    security.grant_filesystem_write(temp_dir.path(), true);
    let runtime = Atlas::new_with_security(security);
    let result = runtime.eval(&code).unwrap();

    assert_eq!(result.to_string(), "Future");
}

#[test]
fn test_file_append_async_returns_future() {
    let temp_dir = TempDir::new().unwrap();
    let dst = temp_dir.path().join("async_append.txt");
    fs::write(&dst, "start").unwrap();

    let code = format!(
        r#"
        let f = file.appendAsync("{}", " end");
        reflect.typeOf(f)
        "#,
        path_for_atlas(&dst)
    );

    let mut security = SecurityContext::new();
    security.grant_filesystem_write(temp_dir.path(), true);
    let runtime = Atlas::new_with_security(security);
    let result = runtime.eval(&code).unwrap();

    assert_eq!(result.to_string(), "Future");
}

#[test]
fn test_file_rename_async_returns_future() {
    let temp_dir = TempDir::new().unwrap();
    let src = temp_dir.path().join("to_rename.txt");
    let dst = temp_dir.path().join("renamed.txt");
    fs::write(&src, "data").unwrap();

    let code = format!(
        r#"
        let f = file.renameAsync("{}", "{}");
        reflect.typeOf(f)
        "#,
        path_for_atlas(&src),
        path_for_atlas(&dst)
    );

    let mut security = SecurityContext::new();
    security.grant_filesystem_write(temp_dir.path(), true);
    security.grant_filesystem_read(temp_dir.path(), true);
    let runtime = Atlas::new_with_security(security);
    let result = runtime.eval(&code).unwrap();

    assert_eq!(result.to_string(), "Future");
}

#[test]
fn test_file_copy_async_returns_future() {
    let temp_dir = TempDir::new().unwrap();
    let src = temp_dir.path().join("to_copy.txt");
    let dst = temp_dir.path().join("copied.txt");
    fs::write(&src, "data").unwrap();

    let code = format!(
        r#"
        let f = file.copyAsync("{}", "{}");
        reflect.typeOf(f)
        "#,
        path_for_atlas(&src),
        path_for_atlas(&dst)
    );

    let mut security = SecurityContext::new();
    security.grant_filesystem_write(temp_dir.path(), true);
    security.grant_filesystem_read(temp_dir.path(), true);
    let runtime = Atlas::new_with_security(security);
    let result = runtime.eval(&code).unwrap();

    assert_eq!(result.to_string(), "Future");
}

#[test]
fn test_bare_async_globals_removed() {
    // B40-P07: bare globals readFileAsync/writeFileAsync/appendFileAsync are removed
    let runtime = Atlas::new();

    // readFileAsync should not be defined
    let result = runtime.eval(r#"readFileAsync("test.txt")"#);
    assert!(
        result.is_err(),
        "readFileAsync bare global should be removed"
    );

    // writeFileAsync should not be defined
    let result = runtime.eval(r#"writeFileAsync("test.txt", "data")"#);
    assert!(
        result.is_err(),
        "writeFileAsync bare global should be removed"
    );

    // appendFileAsync should not be defined
    let result = runtime.eval(r#"appendFileAsync("test.txt", "data")"#);
    assert!(
        result.is_err(),
        "appendFileAsync bare global should be removed"
    );
}
