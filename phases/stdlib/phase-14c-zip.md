# Phase 14c: Zip Archives

## 🚨 BLOCKERS
**REQUIRED:**
- File I/O (phase-05) ✅
- Path manipulation (phase-13a) ✅
- File system operations (phase-13b) ✅

## Objective
Implement zip archive creation and extraction with deflate compression for cross-platform archive management.

## Files
**Create:** `crates/atlas-runtime/src/stdlib/compression/zip.rs` (~400 lines)
**Update:** `crates/atlas-runtime/src/stdlib/compression/mod.rs` (add zip exports)
**Update:** `Cargo.toml` (add zip = "0.6")
**Update:** `crates/atlas-runtime/src/stdlib/mod.rs` (register zip functions)
**Tests:** `crates/atlas-runtime/tests/zip_tests.rs` (~400 lines)

## Dependencies
- zip = "0.6" (zip archive handling)
- File I/O stdlib
- Path manipulation stdlib
- File system operations stdlib

## Implementation

### Zip Archive Creation
Create zip with zipCreate function. Add files to archive. Add directories recursively. Store or deflate compression methods. Compression level setting (0-9). Preserve file metadata. Archive comments support. Return archive path.

### Zip Compression Options
Set compression method (store/deflate). Set compression level. Add file with custom settings. Preserve directory structure. Handle symlinks. Filter files during creation. Central directory validation.

### Zip Archive Extraction
Extract zip with zipExtract function. Extract to specified directory. Preserve directory structure. Extract specific files by pattern. Handle nested directories. Validate zip format. Path traversal prevention. Return extracted files list.

### Zip Utilities
List archive contents with zipList. Get file metadata from archive. Check if file exists in archive. Get archive comment. Get compression ratio. Validate zip format. Add to existing zip. Remove file from zip.

### Advanced Features (Optional)
Password protection (if zip crate supports). Extract to memory option. Streaming extraction for large files. Update file in archive. Convert formats.

### Error Handling
Corrupt archive detection. CRC validation failures. Permission errors during creation/extraction. Disk space errors. Path traversal attack prevention. Invalid zip format errors. Unsupported compression method. Clear error messages.

## Tests (25+ tests)

**Zip creation tests (8):**
1. Create empty zip
2. Add single file
3. Add directory recursively
4. Store compression (level 0)
5. Deflate compression (level 6, 9)
6. Preserve file metadata
7. Archive comments
8. Filter files

**Zip extraction tests (7):**
1. Extract zip archive
2. Extract to directory
3. Preserve directory structure
4. Extract specific files
5. Handle nested directories
6. Path traversal prevention
7. Corrupt zip handling

**Zip utilities tests (5):**
1. List zip contents
2. Check file exists
3. Get compression ratio
4. Add to existing zip
5. Validate zip format

**Compression tests (3):**
1. Store vs deflate comparison
2. Compression levels
3. Large file compression

**Integration tests (2+):**
1. Real-world archive operations
2. Cross-platform compatibility

**Minimum test count:** 25 tests

## Integration Points
- Uses: File I/O (phase-05)
- Uses: Path manipulation (phase-13a)
- Uses: File system ops (phase-13b)
- Creates: Zip archive utilities
- Output: Cross-platform archive format

## Acceptance
- Zip create/extract works
- Deflate compression functional
- Metadata preservation works
- Path traversal prevented
- 25+ tests pass
- No clippy warnings
- cargo test passes
- Interpreter/VM parity maintained
