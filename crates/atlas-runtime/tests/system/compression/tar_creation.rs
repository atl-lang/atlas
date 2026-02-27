use super::*;
use atlas_runtime::stdlib::compression::tar;

// ============================================================================
// Tar Creation Tests
// ============================================================================

#[test]
fn test_tar_create_single_file() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    std_fs::write(&test_file, "test content").unwrap();

    let tar_path = temp.path().join("archive.tar");

    let sources = str_array_value(&[test_file.to_str().unwrap()]);
    let output = str_value(tar_path.to_str().unwrap());

    let result = tar::tar_create(&sources, &output, span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(tar_path.exists());
}

#[test]
fn test_tar_create_directory() {
    let temp = TempDir::new().unwrap();
    let test_dir = create_test_dir(temp.path(), "testdir");
    create_test_file(&test_dir, "file1.txt", "content 1");
    create_test_file(&test_dir, "file2.txt", "content 2");

    let tar_path = temp.path().join("archive.tar");

    let sources = str_array_value(&[test_dir.to_str().unwrap()]);
    let output = str_value(tar_path.to_str().unwrap());

    let result = tar::tar_create(&sources, &output, span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(tar_path.exists());
}

#[test]
fn test_tar_create_multiple_sources() {
    let temp = TempDir::new().unwrap();
    let file1 = temp.path().join("file1.txt");
    let file2 = temp.path().join("file2.txt");
    std_fs::write(&file1, "content 1").unwrap();
    std_fs::write(&file2, "content 2").unwrap();

    let tar_path = temp.path().join("archive.tar");

    let sources = str_array_value(&[file1.to_str().unwrap(), file2.to_str().unwrap()]);
    let output = str_value(tar_path.to_str().unwrap());

    let result = tar::tar_create(&sources, &output, span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(tar_path.exists());
}

#[test]
fn test_tar_create_nonexistent_source() {
    let temp = TempDir::new().unwrap();
    let nonexistent = temp.path().join("nonexistent.txt");
    let tar_path = temp.path().join("archive.tar");

    let sources = str_array_value(&[nonexistent.to_str().unwrap()]);
    let output = str_value(tar_path.to_str().unwrap());

    let error = tar::tar_create(&sources, &output, span()).unwrap_err();
    let error_msg = format!("{:?}", error);
    assert!(error_msg.contains("does not exist"));
}

#[test]
fn test_tar_create_empty_sources() {
    let temp = TempDir::new().unwrap();
    let tar_path = temp.path().join("archive.tar");

    let sources = Value::array(vec![]);
    let output = str_value(tar_path.to_str().unwrap());

    let result = tar::tar_create(&sources, &output, span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(tar_path.exists());
}

#[test]
fn test_tar_create_nested_directories() {
    let temp = TempDir::new().unwrap();
    let test_dir = create_test_dir(temp.path(), "testdir");
    let sub_dir = create_test_dir(&test_dir, "subdir");
    create_test_file(&sub_dir, "nested.txt", "nested content");

    let tar_path = temp.path().join("archive.tar");

    let sources = str_array_value(&[test_dir.to_str().unwrap()]);
    let output = str_value(tar_path.to_str().unwrap());

    let result = tar::tar_create(&sources, &output, span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(tar_path.exists());
}

#[test]
fn test_tar_create_preserves_metadata() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    std_fs::write(&test_file, "test content").unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std_fs::metadata(&test_file).unwrap().permissions();
        perms.set_mode(0o644);
        std_fs::set_permissions(&test_file, perms).unwrap();
    }

    let tar_path = temp.path().join("archive.tar");

    let sources = str_array_value(&[test_file.to_str().unwrap()]);
    let output = str_value(tar_path.to_str().unwrap());

    let result = tar::tar_create(&sources, &output, span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(tar_path.exists());
}

#[test]
fn test_tar_create_invalid_sources_type() {
    let temp = TempDir::new().unwrap();
    let tar_path = temp.path().join("archive.tar");

    // Pass string instead of array
    let sources = str_value("/some/path");
    let output = str_value(tar_path.to_str().unwrap());

    let error = tar::tar_create(&sources, &output, span()).unwrap_err();
    let error_msg = format!("{:?}", error);
    assert!(error_msg.contains("array"));
}

// ============================================================================
// Tar.gz Creation Tests
// ============================================================================

#[test]
fn test_tar_create_gz_default_level() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    std_fs::write(&test_file, "test content that will be compressed").unwrap();

    let tar_gz_path = temp.path().join("archive.tar.gz");

    let sources = str_array_value(&[test_file.to_str().unwrap()]);
    let output = str_value(tar_gz_path.to_str().unwrap());

    let result = tar::tar_create_gz(&sources, &output, None, span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(tar_gz_path.exists());
}

#[test]
fn test_tar_create_gz_level_zero() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    std_fs::write(&test_file, "test content").unwrap();

    let tar_gz_path = temp.path().join("archive.tar.gz");

    let sources = str_array_value(&[test_file.to_str().unwrap()]);
    let output = str_value(tar_gz_path.to_str().unwrap());
    let level = Value::Number(0.0);

    let result = tar::tar_create_gz(&sources, &output, Some(&level), span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(tar_gz_path.exists());
}

#[test]
fn test_tar_create_gz_max_level() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    std_fs::write(&test_file, "test content").unwrap();

    let tar_gz_path = temp.path().join("archive.tar.gz");

    let sources = str_array_value(&[test_file.to_str().unwrap()]);
    let output = str_value(tar_gz_path.to_str().unwrap());
    let level = Value::Number(9.0);

    let result = tar::tar_create_gz(&sources, &output, Some(&level), span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(tar_gz_path.exists());
}

#[test]
fn test_tar_create_gz_invalid_level() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    std_fs::write(&test_file, "test content").unwrap();

    let tar_gz_path = temp.path().join("archive.tar.gz");

    let sources = str_array_value(&[test_file.to_str().unwrap()]);
    let output = str_value(tar_gz_path.to_str().unwrap());
    let level = Value::Number(10.0);

    let error = tar::tar_create_gz(&sources, &output, Some(&level), span()).unwrap_err();
    let error_msg = format!("{:?}", error);
    assert!(error_msg.contains("0-9"));
}

#[test]
fn test_tar_create_gz_large_directory() {
    let temp = TempDir::new().unwrap();
    let test_dir = create_test_dir(temp.path(), "testdir");

    // Create multiple files
    for i in 0..10 {
        create_test_file(
            &test_dir,
            &format!("file{}.txt", i),
            &format!("content {}", i),
        );
    }

    let tar_gz_path = temp.path().join("archive.tar.gz");

    let sources = str_array_value(&[test_dir.to_str().unwrap()]);
    let output = str_value(tar_gz_path.to_str().unwrap());

    let result = tar::tar_create_gz(&sources, &output, None, span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(tar_gz_path.exists());
}
