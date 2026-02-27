use super::*;
use atlas_runtime::stdlib::compression::tar;

// ============================================================================
// Tar Extraction Tests
// ============================================================================

#[test]
fn test_tar_extract_basic() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    std_fs::write(&test_file, "test content").unwrap();

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
            let arr_guard = arr.as_slice();
            assert!(!arr_guard.is_empty());
        }
        _ => panic!("Expected array result"),
    }

    let extracted_file = extract_dir.join("test.txt");
    assert!(extracted_file.exists());
    assert_eq!(
        std_fs::read_to_string(extracted_file).unwrap(),
        "test content"
    );
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
    std_fs::write(&test_file, "test content").unwrap();

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
    assert_eq!(
        std_fs::read_to_string(nested_file).unwrap(),
        "nested content"
    );
}

#[test]
fn test_tar_extract_preserves_content() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    let content = "Hello, World! This is test content.";
    std_fs::write(&test_file, content).unwrap();

    let tar_path = temp.path().join("archive.tar");

    let sources = vec![PathBuf::from(test_file.to_str().unwrap())];
    tar::create_tar(&sources, &tar_path, span()).unwrap();

    let extract_dir = temp.path().join("extracted");
    let tar_val = str_value(tar_path.to_str().unwrap());
    let out_val = str_value(extract_dir.to_str().unwrap());

    tar::tar_extract(&tar_val, &out_val, span()).unwrap();

    let extracted_file = extract_dir.join("test.txt");
    assert!(extracted_file.exists());
    assert_eq!(std_fs::read_to_string(extracted_file).unwrap(), content);
}

#[test]
fn test_tar_extract_returns_file_list() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    std_fs::write(&test_file, "test content").unwrap();

    let tar_path = temp.path().join("archive.tar");

    let sources = vec![PathBuf::from(test_file.to_str().unwrap())];
    tar::create_tar(&sources, &tar_path, span()).unwrap();

    let extract_dir = temp.path().join("extracted");
    let tar_val = str_value(tar_path.to_str().unwrap());
    let out_val = str_value(extract_dir.to_str().unwrap());

    let result = tar::tar_extract(&tar_val, &out_val, span()).unwrap();
    match result {
        Value::Array(arr) => {
            let arr_guard = arr.as_slice();
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
    std_fs::write(&test_file, "test content for compression").unwrap();

    let tar_gz_path = temp.path().join("archive.tar.gz");

    let sources = vec![PathBuf::from(test_file.to_str().unwrap())];
    tar::create_tar_gz(&sources, &tar_gz_path, 6, span()).unwrap();

    let extract_dir = temp.path().join("extracted");
    let tar_gz_val = str_value(tar_gz_path.to_str().unwrap());
    let out_val = str_value(extract_dir.to_str().unwrap());

    let result = tar::tar_extract_gz(&tar_gz_val, &out_val, span()).unwrap();
    match result {
        Value::Array(arr) => {
            let arr_guard = arr.as_slice();
            assert!(!arr_guard.is_empty());
        }
        _ => panic!("Expected array result"),
    }

    let extracted_file = extract_dir.join("test.txt");
    assert!(extracted_file.exists());
    assert_eq!(
        std_fs::read_to_string(extracted_file).unwrap(),
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
