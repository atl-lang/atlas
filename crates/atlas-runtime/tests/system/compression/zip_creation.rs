use super::*;
use atlas_runtime::stdlib::compression::zip as atlas_zip;

// Zip Creation Tests (9)
// ============================================================================

/// 1. Create an empty zip archive (no sources)
#[test]
fn test_zip_create_empty() {
    let temp = TempDir::new().unwrap();
    let zip_path = temp.path().join("empty.zip");

    let sources = str_array_value(&[]);
    let output = str_value(zip_path.to_str().unwrap());

    let result = atlas_zip::zip_create(&sources, &output, None, span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(zip_path.exists());
    assert!(zip_path.metadata().unwrap().len() > 0);
}

/// 2. Create zip with a single file
#[test]
fn test_zip_create_single_file() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("hello.txt");
    std_fs::write(&test_file, "Hello, Atlas!").unwrap();

    let zip_path = temp.path().join("single.zip");
    let sources = str_array_value(&[test_file.to_str().unwrap()]);
    let output = str_value(zip_path.to_str().unwrap());

    let result = atlas_zip::zip_create(&sources, &output, None, span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(zip_path.exists());
}

/// 3. Create zip from a directory (recursively adds all contents)
#[test]
fn test_zip_create_directory_recursive() {
    let temp = TempDir::new().unwrap();
    let test_dir = create_test_dir(temp.path(), "myproject");
    create_test_file(&test_dir, "main.atlas", "fn main() {}");
    create_test_file(&test_dir, "README.md", "# My Project");

    let sub = create_test_dir(&test_dir, "src");
    create_test_file(&sub, "lib.atlas", "// lib");

    let zip_path = temp.path().join("project.zip");
    let sources = str_array_value(&[test_dir.to_str().unwrap()]);
    let output = str_value(zip_path.to_str().unwrap());

    let result = atlas_zip::zip_create(&sources, &output, None, span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(zip_path.exists());

    // Verify contents include nested file
    let val = atlas_zip::zip_contains_file(&output, &str_value("myproject/src/lib.atlas"), span())
        .unwrap();
    assert_eq!(val, Value::Bool(true));
}

/// 4. Store compression (level 0 — no compression)
#[test]
fn test_zip_create_store_compression() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("data.txt");
    std_fs::write(&test_file, "some data").unwrap();

    let zip_path = temp.path().join("stored.zip");
    let sources = str_array_value(&[test_file.to_str().unwrap()]);
    let output = str_value(zip_path.to_str().unwrap());

    let result = atlas_zip::zip_create(&sources, &output, Some(&num_value(0.0)), span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(zip_path.exists());

    // With STORE, the entry's compressed size should equal the uncompressed size
    let list = atlas_zip::zip_list(&output, span()).unwrap();
    if let Value::Array(arr) = list {
        let guard = arr.as_slice();
        if let Some(Value::HashMap(entry_map)) = guard.first() {
            let guard = entry_map.inner();
            use atlas_runtime::stdlib::collections::hash::HashKey;
            use std::sync::Arc;
            let method_key = HashKey::String(Arc::new("method".to_string()));
            if let Some(Value::String(method)) = guard.get(&method_key) {
                assert_eq!(method.as_ref(), "stored");
            }
        }
    }
}

/// 5. Deflate compression at level 6 (default)
#[test]
fn test_zip_create_deflate_level_6() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("text.txt");
    // Write compressible data
    let content = "aaaaaaaaaa".repeat(1000);
    std_fs::write(&test_file, &content).unwrap();

    let zip_path = temp.path().join("deflate6.zip");
    let sources = str_array_value(&[test_file.to_str().unwrap()]);
    let output = str_value(zip_path.to_str().unwrap());

    let result = atlas_zip::zip_create(&sources, &output, Some(&num_value(6.0)), span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(zip_path.exists());

    // Compressed size should be smaller than original
    let original_size = std_fs::metadata(&test_file).unwrap().len();
    let zip_size = std_fs::metadata(&zip_path).unwrap().len();
    assert!(zip_size < original_size);
}

/// 6. Deflate compression at level 9 (maximum)
#[test]
fn test_zip_create_deflate_level_9() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("text.txt");
    let content = "bbbbbbbbbb".repeat(1000);
    std_fs::write(&test_file, &content).unwrap();

    let zip_path = temp.path().join("deflate9.zip");
    let sources = str_array_value(&[test_file.to_str().unwrap()]);
    let output = str_value(zip_path.to_str().unwrap());

    let result = atlas_zip::zip_create(&sources, &output, Some(&num_value(9.0)), span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(zip_path.exists());
}

/// 7. Verify file metadata is preserved (content round-trip)
#[test]
fn test_zip_create_preserves_file_content() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("data.json");
    let content = r#"{"key": "value", "num": 42}"#;
    std_fs::write(&test_file, content).unwrap();

    let zip_path = temp.path().join("meta.zip");
    let sources = str_array_value(&[test_file.to_str().unwrap()]);
    let output = str_value(zip_path.to_str().unwrap());
    atlas_zip::zip_create(&sources, &output, None, span()).unwrap();

    // Extract and verify content is identical
    let extract_dir = temp.path().join("extracted");
    atlas_zip::zip_extract(&output, &str_value(extract_dir.to_str().unwrap()), span()).unwrap();
    let extracted_content = std_fs::read_to_string(extract_dir.join("data.json")).unwrap();
    assert_eq!(extracted_content, content);
}

/// 8. Create zip with an archive-level comment
#[test]
fn test_zip_create_with_comment() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("note.txt");
    std_fs::write(&test_file, "contents").unwrap();

    let zip_path = temp.path().join("commented.zip");
    let sources = str_array_value(&[test_file.to_str().unwrap()]);
    let output = str_value(zip_path.to_str().unwrap());
    let comment = str_value("Atlas archive v1.0");

    let result =
        atlas_zip::zip_create_with_comment(&sources, &output, &comment, None, span()).unwrap();
    assert_eq!(result, Value::Null);
    assert!(zip_path.exists());

    // Read back the comment
    let read_comment = atlas_zip::zip_comment_fn(&output, span()).unwrap();
    assert_eq!(
        read_comment,
        Value::string("Atlas archive v1.0".to_string())
    );
}

/// 9. Filter files during creation (create zip with subset of files)
#[test]
fn test_zip_create_filtered_sources() {
    let temp = TempDir::new().unwrap();
    let atlas_file = temp.path().join("main.atlas");
    let txt_file = temp.path().join("notes.txt");
    let rs_file = temp.path().join("helper.rs");
    std_fs::write(&atlas_file, "fn main() {}").unwrap();
    std_fs::write(&txt_file, "notes").unwrap();
    std_fs::write(&rs_file, "fn helper() {}").unwrap();

    // Only include .atlas and .txt files (simulate filtering)
    let zip_path = temp.path().join("filtered.zip");
    let sources = str_array_value(&[atlas_file.to_str().unwrap(), txt_file.to_str().unwrap()]);
    let output = str_value(zip_path.to_str().unwrap());
    atlas_zip::zip_create(&sources, &output, None, span()).unwrap();

    // Verify only filtered files are included
    let contains_atlas =
        atlas_zip::zip_contains_file(&output, &str_value("main.atlas"), span()).unwrap();
    let contains_txt =
        atlas_zip::zip_contains_file(&output, &str_value("notes.txt"), span()).unwrap();
    let contains_rs =
        atlas_zip::zip_contains_file(&output, &str_value("helper.rs"), span()).unwrap();
    assert_eq!(contains_atlas, Value::Bool(true));
    assert_eq!(contains_txt, Value::Bool(true));
    assert_eq!(contains_rs, Value::Bool(false));
}

// ============================================================================
