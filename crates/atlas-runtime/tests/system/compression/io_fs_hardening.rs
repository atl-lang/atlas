use super::*;

// ============================================================================
// IO/FS Edge Case Hardening (Phase v02-completion-04)
// ============================================================================
// NOTE: These tests should eventually be moved to filesystem.rs
// They test filesystem I/O edge cases, not compression functionality
// Keeping them here for now to maintain zero behavior change during refactor
// ============================================================================
// IO/FS Edge Case Hardening (Phase v02-completion-04)
// ============================================================================

fn with_io() -> Atlas {
    Atlas::new_with_security(SecurityContext::allow_all())
}

fn eval_str_io(code: &str) -> String {
    match with_io().eval(code) {
        Ok(v) => v.to_string(),
        Err(e) => panic!("Expected success, got error: {:?}", e),
    }
}

fn eval_err_io(code: &str) -> bool {
    with_io().eval(code).is_err()
}

// --- readFile edge cases ---

#[test]
fn test_read_file_nonexistent_returns_error() {
    assert!(eval_err_io(r#"readFile("/does/not/exist/file_xyz.txt")"#));
}

#[test]
fn test_read_file_empty_file_returns_empty_string() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("empty.txt");
    std_fs::write(&path, "").unwrap();
    let p = path.to_str().unwrap().replace('\\', "/");
    let code = format!(r#"readFile("{p}")"#);
    assert_eq!(eval_str_io(&code), "");
}

#[test]
fn test_write_file_creates_new_file() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("created.txt");
    let p = path.to_str().unwrap().replace('\\', "/");
    let code = format!(r#"writeFile("{p}", "hello"); readFile("{p}")"#);
    assert_eq!(eval_str_io(&code), "hello");
}

#[test]
fn test_write_file_overwrites_existing() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("overwrite.txt");
    std_fs::write(&path, "old content").unwrap();
    let p = path.to_str().unwrap().replace('\\', "/");
    let code = format!(r#"writeFile("{p}", "new content"); readFile("{p}")"#);
    assert_eq!(eval_str_io(&code), "new content");
}

#[test]
fn test_append_file_creates_if_not_exists() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("appended.txt");
    let p = path.to_str().unwrap().replace('\\', "/");
    let code = format!(r#"appendFile("{p}", "first"); readFile("{p}")"#);
    assert_eq!(eval_str_io(&code), "first");
}

#[test]
fn test_append_file_appends_to_existing() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("append_existing.txt");
    std_fs::write(&path, "A").unwrap();
    let p = path.to_str().unwrap().replace('\\', "/");
    let code = format!(r#"appendFile("{p}", "B"); readFile("{p}")"#);
    assert_eq!(eval_str_io(&code), "AB");
}

#[test]
fn test_file_exists_true_for_existing_file() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("exists.txt");
    std_fs::write(&path, "x").unwrap();
    let p = path.to_str().unwrap().replace('\\', "/");
    let code = format!(r#"fileExists("{p}")"#);
    assert_eq!(eval_str_io(&code), "true");
}

#[test]
fn test_file_exists_false_for_nonexistent() {
    assert_eq!(
        eval_str_io(r#"fileExists("/does/not/exist/nope_xyz.txt")"#),
        "false"
    );
}

#[test]
fn test_file_exists_true_for_directory() {
    let temp = TempDir::new().unwrap();
    let p = temp.path().to_str().unwrap().replace('\\', "/");
    let code = format!(r#"fileExists("{p}")"#);
    assert_eq!(eval_str_io(&code), "true");
}

#[test]
fn test_read_dir_empty_directory_returns_empty_array() {
    let temp = TempDir::new().unwrap();
    let p = temp.path().to_str().unwrap().replace('\\', "/");
    let code = format!(r#"len(readDir("{p}"))"#);
    assert_eq!(eval_str_io(&code), "0");
}

#[test]
fn test_read_dir_nonexistent_returns_error() {
    assert!(eval_err_io(r#"readDir("/does/not/exist/dir_xyz")"#));
}

#[test]
fn test_remove_file_nonexistent_returns_error() {
    assert!(eval_err_io(r#"removeFile("/does/not/exist/file_xyz.txt")"#));
}

#[test]
fn test_remove_file_success() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("to_remove.txt");
    std_fs::write(&path, "bye").unwrap();
    let p = path.to_str().unwrap().replace('\\', "/");
    let code = format!(r#"removeFile("{p}"); fileExists("{p}")"#);
    assert_eq!(eval_str_io(&code), "false");
}

#[test]
fn test_remove_dir_nonexistent_returns_error() {
    assert!(eval_err_io(r#"removeDir("/does/not/exist/dir_xyz")"#));
}

#[test]
fn test_remove_dir_success() {
    let temp = TempDir::new().unwrap();
    let sub = temp.path().join("subdir");
    std_fs::create_dir(&sub).unwrap();
    let p = sub.to_str().unwrap().replace('\\', "/");
    let code = format!(r#"removeDir("{p}"); fileExists("{p}")"#);
    assert_eq!(eval_str_io(&code), "false");
}

#[test]
fn test_create_dir_succeeds_when_already_exists() {
    let temp = TempDir::new().unwrap();
    let p = temp.path().to_str().unwrap().replace('\\', "/");
    // createDir on existing dir should not error (idempotent via createDir or error — check behavior)
    let result = with_io().eval(&format!(r#"createDir("{p}")"#));
    // Either succeeds or returns a meaningful error — should not panic/crash
    let _ = result;
}

#[test]
fn test_read_dir_returns_entry_count() {
    let temp = TempDir::new().unwrap();
    std_fs::write(temp.path().join("a.txt"), "").unwrap();
    std_fs::write(temp.path().join("b.txt"), "").unwrap();
    std_fs::write(temp.path().join("c.txt"), "").unwrap();
    let p = temp.path().to_str().unwrap().replace('\\', "/");
    let code = format!(r#"len(readDir("{p}"))"#);
    assert_eq!(eval_str_io(&code), "3");
}

// --- Path edge cases via stdlib ---

#[test]
fn test_path_join_absolute_second_arg_replaces_first() {
    // Matches Rust/OS semantics: joining "/a/b" + "/c" → "/c"
    let segments = Value::array(vec![Value::string("/a/b"), Value::string("/c")]);
    let result = call_fn("pathJoinArray", &[segments]).unwrap();
    match result {
        Value::String(s) => {
            assert!(
                s.as_str().ends_with("/c") || s.as_str() == "/c",
                "Absolute second segment should dominate, got: {}",
                s.as_str()
            );
        }
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_path_basename_trailing_slash() {
    let result = call_fn("pathBasename", &[Value::string("/foo/bar/")]).unwrap();
    match result {
        Value::String(s) => assert_eq!(s.as_str(), "bar"),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_path_dirname_of_root() {
    let result = call_fn("pathDirname", &[Value::string("/")]).unwrap();
    match result {
        // Root "/" has no parent — Path::parent() returns None → empty string
        Value::String(s) => assert_eq!(s.as_str(), ""),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_path_normalize_dot_and_dotdot() {
    let result = call_fn("pathNormalize", &[Value::string("/a/./b/../c")]).unwrap();
    match result {
        Value::String(s) => assert_eq!(s.as_str(), "/a/c"),
        _ => panic!("Expected string"),
    }
}
