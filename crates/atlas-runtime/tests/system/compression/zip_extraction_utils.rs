use super::*;
use atlas_runtime::stdlib::compression::zip as atlas_zip;

// Zip Extraction Tests (7)
// ============================================================================

/// 10. Extract a zip archive
#[test]
fn test_zip_extract_archive() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("extract_me.txt");
    std_fs::write(&test_file, "extractable content").unwrap();

    let zip_path = temp.path().join("archive.zip");
    let sources = str_array_value(&[test_file.to_str().unwrap()]);
    let output = str_value(zip_path.to_str().unwrap());
    atlas_zip::zip_create(&sources, &output, None, span()).unwrap();

    let extract_dir = temp.path().join("out");
    let result =
        atlas_zip::zip_extract(&output, &str_value(extract_dir.to_str().unwrap()), span()).unwrap();

    // Returns array of extracted file paths
    assert!(matches!(result, Value::Array(_)));
    assert!(extract_dir.join("extract_me.txt").exists());
}

/// 11. Extract to a specified directory
#[test]
fn test_zip_extract_to_directory() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("file.txt");
    std_fs::write(&test_file, "hello").unwrap();

    let zip_path = temp.path().join("arch.zip");
    atlas_zip::zip_create(
        &str_array_value(&[test_file.to_str().unwrap()]),
        &str_value(zip_path.to_str().unwrap()),
        None,
        span(),
    )
    .unwrap();

    let dest = temp.path().join("destination");
    atlas_zip::zip_extract(
        &str_value(zip_path.to_str().unwrap()),
        &str_value(dest.to_str().unwrap()),
        span(),
    )
    .unwrap();

    assert!(dest.exists());
    assert!(dest.join("file.txt").exists());
    let content = std_fs::read_to_string(dest.join("file.txt")).unwrap();
    assert_eq!(content, "hello");
}

/// 12. Preserve directory structure on extraction
#[test]
fn test_zip_extract_preserves_directory_structure() {
    let temp = TempDir::new().unwrap();
    let src_dir = create_test_dir(temp.path(), "project");
    let sub = create_test_dir(&src_dir, "lib");
    create_test_file(&src_dir, "main.atlas", "main");
    create_test_file(&sub, "utils.atlas", "utils");

    let zip_path = temp.path().join("project.zip");
    atlas_zip::zip_create(
        &str_array_value(&[src_dir.to_str().unwrap()]),
        &str_value(zip_path.to_str().unwrap()),
        None,
        span(),
    )
    .unwrap();

    let out = temp.path().join("output");
    atlas_zip::zip_extract(
        &str_value(zip_path.to_str().unwrap()),
        &str_value(out.to_str().unwrap()),
        span(),
    )
    .unwrap();

    assert!(out.join("project").join("main.atlas").exists());
    assert!(out.join("project").join("lib").join("utils.atlas").exists());
}

/// 13. Extract specific files by name
#[test]
fn test_zip_extract_specific_files() {
    let temp = TempDir::new().unwrap();
    let file_a = temp.path().join("a.txt");
    let file_b = temp.path().join("b.txt");
    std_fs::write(&file_a, "aaa").unwrap();
    std_fs::write(&file_b, "bbb").unwrap();

    let zip_path = temp.path().join("both.zip");
    atlas_zip::zip_create(
        &str_array_value(&[file_a.to_str().unwrap(), file_b.to_str().unwrap()]),
        &str_value(zip_path.to_str().unwrap()),
        None,
        span(),
    )
    .unwrap();

    let out = temp.path().join("partial");
    let files_to_extract = str_array_value(&["a.txt"]);
    let result = atlas_zip::zip_extract_files(
        &str_value(zip_path.to_str().unwrap()),
        &str_value(out.to_str().unwrap()),
        &files_to_extract,
        span(),
    )
    .unwrap();

    assert!(matches!(result, Value::Array(_)));
    assert!(out.join("a.txt").exists());
    assert!(!out.join("b.txt").exists());
}

/// 14. Handle nested directories on extraction
#[test]
fn test_zip_extract_nested_directories() {
    let temp = TempDir::new().unwrap();
    let root = create_test_dir(temp.path(), "root");
    let level1 = create_test_dir(&root, "level1");
    let level2 = create_test_dir(&level1, "level2");
    create_test_file(&level2, "deep.txt", "deep content");

    let zip_path = temp.path().join("nested.zip");
    atlas_zip::zip_create(
        &str_array_value(&[root.to_str().unwrap()]),
        &str_value(zip_path.to_str().unwrap()),
        None,
        span(),
    )
    .unwrap();

    let out = temp.path().join("nested_out");
    atlas_zip::zip_extract(
        &str_value(zip_path.to_str().unwrap()),
        &str_value(out.to_str().unwrap()),
        span(),
    )
    .unwrap();

    assert!(out
        .join("root")
        .join("level1")
        .join("level2")
        .join("deep.txt")
        .exists());
    let content = std_fs::read_to_string(
        out.join("root")
            .join("level1")
            .join("level2")
            .join("deep.txt"),
    )
    .unwrap();
    assert_eq!(content, "deep content");
}

/// 15. Path traversal prevention
#[test]
fn test_zip_extract_path_traversal_prevention() {
    use ::zip::write::FileOptions;
    use ::zip::ZipWriter as ExtZipWriter;
    use std::io::Write as IoWrite;

    let temp = TempDir::new().unwrap();
    let malicious_zip = temp.path().join("malicious.zip");

    // Manually craft a zip with a path traversal entry
    let file = std_fs::File::create(&malicious_zip).unwrap();
    let mut writer = ExtZipWriter::new(file);
    let opts = FileOptions::default();
    writer.start_file("../../../etc/passwd", opts).unwrap();
    writer.write_all(b"evil content").unwrap();
    writer.finish().unwrap();

    let out = temp.path().join("safe_out");
    let result = atlas_zip::zip_extract(
        &str_value(malicious_zip.to_str().unwrap()),
        &str_value(out.to_str().unwrap()),
        span(),
    );

    // Must error or extract safely
    if result.is_ok() {
        // If it didn't error, verify the file was not extracted outside the output dir
        let escaped = temp
            .path()
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("etc").join("passwd"));
        if let Some(escaped_path) = escaped {
            assert!(
                !escaped_path.exists(),
                "Path traversal succeeded - security bug!"
            );
        }
    } else {
        // Error is the expected behaviour
        assert!(result.is_err());
    }
}

/// 16. Corrupt zip returns an error
#[test]
fn test_zip_extract_corrupt_zip_error() {
    let temp = TempDir::new().unwrap();
    let corrupt = temp.path().join("corrupt.zip");
    std_fs::write(&corrupt, b"this is not a zip file at all").unwrap();

    let out = temp.path().join("out");
    let result = atlas_zip::zip_extract(
        &str_value(corrupt.to_str().unwrap()),
        &str_value(out.to_str().unwrap()),
        span(),
    );

    assert!(result.is_err());
}

// ============================================================================
// Zip Utilities Tests (5)
// ============================================================================

/// 17. List zip archive contents
#[test]
fn test_zip_list_contents() {
    let temp = TempDir::new().unwrap();
    let f1 = temp.path().join("alpha.txt");
    let f2 = temp.path().join("beta.txt");
    std_fs::write(&f1, "alpha").unwrap();
    std_fs::write(&f2, "beta").unwrap();

    let zip_path = temp.path().join("list_test.zip");
    atlas_zip::zip_create(
        &str_array_value(&[f1.to_str().unwrap(), f2.to_str().unwrap()]),
        &str_value(zip_path.to_str().unwrap()),
        None,
        span(),
    )
    .unwrap();

    let list = atlas_zip::zip_list(&str_value(zip_path.to_str().unwrap()), span()).unwrap();

    if let Value::Array(arr) = &list {
        let guard = arr.as_slice();
        assert_eq!(guard.len(), 2);
    } else {
        panic!("zipList should return an array");
    }
}

/// 18. Check file exists in zip (present)
#[test]
fn test_zip_contains_present() {
    let temp = TempDir::new().unwrap();
    let f = temp.path().join("present.txt");
    std_fs::write(&f, "here").unwrap();

    let zip_path = temp.path().join("contains.zip");
    atlas_zip::zip_create(
        &str_array_value(&[f.to_str().unwrap()]),
        &str_value(zip_path.to_str().unwrap()),
        None,
        span(),
    )
    .unwrap();

    let result = atlas_zip::zip_contains_file(
        &str_value(zip_path.to_str().unwrap()),
        &str_value("present.txt"),
        span(),
    )
    .unwrap();

    assert_eq!(result, Value::Bool(true));
}

/// 19. Check file exists in zip (absent)
#[test]
fn test_zip_contains_absent() {
    let temp = TempDir::new().unwrap();
    let f = temp.path().join("file.txt");
    std_fs::write(&f, "here").unwrap();

    let zip_path = temp.path().join("contains2.zip");
    atlas_zip::zip_create(
        &str_array_value(&[f.to_str().unwrap()]),
        &str_value(zip_path.to_str().unwrap()),
        None,
        span(),
    )
    .unwrap();

    let result = atlas_zip::zip_contains_file(
        &str_value(zip_path.to_str().unwrap()),
        &str_value("ghost.txt"),
        span(),
    )
    .unwrap();

    assert_eq!(result, Value::Bool(false));
}

/// 20. Get compression ratio
#[test]
fn test_zip_compression_ratio() {
    let temp = TempDir::new().unwrap();
    let f = temp.path().join("ratio.txt");
    // Write highly compressible data
    std_fs::write(&f, "x".repeat(10000)).unwrap();

    let zip_path = temp.path().join("ratio.zip");
    atlas_zip::zip_create(
        &str_array_value(&[f.to_str().unwrap()]),
        &str_value(zip_path.to_str().unwrap()),
        Some(&num_value(6.0)),
        span(),
    )
    .unwrap();

    let ratio =
        atlas_zip::zip_compression_ratio(&str_value(zip_path.to_str().unwrap()), span()).unwrap();

    if let Value::Number(r) = ratio {
        assert!(r >= 0.0, "ratio should be non-negative");
        assert!(r <= 1.0, "deflate ratio should be at most 1.0");
        assert!(r < 0.5, "10000 'x' chars should compress well");
    } else {
        panic!("zipCompressionRatio should return a number");
    }
}

/// 21. Add file to existing zip
#[test]
fn test_zip_add_file_to_existing() {
    let temp = TempDir::new().unwrap();
    let original = temp.path().join("original.txt");
    let addition = temp.path().join("added.txt");
    std_fs::write(&original, "original").unwrap();
    std_fs::write(&addition, "added file").unwrap();

    let zip_path = temp.path().join("grow.zip");
    atlas_zip::zip_create(
        &str_array_value(&[original.to_str().unwrap()]),
        &str_value(zip_path.to_str().unwrap()),
        None,
        span(),
    )
    .unwrap();

    // Add new file
    let result = atlas_zip::zip_add_file_fn(
        &str_value(zip_path.to_str().unwrap()),
        &str_value(addition.to_str().unwrap()),
        None,
        None,
        span(),
    )
    .unwrap();

    assert_eq!(result, Value::Null);

    // Both files should now be in the archive
    let has_original = atlas_zip::zip_contains_file(
        &str_value(zip_path.to_str().unwrap()),
        &str_value("original.txt"),
        span(),
    )
    .unwrap();
    let has_added = atlas_zip::zip_contains_file(
        &str_value(zip_path.to_str().unwrap()),
        &str_value("added.txt"),
        span(),
    )
    .unwrap();

    assert_eq!(has_original, Value::Bool(true));
    assert_eq!(has_added, Value::Bool(true));
}

// ============================================================================
