use super::*;

// ============================================================================
// Directory Operations Tests
// ============================================================================

#[test]
fn test_mkdir_creates_directory() {
    let temp = TempDir::new().unwrap();
    let dir_path = temp.path().join("test_dir");
    let path_str = dir_path.to_str().unwrap();

    let result = fs::mkdir(path_str, span());
    assert!(result.is_ok());
    assert!(dir_path.exists());
    assert!(dir_path.is_dir());
}

#[test]
fn test_mkdir_fails_if_parent_missing() {
    let temp = TempDir::new().unwrap();
    let dir_path = temp.path().join("missing_parent").join("test_dir");
    let path_str = dir_path.to_str().unwrap();

    let result = fs::mkdir(path_str, span());
    assert!(result.is_err());
}

#[test]
fn test_mkdirp_creates_directory_recursively() {
    let temp = TempDir::new().unwrap();
    let dir_path = temp.path().join("a").join("b").join("c");
    let path_str = dir_path.to_str().unwrap();

    let result = fs::mkdirp(path_str, span());
    assert!(result.is_ok());
    assert!(dir_path.exists());
    assert!(dir_path.is_dir());
}

#[test]
fn test_mkdirp_succeeds_if_directory_exists() {
    let temp = TempDir::new().unwrap();
    let dir_path = temp.path().join("test_dir");
    std_fs::create_dir(&dir_path).unwrap();

    let path_str = dir_path.to_str().unwrap();
    let result = fs::mkdirp(path_str, span());
    assert!(result.is_ok());
}

#[test]
fn test_rmdir_removes_empty_directory() {
    let temp = TempDir::new().unwrap();
    let dir_path = temp.path().join("test_dir");
    std_fs::create_dir(&dir_path).unwrap();

    let path_str = dir_path.to_str().unwrap();
    let result = fs::rmdir(path_str, span());
    assert!(result.is_ok());
    assert!(!dir_path.exists());
}

#[test]
fn test_rmdir_fails_if_directory_not_empty() {
    let temp = TempDir::new().unwrap();
    let dir_path = temp.path().join("test_dir");
    std_fs::create_dir(&dir_path).unwrap();
    std_fs::write(dir_path.join("file.txt"), "content").unwrap();

    let path_str = dir_path.to_str().unwrap();
    let result = fs::rmdir(path_str, span());
    assert!(result.is_err());
}

#[test]
fn test_rmdir_recursive_removes_directory_with_contents() {
    let temp = TempDir::new().unwrap();
    let dir_path = temp.path().join("test_dir");
    std_fs::create_dir(&dir_path).unwrap();
    std_fs::write(dir_path.join("file.txt"), "content").unwrap();
    std_fs::create_dir(dir_path.join("subdir")).unwrap();

    let path_str = dir_path.to_str().unwrap();
    let result = fs::rmdir_recursive(path_str, span());
    assert!(result.is_ok());
    assert!(!dir_path.exists());
}

#[test]
fn test_readdir_lists_directory_contents() {
    let temp = TempDir::new().unwrap();
    std_fs::write(temp.path().join("file1.txt"), "content").unwrap();
    std_fs::write(temp.path().join("file2.txt"), "content").unwrap();
    std_fs::create_dir(temp.path().join("subdir")).unwrap();

    let path_str = temp.path().to_str().unwrap();
    let result = fs::readdir(path_str, span()).unwrap();
    let entries = extract_array(&result);

    assert_eq!(entries.len(), 3);
    let names: Vec<String> = entries.iter().map(extract_string).collect();
    assert!(names.contains(&"file1.txt".to_string()));
    assert!(names.contains(&"file2.txt".to_string()));
    assert!(names.contains(&"subdir".to_string()));
}

#[test]
fn test_walk_traverses_directory_tree() {
    let temp = TempDir::new().unwrap();
    std_fs::write(temp.path().join("root.txt"), "content").unwrap();
    std_fs::create_dir(temp.path().join("dir1")).unwrap();
    std_fs::write(temp.path().join("dir1").join("file1.txt"), "content").unwrap();
    std_fs::create_dir(temp.path().join("dir1").join("subdir")).unwrap();
    std_fs::write(
        temp.path().join("dir1").join("subdir").join("file2.txt"),
        "content",
    )
    .unwrap();

    let path_str = temp.path().to_str().unwrap();
    let result = fs::walk(path_str, span()).unwrap();
    let entries = extract_array(&result);

    assert!(entries.len() >= 5);
}

#[test]
fn test_filter_entries_with_wildcard() {
    let entries = vec![
        Value::string("file1.txt".to_string()),
        Value::string("file2.rs".to_string()),
        Value::string("test.txt".to_string()),
        Value::string("readme.md".to_string()),
    ];

    let result = fs::filter_entries(&entries, "*.txt", span()).unwrap();
    let filtered = extract_array(&result);

    assert_eq!(filtered.len(), 2);
    let names: Vec<String> = filtered.iter().map(extract_string).collect();
    assert!(names.contains(&"file1.txt".to_string()));
    assert!(names.contains(&"test.txt".to_string()));
}

#[test]
fn test_sort_entries_alphabetically() {
    let entries = vec![
        Value::string("zebra.txt".to_string()),
        Value::string("apple.txt".to_string()),
        Value::string("Banana.txt".to_string()),
    ];

    let result = fs::sort_entries(&entries, span()).unwrap();
    let sorted = extract_array(&result);

    assert_eq!(sorted.len(), 3);
    assert_eq!(extract_string(&sorted[0]), "apple.txt");
    assert_eq!(extract_string(&sorted[1]), "Banana.txt");
    assert_eq!(extract_string(&sorted[2]), "zebra.txt");
}
