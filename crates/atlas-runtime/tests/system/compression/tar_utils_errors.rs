use super::*;
use atlas_runtime::stdlib::compression::tar;

// ============================================================================
// Tar Utility Tests
// ============================================================================

#[test]
fn test_tar_list_basic() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    std_fs::write(&test_file, "test content").unwrap();

    let tar_path = temp.path().join("archive.tar");

    let sources = vec![PathBuf::from(test_file.to_str().unwrap())];
    tar::create_tar(&sources, &tar_path, span()).unwrap();

    let tar_val = str_value(tar_path.to_str().unwrap());
    let result = tar::tar_list(&tar_val, span()).unwrap();

    match result {
        Value::Array(arr) => {
            let arr_guard = arr.as_slice();
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
    std_fs::write(&file1, "content 1").unwrap();
    std_fs::write(&file2, "content 2").unwrap();

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
            let arr_guard = arr.as_slice();
            assert_eq!(arr_guard.len(), 2);
        }
        _ => panic!("Expected array result"),
    }
}

#[test]
fn test_tar_contains_existing_file() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    std_fs::write(&test_file, "test content").unwrap();

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
    std_fs::write(&test_file, "test content").unwrap();

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
    std_fs::write(&test_file, content).unwrap();

    let tar_path = temp.path().join("archive.tar");

    // Create tar
    let sources = vec![PathBuf::from(test_file.to_str().unwrap())];
    tar::create_tar(&sources, &tar_path, span()).unwrap();

    // Extract tar
    let extract_dir = temp.path().join("extracted");
    tar::extract_tar(&tar_path, &extract_dir, span()).unwrap();

    // Verify content matches
    let extracted_file = extract_dir.join("test.txt");
    assert_eq!(std_fs::read_to_string(extracted_file).unwrap(), content);
}

#[test]
fn test_tar_gz_roundtrip_file() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    let content = "This is test content for tar.gz round-trip verification with compression";
    std_fs::write(&test_file, content).unwrap();

    let tar_gz_path = temp.path().join("archive.tar.gz");

    // Create tar.gz
    let sources = vec![PathBuf::from(test_file.to_str().unwrap())];
    tar::create_tar_gz(&sources, &tar_gz_path, 9, span()).unwrap();

    // Extract tar.gz
    let extract_dir = temp.path().join("extracted");
    tar::extract_tar_gz(&tar_gz_path, &extract_dir, span()).unwrap();

    // Verify content matches
    let extracted_file = extract_dir.join("test.txt");
    assert_eq!(std_fs::read_to_string(extracted_file).unwrap(), content);
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
        std_fs::read_to_string(extracted_dir.join("file1.txt")).unwrap(),
        "content 1"
    );
    assert_eq!(
        std_fs::read_to_string(extracted_dir.join("file2.txt")).unwrap(),
        "content 2"
    );
    assert_eq!(
        std_fs::read_to_string(extracted_dir.join("subdir").join("nested.txt")).unwrap(),
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
        std_fs::read_to_string(extracted_subdir.join("file1.txt")).unwrap(),
        "content 1"
    );
    assert_eq!(
        std_fs::read_to_string(extracted_subdir.join("file2.txt")).unwrap(),
        "content 2"
    );
}

#[test]
fn test_tar_roundtrip_with_list() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    std_fs::write(&test_file, "test content").unwrap();

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
            assert!(!arr.is_empty());
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
    std_fs::write(&test_file, "test content").unwrap();

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
