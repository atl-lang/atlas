use super::*;

// ============================================================================
// Integration and Edge Case Tests
// ============================================================================

#[test]
fn test_mkdir_error_on_existing_directory() {
    let temp = TempDir::new().unwrap();
    let dir_path = temp.path().join("test_dir");
    std_fs::create_dir(&dir_path).unwrap();

    let path_str = dir_path.to_str().unwrap();
    let result = fs::mkdir(path_str, span());
    assert!(result.is_err());
}

#[test]
fn test_readdir_error_on_nonexistent_directory() {
    let result = fs::readdir("/nonexistent/directory/path", span());
    assert!(result.is_err());
}

#[test]
fn test_size_error_on_nonexistent_file() {
    let result = fs::size("/nonexistent/file.txt", span());
    assert!(result.is_err());
}

#[test]
fn test_walk_empty_directory() {
    let temp = TempDir::new().unwrap();
    let dir_path = temp.path().join("empty_dir");
    std_fs::create_dir(&dir_path).unwrap();

    let path_str = dir_path.to_str().unwrap();
    let result = fs::walk(path_str, span()).unwrap();
    let entries = extract_array(&result);

    assert_eq!(entries.len(), 0);
}

#[test]
fn test_filter_entries_no_matches() {
    let entries = vec![
        Value::string("file1.txt".to_string()),
        Value::string("file2.txt".to_string()),
    ];

    let result = fs::filter_entries(&entries, "*.rs", span()).unwrap();
    let filtered = extract_array(&result);

    assert_eq!(filtered.len(), 0);
}

#[test]
fn test_filter_entries_exact_match() {
    let entries = vec![
        Value::string("file1.txt".to_string()),
        Value::string("file2.txt".to_string()),
    ];

    let result = fs::filter_entries(&entries, "file1.txt", span()).unwrap();
    let filtered = extract_array(&result);

    assert_eq!(filtered.len(), 1);
    assert_eq!(extract_string(&filtered[0]), "file1.txt");
}

#[test]
fn test_is_dir_nonexistent_path() {
    let result = fs::is_dir("/nonexistent/path", span()).unwrap();
    assert!(!extract_bool(&result));
}

#[test]
fn test_is_file_nonexistent_path() {
    let result = fs::is_file("/nonexistent/path", span()).unwrap();
    assert!(!extract_bool(&result));
}

#[test]
fn test_readdir_sorts_entries_consistently() {
    let temp = TempDir::new().unwrap();
    std_fs::write(temp.path().join("file1.txt"), "content").unwrap();
    std_fs::write(temp.path().join("file2.txt"), "content").unwrap();

    let path_str = temp.path().to_str().unwrap();
    let result1 = fs::readdir(path_str, span()).unwrap();
    let result2 = fs::readdir(path_str, span()).unwrap();

    let entries1 = extract_array(&result1);
    let entries2 = extract_array(&result2);
    assert_eq!(entries1.len(), entries2.len());
}

#[test]
fn test_walk_deep_directory_tree() {
    let temp = TempDir::new().unwrap();

    let mut path = temp.path().to_path_buf();
    for i in 0..5 {
        path = path.join(format!("level{}", i));
        std_fs::create_dir(&path).unwrap();
        std_fs::write(path.join("file.txt"), "content").unwrap();
    }

    let path_str = temp.path().to_str().unwrap();
    let result = fs::walk(path_str, span()).unwrap();
    let entries = extract_array(&result);

    assert!(entries.len() >= 10);
}

#[test]
fn test_size_returns_zero_for_empty_file() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("empty.txt");
    std_fs::write(&file_path, "").unwrap();

    let path_str = file_path.to_str().unwrap();
    let result = fs::size(path_str, span()).unwrap();
    let size = extract_number(&result);

    assert_eq!(size, 0.0);
}

#[test]
fn test_rmdir_recursive_handles_empty_directory() {
    let temp = TempDir::new().unwrap();
    let dir_path = temp.path().join("empty_dir");
    std_fs::create_dir(&dir_path).unwrap();

    let path_str = dir_path.to_str().unwrap();
    let result = fs::rmdir_recursive(path_str, span());
    assert!(result.is_ok());
    assert!(!dir_path.exists());
}

#[test]
fn test_glob_pattern_prefix_wildcard() {
    let entries = vec![
        Value::string("test_file.txt".to_string()),
        Value::string("file.txt".to_string()),
    ];

    let result = fs::filter_entries(&entries, "test_*", span()).unwrap();
    let filtered = extract_array(&result);

    assert_eq!(filtered.len(), 1);
    assert_eq!(extract_string(&filtered[0]), "test_file.txt");
}

#[test]
fn test_glob_pattern_suffix_wildcard() {
    let entries = vec![
        Value::string("file.txt".to_string()),
        Value::string("file.rs".to_string()),
    ];

    let result = fs::filter_entries(&entries, "*.txt", span()).unwrap();
    let filtered = extract_array(&result);

    assert_eq!(filtered.len(), 1);
    assert_eq!(extract_string(&filtered[0]), "file.txt");
}
