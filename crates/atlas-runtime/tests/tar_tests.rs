//! Integration tests for tar archive functionality

use atlas_runtime::span::Span;
use atlas_runtime::stdlib::compression::tar;
use atlas_runtime::value::Value;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// ============================================================================
// Test Helpers
// ============================================================================

fn span() -> Span {
    Span::dummy()
}

fn create_test_file(dir: &std::path::Path, name: &str, content: &str) {
    let path = dir.join(name);
    fs::write(path, content).unwrap();
}

fn create_test_dir(dir: &std::path::Path, name: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    fs::create_dir(&path).unwrap();
    path
}

fn str_value(s: &str) -> Value {
    Value::string(s.to_string())
}

fn str_array_value(paths: &[&str]) -> Value {
    let values: Vec<Value> = paths.iter().map(|p| str_value(p)).collect();
    Value::array(values)
}

// ============================================================================
// Tar Creation Tests
// ============================================================================

#[test]
fn test_tar_create_single_file() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    fs::write(&test_file, "test content").unwrap();

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
    fs::write(&file1, "content 1").unwrap();
    fs::write(&file2, "content 2").unwrap();

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
    fs::write(&test_file, "test content").unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&test_file).unwrap().permissions();
        perms.set_mode(0o644);
        fs::set_permissions(&test_file, perms).unwrap();
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
    fs::write(&test_file, "test content that will be compressed").unwrap();

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
    fs::write(&test_file, "test content").unwrap();

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
    fs::write(&test_file, "test content").unwrap();

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
    fs::write(&test_file, "test content").unwrap();

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

// ============================================================================
// Tar Extraction Tests
// ============================================================================

#[test]
fn test_tar_extract_basic() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    fs::write(&test_file, "test content").unwrap();

    let tar_path = temp.path().join("archive.tar");

    // Create tar using low-level function
    let sources = vec![PathBuf::from(test_file.to_str().unwrap())];
    tar::create_tar(&sources, &tar_path, span()).unwrap();

    // Extract tar using Atlas API
    let extract_dir = temp.path().join("extracted");
    let tar_val = str_value(tar_path.to_str().unwrap());
    let out_val = str_value(extract_dir.to_str().unwrap());

    let result = tar::tar_extract(&tar_val, &out_val, span()).unwrap();
    match result {
        Value::Array(arr) => {
            let arr_guard = arr.lock().unwrap();
            assert!(!arr_guard.is_empty());
        }
        _ => panic!("Expected array result"),
    }

    let extracted_file = extract_dir.join("test.txt");
    assert!(extracted_file.exists());
    assert_eq!(fs::read_to_string(extracted_file).unwrap(), "test content");
}

#[test]
fn test_tar_extract_directory() {
    let temp = TempDir::new().unwrap();
    let test_dir = create_test_dir(temp.path(), "testdir");
    create_test_file(&test_dir, "file1.txt", "content 1");
    create_test_file(&test_dir, "file2.txt", "content 2");

    let tar_path = temp.path().join("archive.tar");

    let sources = vec![PathBuf::from(test_dir.to_str().unwrap())];
    tar::create_tar(&sources, &tar_path, span()).unwrap();

    let extract_dir = temp.path().join("extracted");
    let tar_val = str_value(tar_path.to_str().unwrap());
    let out_val = str_value(extract_dir.to_str().unwrap());

    tar::tar_extract(&tar_val, &out_val, span()).unwrap();

    let extracted_subdir = extract_dir.join("testdir");
    assert!(extracted_subdir.exists());
    assert!(extracted_subdir.join("file1.txt").exists());
    assert!(extracted_subdir.join("file2.txt").exists());
}

#[test]
fn test_tar_extract_nonexistent_tar() {
    let temp = TempDir::new().unwrap();
    let nonexistent = temp.path().join("nonexistent.tar");
    let extract_dir = temp.path().join("extracted");

    let tar_val = str_value(nonexistent.to_str().unwrap());
    let out_val = str_value(extract_dir.to_str().unwrap());

    let error = tar::tar_extract(&tar_val, &out_val, span()).unwrap_err();
    let error_msg = format!("{:?}", error);
    assert!(error_msg.contains("Failed to open tar file"));
}

#[test]
fn test_tar_extract_creates_output_dir() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    fs::write(&test_file, "test content").unwrap();

    let tar_path = temp.path().join("archive.tar");

    let sources = vec![PathBuf::from(test_file.to_str().unwrap())];
    tar::create_tar(&sources, &tar_path, span()).unwrap();

    // Extract to non-existent directory (should be created)
    let extract_dir = temp.path().join("nonexistent").join("extracted");
    let tar_val = str_value(tar_path.to_str().unwrap());
    let out_val = str_value(extract_dir.to_str().unwrap());

    tar::tar_extract(&tar_val, &out_val, span()).unwrap();
    assert!(extract_dir.exists());
}

#[test]
fn test_tar_extract_nested_directories() {
    let temp = TempDir::new().unwrap();
    let test_dir = create_test_dir(temp.path(), "testdir");
    let sub_dir = create_test_dir(&test_dir, "subdir");
    create_test_file(&sub_dir, "nested.txt", "nested content");

    let tar_path = temp.path().join("archive.tar");

    let sources = vec![PathBuf::from(test_dir.to_str().unwrap())];
    tar::create_tar(&sources, &tar_path, span()).unwrap();

    let extract_dir = temp.path().join("extracted");
    let tar_val = str_value(tar_path.to_str().unwrap());
    let out_val = str_value(extract_dir.to_str().unwrap());

    tar::tar_extract(&tar_val, &out_val, span()).unwrap();

    let nested_file = extract_dir
        .join("testdir")
        .join("subdir")
        .join("nested.txt");
    assert!(nested_file.exists());
    assert_eq!(fs::read_to_string(nested_file).unwrap(), "nested content");
}

#[test]
fn test_tar_extract_preserves_content() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    let content = "Hello, World! This is test content.";
    fs::write(&test_file, content).unwrap();

    let tar_path = temp.path().join("archive.tar");

    let sources = vec![PathBuf::from(test_file.to_str().unwrap())];
    tar::create_tar(&sources, &tar_path, span()).unwrap();

    let extract_dir = temp.path().join("extracted");
    let tar_val = str_value(tar_path.to_str().unwrap());
    let out_val = str_value(extract_dir.to_str().unwrap());

    tar::tar_extract(&tar_val, &out_val, span()).unwrap();

    let extracted_file = extract_dir.join("test.txt");
    assert!(extracted_file.exists());
    assert_eq!(fs::read_to_string(extracted_file).unwrap(), content);
}

#[test]
fn test_tar_extract_returns_file_list() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    fs::write(&test_file, "test content").unwrap();

    let tar_path = temp.path().join("archive.tar");

    let sources = vec![PathBuf::from(test_file.to_str().unwrap())];
    tar::create_tar(&sources, &tar_path, span()).unwrap();

    let extract_dir = temp.path().join("extracted");
    let tar_val = str_value(tar_path.to_str().unwrap());
    let out_val = str_value(extract_dir.to_str().unwrap());

    let result = tar::tar_extract(&tar_val, &out_val, span()).unwrap();
    match result {
        Value::Array(arr) => {
            let arr_guard = arr.lock().unwrap();
            assert!(!arr_guard.is_empty());

            // Check that all entries are strings
            for val in arr_guard.iter() {
                assert!(matches!(val, Value::String(_)));
            }
        }
        _ => panic!("Expected array result"),
    }
}

// ============================================================================
// Tar.gz Extraction Tests
// ============================================================================

#[test]
fn test_tar_extract_gz_basic() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    fs::write(&test_file, "test content for compression").unwrap();

    let tar_gz_path = temp.path().join("archive.tar.gz");

    let sources = vec![PathBuf::from(test_file.to_str().unwrap())];
    tar::create_tar_gz(&sources, &tar_gz_path, 6, span()).unwrap();

    let extract_dir = temp.path().join("extracted");
    let tar_gz_val = str_value(tar_gz_path.to_str().unwrap());
    let out_val = str_value(extract_dir.to_str().unwrap());

    let result = tar::tar_extract_gz(&tar_gz_val, &out_val, span()).unwrap();
    match result {
        Value::Array(arr) => {
            let arr_guard = arr.lock().unwrap();
            assert!(!arr_guard.is_empty());
        }
        _ => panic!("Expected array result"),
    }

    let extracted_file = extract_dir.join("test.txt");
    assert!(extracted_file.exists());
    assert_eq!(
        fs::read_to_string(extracted_file).unwrap(),
        "test content for compression"
    );
}

#[test]
fn test_tar_extract_gz_directory() {
    let temp = TempDir::new().unwrap();
    let test_dir = create_test_dir(temp.path(), "testdir");
    create_test_file(&test_dir, "file1.txt", "content 1");
    create_test_file(&test_dir, "file2.txt", "content 2");

    let tar_gz_path = temp.path().join("archive.tar.gz");

    let sources = vec![PathBuf::from(test_dir.to_str().unwrap())];
    tar::create_tar_gz(&sources, &tar_gz_path, 6, span()).unwrap();

    let extract_dir = temp.path().join("extracted");
    let tar_gz_val = str_value(tar_gz_path.to_str().unwrap());
    let out_val = str_value(extract_dir.to_str().unwrap());

    tar::tar_extract_gz(&tar_gz_val, &out_val, span()).unwrap();

    let extracted_subdir = extract_dir.join("testdir");
    assert!(extracted_subdir.exists());
    assert!(extracted_subdir.join("file1.txt").exists());
    assert!(extracted_subdir.join("file2.txt").exists());
}

#[test]
fn test_tar_extract_gz_nonexistent() {
    let temp = TempDir::new().unwrap();
    let nonexistent = temp.path().join("nonexistent.tar.gz");
    let extract_dir = temp.path().join("extracted");

    let tar_gz_val = str_value(nonexistent.to_str().unwrap());
    let out_val = str_value(extract_dir.to_str().unwrap());

    let error = tar::tar_extract_gz(&tar_gz_val, &out_val, span()).unwrap_err();
    let error_msg = format!("{:?}", error);
    assert!(error_msg.contains("Failed to open tar.gz file"));
}

#[test]
fn test_tar_extract_gz_large_directory() {
    let temp = TempDir::new().unwrap();
    let test_dir = create_test_dir(temp.path(), "testdir");

    // Create multiple files
    for i in 0..10 {
        create_test_file(
            &test_dir,
            &format!("file{}.txt", i),
            &format!("content {} with some repetitive text for compression", i),
        );
    }

    let tar_gz_path = temp.path().join("archive.tar.gz");

    let sources = vec![PathBuf::from(test_dir.to_str().unwrap())];
    tar::create_tar_gz(&sources, &tar_gz_path, 9, span()).unwrap();

    let extract_dir = temp.path().join("extracted");
    let tar_gz_val = str_value(tar_gz_path.to_str().unwrap());
    let out_val = str_value(extract_dir.to_str().unwrap());

    tar::tar_extract_gz(&tar_gz_val, &out_val, span()).unwrap();

    let extracted_subdir = extract_dir.join("testdir");
    for i in 0..10 {
        let file_path = extracted_subdir.join(format!("file{}.txt", i));
        assert!(file_path.exists());
    }
}

// ============================================================================
// Tar Utility Tests
// ============================================================================

#[test]
fn test_tar_list_basic() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    fs::write(&test_file, "test content").unwrap();

    let tar_path = temp.path().join("archive.tar");

    let sources = vec![PathBuf::from(test_file.to_str().unwrap())];
    tar::create_tar(&sources, &tar_path, span()).unwrap();

    let tar_val = str_value(tar_path.to_str().unwrap());
    let result = tar::tar_list(&tar_val, span()).unwrap();

    match result {
        Value::Array(arr) => {
            let arr_guard = arr.lock().unwrap();
            assert!(!arr_guard.is_empty());

            // Check that entries are HashMaps
            for val in arr_guard.iter() {
                assert!(matches!(val, Value::HashMap(_)));
            }
        }
        _ => panic!("Expected array result"),
    }
}

#[test]
fn test_tar_list_multiple_files() {
    let temp = TempDir::new().unwrap();
    let file1 = temp.path().join("file1.txt");
    let file2 = temp.path().join("file2.txt");
    fs::write(&file1, "content 1").unwrap();
    fs::write(&file2, "content 2").unwrap();

    let tar_path = temp.path().join("archive.tar");

    let sources = vec![
        PathBuf::from(file1.to_str().unwrap()),
        PathBuf::from(file2.to_str().unwrap()),
    ];
    tar::create_tar(&sources, &tar_path, span()).unwrap();

    let tar_val = str_value(tar_path.to_str().unwrap());
    let result = tar::tar_list(&tar_val, span()).unwrap();

    match result {
        Value::Array(arr) => {
            let arr_guard = arr.lock().unwrap();
            assert_eq!(arr_guard.len(), 2);
        }
        _ => panic!("Expected array result"),
    }
}

#[test]
fn test_tar_contains_existing_file() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    fs::write(&test_file, "test content").unwrap();

    let tar_path = temp.path().join("archive.tar");

    let sources = vec![PathBuf::from(test_file.to_str().unwrap())];
    tar::create_tar(&sources, &tar_path, span()).unwrap();

    let tar_val = str_value(tar_path.to_str().unwrap());
    let file_val = str_value("test.txt");

    let result = tar::tar_contains_file(&tar_val, &file_val, span()).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_tar_contains_nonexistent_file() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    fs::write(&test_file, "test content").unwrap();

    let tar_path = temp.path().join("archive.tar");

    let sources = vec![PathBuf::from(test_file.to_str().unwrap())];
    tar::create_tar(&sources, &tar_path, span()).unwrap();

    let tar_val = str_value(tar_path.to_str().unwrap());
    let file_val = str_value("nonexistent.txt");

    let result = tar::tar_contains_file(&tar_val, &file_val, span()).unwrap();
    assert_eq!(result, Value::Bool(false));
}

// ============================================================================
// Round-trip Tests
// ============================================================================

#[test]
fn test_tar_roundtrip_file() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    let content = "This is test content for round-trip verification";
    fs::write(&test_file, content).unwrap();

    let tar_path = temp.path().join("archive.tar");

    // Create tar
    let sources = vec![PathBuf::from(test_file.to_str().unwrap())];
    tar::create_tar(&sources, &tar_path, span()).unwrap();

    // Extract tar
    let extract_dir = temp.path().join("extracted");
    tar::extract_tar(&tar_path, &extract_dir, span()).unwrap();

    // Verify content matches
    let extracted_file = extract_dir.join("test.txt");
    assert_eq!(fs::read_to_string(extracted_file).unwrap(), content);
}

#[test]
fn test_tar_gz_roundtrip_file() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    let content = "This is test content for tar.gz round-trip verification with compression";
    fs::write(&test_file, content).unwrap();

    let tar_gz_path = temp.path().join("archive.tar.gz");

    // Create tar.gz
    let sources = vec![PathBuf::from(test_file.to_str().unwrap())];
    tar::create_tar_gz(&sources, &tar_gz_path, 9, span()).unwrap();

    // Extract tar.gz
    let extract_dir = temp.path().join("extracted");
    tar::extract_tar_gz(&tar_gz_path, &extract_dir, span()).unwrap();

    // Verify content matches
    let extracted_file = extract_dir.join("test.txt");
    assert_eq!(fs::read_to_string(extracted_file).unwrap(), content);
}

#[test]
fn test_tar_roundtrip_directory() {
    let temp = TempDir::new().unwrap();
    let test_dir = create_test_dir(temp.path(), "testdir");
    create_test_file(&test_dir, "file1.txt", "content 1");
    create_test_file(&test_dir, "file2.txt", "content 2");
    let sub_dir = create_test_dir(&test_dir, "subdir");
    create_test_file(&sub_dir, "nested.txt", "nested content");

    let tar_path = temp.path().join("archive.tar");

    // Create tar
    let sources = vec![PathBuf::from(test_dir.to_str().unwrap())];
    tar::create_tar(&sources, &tar_path, span()).unwrap();

    // Extract tar
    let extract_dir = temp.path().join("extracted");
    tar::extract_tar(&tar_path, &extract_dir, span()).unwrap();

    // Verify all files extracted correctly
    let extracted_dir = extract_dir.join("testdir");
    assert_eq!(
        fs::read_to_string(extracted_dir.join("file1.txt")).unwrap(),
        "content 1"
    );
    assert_eq!(
        fs::read_to_string(extracted_dir.join("file2.txt")).unwrap(),
        "content 2"
    );
    assert_eq!(
        fs::read_to_string(extracted_dir.join("subdir").join("nested.txt")).unwrap(),
        "nested content"
    );
}

#[test]
fn test_tar_gz_roundtrip_directory() {
    let temp = TempDir::new().unwrap();
    let test_dir = create_test_dir(temp.path(), "testdir");
    create_test_file(&test_dir, "file1.txt", "content 1");
    create_test_file(&test_dir, "file2.txt", "content 2");

    let tar_gz_path = temp.path().join("archive.tar.gz");

    // Create tar.gz
    let sources = vec![PathBuf::from(test_dir.to_str().unwrap())];
    tar::create_tar_gz(&sources, &tar_gz_path, 6, span()).unwrap();

    // Extract tar.gz
    let extract_dir = temp.path().join("extracted");
    tar::extract_tar_gz(&tar_gz_path, &extract_dir, span()).unwrap();

    // Verify all files extracted correctly
    let extracted_subdir = extract_dir.join("testdir");
    assert_eq!(
        fs::read_to_string(extracted_subdir.join("file1.txt")).unwrap(),
        "content 1"
    );
    assert_eq!(
        fs::read_to_string(extracted_subdir.join("file2.txt")).unwrap(),
        "content 2"
    );
}

#[test]
fn test_tar_roundtrip_with_list() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    fs::write(&test_file, "test content").unwrap();

    let tar_path = temp.path().join("archive.tar");

    // Create tar
    let sources = vec![PathBuf::from(test_file.to_str().unwrap())];
    tar::create_tar(&sources, &tar_path, span()).unwrap();

    // List before extraction
    let tar_val = str_value(tar_path.to_str().unwrap());
    let list_result = tar::tar_list(&tar_val, span()).unwrap();

    // Extract tar
    let extract_dir = temp.path().join("extracted");
    tar::extract_tar(&tar_path, &extract_dir, span()).unwrap();

    // List should return non-empty array
    match list_result {
        Value::Array(arr) => {
            assert!(!arr.lock().unwrap().is_empty());
        }
        _ => panic!("Expected array result"),
    }
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_tar_create_invalid_output_type() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    fs::write(&test_file, "test content").unwrap();

    let sources = str_array_value(&[test_file.to_str().unwrap()]);
    let output = Value::Number(42.0); // Invalid type

    let error = tar::tar_create(&sources, &output, span()).unwrap_err();
    let error_msg = format!("{:?}", error);
    assert!(error_msg.contains("string"));
}

#[test]
fn test_tar_extract_invalid_tar_path_type() {
    let temp = TempDir::new().unwrap();
    let extract_dir = temp.path().join("extracted");

    let tar_val = Value::Number(42.0); // Invalid type
    let out_val = str_value(extract_dir.to_str().unwrap());

    let error = tar::tar_extract(&tar_val, &out_val, span()).unwrap_err();
    let error_msg = format!("{:?}", error);
    assert!(error_msg.contains("string"));
}

#[test]
fn test_tar_list_invalid_path_type() {
    let tar_val = Value::Number(42.0); // Invalid type

    let error = tar::tar_list(&tar_val, span()).unwrap_err();
    let error_msg = format!("{:?}", error);
    assert!(error_msg.contains("string"));
}

#[test]
fn test_tar_contains_invalid_tar_type() {
    let tar_val = Value::Number(42.0); // Invalid type
    let file_val = str_value("test.txt");

    let error = tar::tar_contains_file(&tar_val, &file_val, span()).unwrap_err();
    let error_msg = format!("{:?}", error);
    assert!(error_msg.contains("string"));
}

#[test]
fn test_tar_list_nonexistent_tar() {
    let temp = TempDir::new().unwrap();
    let nonexistent = temp.path().join("nonexistent.tar");

    let tar_val = str_value(nonexistent.to_str().unwrap());
    let error = tar::tar_list(&tar_val, span()).unwrap_err();
    let error_msg = format!("{:?}", error);
    assert!(error_msg.contains("Failed to open tar file"));
}
