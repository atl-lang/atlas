# Phase 14b: Tar Archives

## 🚨 BLOCKERS
**REQUIRED:**
- Gzip compression (phase-14a) ✅
- File I/O (phase-05) ✅
- Path manipulation (phase-13a) ✅
- File system operations (phase-13b) ✅

## Objective
Implement tar archive creation and extraction with tar.gz support for package distribution and backups.

## Files
**Update:** `crates/atlas-runtime/src/stdlib/compression/tar.rs` (~400 lines)
**Update:** `crates/atlas-runtime/src/stdlib/compression/mod.rs` (add tar exports)
**Update:** `Cargo.toml` (add tar = "0.4")
**Update:** `crates/atlas-runtime/src/stdlib/mod.rs` (register tar functions)
**Tests:** `crates/atlas-runtime/tests/tar_tests.rs` (~400 lines)

## Dependencies
- tar = "0.4" (tar archive handling)
- flate2 (for tar.gz support)
- File I/O stdlib
- Path manipulation stdlib
- Gzip compression (phase-14a)

## Implementation

### Tar Archive Creation
Create tar with tarCreate function. Add files to archive. Add directories recursively. Preserve file metadata (permissions, timestamps, owner). Set base path for archive. Filter files during creation. Validate tar format. Return archive path.

### Tar.gz Compression
Create compressed tar.gz with tarCreateGz. Combine tar + gzip. Compression level setting. Stream tar.gz creation. Memory-efficient processing. Auto-detect .tar.gz extension.

### Tar Archive Extraction
Extract tar with tarExtract function. Extract to specified directory. Preserve metadata on extraction. Selective file extraction by pattern. Handle directory creation. Validate archive integrity. Path traversal prevention. Return extracted files list.

### Tar.gz Decompression
Extract tar.gz with tarExtractGz. Auto-decompress + extract. Stream processing. Memory limits. Handle corrupt archives. Validate both gzip and tar layers.

### Tar Utilities
List archive contents with tarList. Get file metadata from archive. Check if file exists in archive. Get archive size. Validate tar format. Append to existing tar. Convert tar to tar.gz.

### Error Handling
Corrupt archive detection. Checksum validation failures. Permission errors during creation/extraction. Disk space errors. Path traversal attack prevention. Invalid tar format errors. Missing file errors. Clear error messages.

## Tests (25+ tests)

**Tar creation tests (7):**
1. Create empty tar
2. Add single file
3. Add directory recursively
4. Preserve file permissions
5. Preserve timestamps
6. Filter files (glob pattern)
7. Validate tar format

**Tar.gz creation tests (4):**
1. Create tar.gz archive
2. Compression level setting
3. Large directory tar.gz
4. Stream tar.gz creation

**Tar extraction tests (7):**
1. Extract tar archive
2. Extract to directory
3. Preserve metadata
4. Extract specific files
5. Handle nested directories
6. Path traversal prevention
7. Corrupt tar handling

**Tar.gz extraction tests (4):**
1. Extract tar.gz
2. Auto-detect format
3. Large archive extraction
4. Corrupt tar.gz handling

**Tar utilities tests (3+):**
1. List tar contents
2. Check file exists
3. Append to tar

**Minimum test count:** 25 tests

## Integration Points
- Uses: Gzip compression (phase-14a)
- Uses: File I/O (phase-05)
- Uses: Path manipulation (phase-13a)
- Uses: File system ops (phase-13b)
- Creates: Tar archive utilities
- Output: Package distribution capabilities

## Acceptance
- Tar create/extract works
- Tar.gz support functional
- Metadata preservation works
- Path traversal prevented
- 25+ tests pass
- No clippy warnings
- cargo test passes
- Interpreter/VM parity maintained
