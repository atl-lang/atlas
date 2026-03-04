use super::*;

// ============================================================================
// File Metadata Tests
// ============================================================================

#[test]
fn test_size_returns_file_size() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.txt");
    std_fs::write(&file_path, "hello").unwrap();

    let path_str = file_path.to_str().unwrap();
    let result = fs::size(path_str, span()).unwrap();
    let size = extract_number(&result);

    assert_eq!(size, 5.0);
}

#[test]
fn test_mtime_returns_modified_time() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.txt");
    std_fs::write(&file_path, "content").unwrap();

    let path_str = file_path.to_str().unwrap();
    let result = fs::mtime(path_str, span()).unwrap();
    let mtime = extract_number(&result);

    assert!(mtime > 0.0);
}

#[test]
fn test_ctime_returns_created_time() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.txt");
    std_fs::write(&file_path, "content").unwrap();

    let path_str = file_path.to_str().unwrap();
    let result = fs::ctime(path_str, span()).unwrap();
    let ctime = extract_number(&result);

    assert!(ctime > 0.0);
}

#[test]
fn test_atime_returns_access_time() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.txt");
    std_fs::write(&file_path, "content").unwrap();

    let path_str = file_path.to_str().unwrap();
    let result = fs::atime(path_str, span()).unwrap();
    let atime = extract_number(&result);

    assert!(atime > 0.0);
}

#[test]
fn test_permissions_returns_file_permissions() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.txt");
    std_fs::write(&file_path, "content").unwrap();

    let path_str = file_path.to_str().unwrap();
    let result = fs::permissions(path_str, span()).unwrap();
    let perms = extract_number(&result);

    assert!(perms > 0.0);
}

#[test]
fn test_is_dir_detects_directory() {
    let temp = TempDir::new().unwrap();
    let dir_path = temp.path().join("test_dir");
    std_fs::create_dir(&dir_path).unwrap();

    let path_str = dir_path.to_str().unwrap();
    let result = fs::is_dir(path_str, span()).unwrap();
    assert!(extract_bool(&result));
}

#[test]
fn test_is_dir_returns_false_for_file() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.txt");
    std_fs::write(&file_path, "content").unwrap();

    let path_str = file_path.to_str().unwrap();
    let result = fs::is_dir(path_str, span()).unwrap();
    assert!(!extract_bool(&result));
}

#[test]
fn test_is_file_detects_file() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.txt");
    std_fs::write(&file_path, "content").unwrap();

    let path_str = file_path.to_str().unwrap();
    let result = fs::is_file(path_str, span()).unwrap();
    assert!(extract_bool(&result));
}

#[test]
fn test_is_file_returns_false_for_directory() {
    let temp = TempDir::new().unwrap();
    let dir_path = temp.path().join("test_dir");
    std_fs::create_dir(&dir_path).unwrap();

    let path_str = dir_path.to_str().unwrap();
    let result = fs::is_file(path_str, span()).unwrap();
    assert!(!extract_bool(&result));
}

#[test]
#[cfg(unix)]
fn test_is_symlink_detects_symlink() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.txt");
    let link_path = temp.path().join("link.txt");
    std_fs::write(&file_path, "content").unwrap();

    std::os::unix::fs::symlink(&file_path, &link_path).unwrap();

    let path_str = link_path.to_str().unwrap();
    let result = fs::is_symlink(path_str, span()).unwrap();
    assert!(extract_bool(&result));
}

#[test]
fn test_permissions_on_directory() {
    let temp = TempDir::new().unwrap();
    let dir_path = temp.path().join("test_dir");
    std_fs::create_dir(&dir_path).unwrap();

    let path_str = dir_path.to_str().unwrap();
    let result = fs::permissions(path_str, span()).unwrap();
    let perms = extract_number(&result);

    assert!(perms > 0.0);
}
