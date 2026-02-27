use super::*;
use atlas_runtime::stdlib::compression::zip as atlas_zip;

// Zip Validation Tests (2)
// ============================================================================

/// 22. Validate a valid zip file
#[test]
fn test_zip_validate_valid() {
    let temp = TempDir::new().unwrap();
    let f = temp.path().join("val.txt");
    std_fs::write(&f, "validate me").unwrap();

    let zip_path = temp.path().join("valid.zip");
    atlas_zip::zip_create(
        &str_array_value(&[f.to_str().unwrap()]),
        &str_value(zip_path.to_str().unwrap()),
        None,
        span(),
    )
    .unwrap();

    let result =
        atlas_zip::zip_validate_fn(&str_value(zip_path.to_str().unwrap()), span()).unwrap();
    assert_eq!(result, Value::Bool(true));
}

/// 23. Validate an invalid file
#[test]
fn test_zip_validate_invalid() {
    let temp = TempDir::new().unwrap();
    let not_a_zip = temp.path().join("fake.zip");
    std_fs::write(&not_a_zip, b"PKnot a real zip file").unwrap();

    let result =
        atlas_zip::zip_validate_fn(&str_value(not_a_zip.to_str().unwrap()), span()).unwrap();
    assert_eq!(result, Value::Bool(false));
}

// ============================================================================
// Compression Method Tests (3)
// ============================================================================

/// 24. Store vs deflate: stored archive is larger
#[test]
fn test_zip_store_vs_deflate_size_comparison() {
    let temp = TempDir::new().unwrap();
    let f = temp.path().join("compare.txt");
    // Highly compressible content
    std_fs::write(&f, "z".repeat(50000)).unwrap();

    let stored_zip = temp.path().join("stored.zip");
    let deflated_zip = temp.path().join("deflated.zip");

    let sources = str_array_value(&[f.to_str().unwrap()]);

    atlas_zip::zip_create(
        &sources,
        &str_value(stored_zip.to_str().unwrap()),
        Some(&num_value(0.0)),
        span(),
    )
    .unwrap();
    atlas_zip::zip_create(
        &sources,
        &str_value(deflated_zip.to_str().unwrap()),
        Some(&num_value(6.0)),
        span(),
    )
    .unwrap();

    let stored_size = std_fs::metadata(&stored_zip).unwrap().len();
    let deflated_size = std_fs::metadata(&deflated_zip).unwrap().len();

    // Deflated archive must be smaller than stored for highly compressible data
    assert!(
        deflated_size < stored_size,
        "deflated ({}) should be smaller than stored ({})",
        deflated_size,
        stored_size
    );
}

/// 25. Multiple compression levels all produce valid archives
#[test]
fn test_zip_compression_levels_all_valid() {
    let temp = TempDir::new().unwrap();
    let f = temp.path().join("levels.txt");
    std_fs::write(&f, "test data for compression level testing").unwrap();

    for level in [0.0_f64, 1.0, 3.0, 5.0, 6.0, 9.0] {
        let zip_path = temp.path().join(format!("level_{}.zip", level as u32));
        atlas_zip::zip_create(
            &str_array_value(&[f.to_str().unwrap()]),
            &str_value(zip_path.to_str().unwrap()),
            Some(&num_value(level)),
            span(),
        )
        .unwrap();

        let valid =
            atlas_zip::zip_validate_fn(&str_value(zip_path.to_str().unwrap()), span()).unwrap();
        assert_eq!(
            valid,
            Value::Bool(true),
            "Level {} should produce a valid zip",
            level
        );
    }
}

/// 26. Large file compression (> 512 KB)
#[test]
fn test_zip_large_file_compression() {
    let temp = TempDir::new().unwrap();
    let large_file = temp.path().join("large.bin");
    // 1 MB of repeating bytes (highly compressible)
    let data: Vec<u8> = (0..1024 * 1024).map(|i| (i % 64) as u8).collect();
    std_fs::write(&large_file, &data).unwrap();

    let zip_path = temp.path().join("large.zip");
    let result = atlas_zip::zip_create(
        &str_array_value(&[large_file.to_str().unwrap()]),
        &str_value(zip_path.to_str().unwrap()),
        Some(&num_value(6.0)),
        span(),
    )
    .unwrap();

    assert_eq!(result, Value::Null);
    assert!(zip_path.exists());

    // Zip should be smaller than original
    let zip_size = std_fs::metadata(&zip_path).unwrap().len();
    assert!(zip_size < data.len() as u64);
}

// ============================================================================
// Integration Tests (3)
// ============================================================================

/// 27. Full round-trip: create → extract → verify content integrity
#[test]
fn test_zip_round_trip_integrity() {
    let temp = TempDir::new().unwrap();

    // Create source files with known content
    let src = create_test_dir(temp.path(), "source");
    create_test_file(&src, "config.toml", "[package]\nname = \"atlas\"\n");
    create_test_file(&src, "README.md", "# Atlas\nFast compiler.\n");

    let sub = create_test_dir(&src, "tests");
    create_test_file(
        &sub,
        "test_suite.atlas",
        "fn test_basic() { assert(1 == 1); }",
    );

    // Create zip
    let zip_path = temp.path().join("roundtrip.zip");
    atlas_zip::zip_create(
        &str_array_value(&[src.to_str().unwrap()]),
        &str_value(zip_path.to_str().unwrap()),
        Some(&num_value(6.0)),
        span(),
    )
    .unwrap();

    // Extract zip
    let out = temp.path().join("restored");
    atlas_zip::zip_extract(
        &str_value(zip_path.to_str().unwrap()),
        &str_value(out.to_str().unwrap()),
        span(),
    )
    .unwrap();

    // Verify content integrity
    let config = std_fs::read_to_string(out.join("source").join("config.toml")).unwrap();
    let readme = std_fs::read_to_string(out.join("source").join("README.md")).unwrap();
    let test_suite =
        std_fs::read_to_string(out.join("source").join("tests").join("test_suite.atlas")).unwrap();

    assert_eq!(config, "[package]\nname = \"atlas\"\n");
    assert_eq!(readme, "# Atlas\nFast compiler.\n");
    assert_eq!(test_suite, "fn test_basic() { assert(1 == 1); }");
}

/// 28. Zip list returns correct metadata fields
#[test]
fn test_zip_list_metadata_fields() {
    let temp = TempDir::new().unwrap();
    let f = temp.path().join("meta_test.txt");
    std_fs::write(&f, "metadata check").unwrap();

    let zip_path = temp.path().join("metadata.zip");
    atlas_zip::zip_create(
        &str_array_value(&[f.to_str().unwrap()]),
        &str_value(zip_path.to_str().unwrap()),
        None,
        span(),
    )
    .unwrap();

    let list = atlas_zip::zip_list(&str_value(zip_path.to_str().unwrap()), span()).unwrap();

    if let Value::Array(arr) = &list {
        let guard = arr.as_slice();
        let first = &guard[0];

        if let Value::HashMap(map) = first {
            use atlas_runtime::stdlib::collections::hash::HashKey;
            use std::sync::Arc;

            let map_guard = map.inner();

            // Must have all required fields
            let name_key = HashKey::String(Arc::new("name".to_string()));
            let size_key = HashKey::String(Arc::new("size".to_string()));
            let csize_key = HashKey::String(Arc::new("compressedSize".to_string()));
            let isdir_key = HashKey::String(Arc::new("isDir".to_string()));
            let method_key = HashKey::String(Arc::new("method".to_string()));

            assert!(map_guard.get(&name_key).is_some(), "missing 'name' field");
            assert!(map_guard.get(&size_key).is_some(), "missing 'size' field");
            assert!(
                map_guard.get(&csize_key).is_some(),
                "missing 'compressedSize' field"
            );
            assert!(map_guard.get(&isdir_key).is_some(), "missing 'isDir' field");
            assert!(
                map_guard.get(&method_key).is_some(),
                "missing 'method' field"
            );

            // File should not be a directory
            assert_eq!(map_guard.get(&isdir_key), Some(&Value::Bool(false)));
        } else {
            panic!("list entry should be a HashMap");
        }
    } else {
        panic!("zipList should return an Array");
    }
}

/// 29. Comment round-trip: write and read back correctly
#[test]
fn test_zip_comment_round_trip() {
    let temp = TempDir::new().unwrap();
    let f = temp.path().join("note.txt");
    std_fs::write(&f, "noted").unwrap();

    let zip_path = temp.path().join("with_comment.zip");
    let long_comment = "This archive was created by Atlas stdlib phase-14c. Version: 0.2.0-dev";

    atlas_zip::zip_create_with_comment(
        &str_array_value(&[f.to_str().unwrap()]),
        &str_value(zip_path.to_str().unwrap()),
        &str_value(long_comment),
        Some(&num_value(6.0)),
        span(),
    )
    .unwrap();

    let read_back =
        atlas_zip::zip_comment_fn(&str_value(zip_path.to_str().unwrap()), span()).unwrap();
    assert_eq!(read_back, Value::string(long_comment.to_string()));
}

// ============================================================================
// Error Handling Tests (4)
// ============================================================================

/// 30. Missing source file returns error
#[test]
fn test_zip_create_missing_source_error() {
    let temp = TempDir::new().unwrap();
    let zip_path = temp.path().join("will_fail.zip");

    let result = atlas_zip::zip_create(
        &str_array_value(&["/does/not/exist/file.txt"]),
        &str_value(zip_path.to_str().unwrap()),
        None,
        span(),
    );

    assert!(result.is_err());
}

/// 31. Invalid compression level returns error
#[test]
fn test_zip_create_invalid_level_error() {
    let temp = TempDir::new().unwrap();
    let f = temp.path().join("ok.txt");
    std_fs::write(&f, "ok").unwrap();
    let zip_path = temp.path().join("bad_level.zip");

    let result = atlas_zip::zip_create(
        &str_array_value(&[f.to_str().unwrap()]),
        &str_value(zip_path.to_str().unwrap()),
        Some(&num_value(10.0)), // out of range
        span(),
    );

    assert!(result.is_err());
}

/// 32. Extract specific file that doesn't exist returns error
#[test]
fn test_zip_extract_missing_entry_error() {
    let temp = TempDir::new().unwrap();
    let f = temp.path().join("a.txt");
    std_fs::write(&f, "a").unwrap();

    let zip_path = temp.path().join("one_file.zip");
    atlas_zip::zip_create(
        &str_array_value(&[f.to_str().unwrap()]),
        &str_value(zip_path.to_str().unwrap()),
        None,
        span(),
    )
    .unwrap();

    let out = temp.path().join("out");
    let result = atlas_zip::zip_extract_files(
        &str_value(zip_path.to_str().unwrap()),
        &str_value(out.to_str().unwrap()),
        &str_array_value(&["nonexistent.txt"]),
        span(),
    );

    assert!(result.is_err());
}

/// 33. Type error on non-string sources returns error
#[test]
fn test_zip_create_type_error_sources() {
    let temp = TempDir::new().unwrap();
    let zip_path = temp.path().join("type_err.zip");

    // Pass a number inside the sources array
    let bad_sources = Value::array(vec![Value::Number(42.0)]);
    let result = atlas_zip::zip_create(
        &bad_sources,
        &str_value(zip_path.to_str().unwrap()),
        None,
        span(),
    );

    assert!(result.is_err());
}
