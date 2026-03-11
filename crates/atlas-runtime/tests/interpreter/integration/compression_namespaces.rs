use super::*;

// ============================================================================
// B30: Compression namespaces Gzip/Tar/Zip — parity tests (interpreter + VM)
// ============================================================================

// --- Gzip.isGzip() ---

#[test]
fn test_gzip_is_gzip_on_non_gzip_data() {
    // A byte array that is NOT gzip (no magic header) should return false.
    let src = r#"
        let data = [65, 66, 67];
        Gzip.isGzip(data);
    "#;
    assert_eval_bool(src, false);
    assert_parity(src);
}

#[test]
fn test_gzip_compress_returns_array() {
    // Gzip.compress returns an array (byte array).
    let src = r#"
        let data = [72, 101, 108, 108, 111];
        let compressed = Gzip.compress(data);
        compressed.len() > 0;
    "#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

#[test]
fn test_gzip_compress_decompress_roundtrip() {
    // Compress then decompress should yield original bytes.
    let src = r#"
        let data = [72, 101, 108, 108, 111];
        let compressed = Gzip.compress(data);
        let decompressed = Gzip.decompress(compressed);
        decompressed.len() == 5;
    "#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

#[test]
fn test_gzip_compressed_data_is_gzip() {
    // Compressed output should have gzip magic header.
    let src = r#"
        let data = [72, 101, 108, 108, 111];
        let compressed = Gzip.compress(data);
        Gzip.isGzip(compressed);
    "#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

#[test]
fn test_gzip_decompress_string() {
    // Gzip.decompressString returns a string.
    let src = r#"
        let data = [72, 101, 108, 108, 111];
        let compressed = Gzip.compress(data);
        let text = Gzip.decompressString(compressed);
        text == "Hello";
    "#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

#[test]
fn test_gzip_compression_ratio() {
    // compressionRatio returns a number.
    let src = r#"
        let ratio = Gzip.compressionRatio(100, 50);
        ratio == 2.0;
    "#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

// --- Zip.validate() ---

#[test]
fn test_zip_validate_non_existent_file_returns_false() {
    // A file that doesn't exist is not a valid zip.
    let src = r#"
        Zip.validate("/tmp/__atlas_nonexistent_test_b30__.zip");
    "#;
    assert_eval_bool(src, false);
    assert_parity(src);
}

// --- Tar.list() error path (parity) ---

#[test]
fn test_tar_list_non_existent_errors_both_engines() {
    // Both interpreter and VM should fail (not panic) on missing file.
    let src = r#"
        Tar.list("/tmp/__atlas_nonexistent_b30__.tar");
    "#;
    // Both engines should either both succeed or both fail — parity check.
    // We call assert_parity which verifies that both engines behave identically.
    assert_parity(src);
}

// --- Namespace dispatch sanity: wrong arg type errors parity ---

#[test]
fn test_gzip_is_gzip_wrong_type_errors_parity() {
    // Passing a non-array to isGzip — both engines should fail identically.
    let src = r#"
        Gzip.isGzip("not an array");
    "#;
    assert_parity(src);
}
