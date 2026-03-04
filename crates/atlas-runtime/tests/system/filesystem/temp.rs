use super::*;

// ============================================================================
// Temporary Files Tests
// ============================================================================

#[test]
fn test_tmpfile_creates_temporary_file() {
    let result = fs::tmpfile(span()).unwrap();
    let path = extract_string(&result);

    assert!(!path.is_empty());
    assert!(Path::new(&path).exists());
    assert!(Path::new(&path).is_file());

    std_fs::remove_file(&path).ok();
}

#[test]
fn test_tmpdir_creates_temporary_directory() {
    let result = fs::tmpdir(span()).unwrap();
    let path = extract_string(&result);

    assert!(!path.is_empty());
    assert!(Path::new(&path).exists());
    assert!(Path::new(&path).is_dir());

    std_fs::remove_dir(&path).ok();
}

#[test]
fn test_tmpfile_named_creates_file_with_prefix() {
    let result = fs::tmpfile_named("atlas_test", span()).unwrap();
    let path = extract_string(&result);

    assert!(!path.is_empty());
    assert!(path.contains("atlas_test"));
    assert!(Path::new(&path).exists());

    std_fs::remove_file(&path).ok();
}

#[test]
fn test_get_temp_dir_returns_system_temp_directory() {
    let result = fs::get_temp_dir(span()).unwrap();
    let path = extract_string(&result);

    assert!(!path.is_empty());
    assert!(Path::new(&path).exists());
    assert!(Path::new(&path).is_dir());
}

#[test]
fn test_tmpfile_creates_unique_files() {
    let result1 = fs::tmpfile(span()).unwrap();
    let result2 = fs::tmpfile(span()).unwrap();

    let path1 = extract_string(&result1);
    let path2 = extract_string(&result2);

    assert_ne!(path1, path2);

    std_fs::remove_file(&path1).ok();
    std_fs::remove_file(&path2).ok();
}

#[test]
fn test_tmpdir_creates_unique_directories() {
    let result1 = fs::tmpdir(span()).unwrap();
    let result2 = fs::tmpdir(span()).unwrap();

    let path1 = extract_string(&result1);
    let path2 = extract_string(&result2);

    assert_ne!(path1, path2);

    std_fs::remove_dir(&path1).ok();
    std_fs::remove_dir(&path2).ok();
}
