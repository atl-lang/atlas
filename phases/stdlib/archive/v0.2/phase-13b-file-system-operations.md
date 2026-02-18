# Phase 13b: File System Operations

## 🚨 BLOCKERS
**REQUIRED:**
- Path manipulation (phase-13a) ✅
- File I/O (phase-05) ✅

## Objective
Implement comprehensive file system utilities - directory operations, file metadata, temporary files, and symlinks using path manipulation from 13a.

## Files
**Create:** `crates/atlas-runtime/src/stdlib/fs.rs` (~600 lines)
**Update:** `crates/atlas-runtime/src/stdlib/mod.rs` (register fs module)
**Tests:** `crates/atlas-runtime/tests/fs_tests.rs` (~500 lines)

## Dependencies
- Path manipulation (phase-13a)
- File I/O operations (phase-05)
- std::fs for file system ops
- Security permissions (foundation/phase-15)

## Implementation

### Directory Operations
Create directory with mkdir. Create directory recursively with mkdirp. Remove directory with rmdir. Remove directory recursively with rmdirRecursive. List directory contents with readdir. Walk directory tree with walk. Filter directory listing. Sort directory entries. Glob pattern matching for paths.

### File Metadata
Get file metadata size, permissions, timestamps. File size with size function. Modified time with mtime. Created time with ctime. Access time with atime. File permissions query. Check if file is directory, file, or symlink. File type detection. Inode information (Unix).

### Temporary Files and Directories
Create temporary file with tmpfile. Create temporary directory with tmpdir. Automatic cleanup on exit. Named temporary files. Secure temporary file creation. Temporary path generation. Platform-specific temp directory.

### Symlink Operations
Create symbolic link with symlink. Read symlink target with readlink. Check if path is symlink. Resolve symlink chains. Relative and absolute symlinks. Platform support check. Permission handling for symlinks.

## Tests (TDD - Use rstest)

**Directory operations (8 tests):**
1. Create directory
2. Create directory recursively
3. Remove directory
4. Remove directory recursively
5. List directory contents
6. Walk directory tree
7. Glob pattern matching
8. Filter and sort entries

**File metadata (8 tests):**
1. Get file size
2. Get modified time
3. Get created time
4. Get access time
5. Query file permissions
6. Check is directory
7. Check is file
8. Check is symlink

**Temporary files (6 tests):**
1. Create temporary file
2. Create temporary directory
3. Automatic cleanup
4. Named temporary file
5. Secure temp creation
6. Temp directory location

**Symlink tests (6 tests):**
1. Create symbolic link
2. Read symlink target
3. Check is symlink
4. Resolve symlink chain
5. Relative symlink
6. Absolute symlink

**Integration (12+ tests):**
1. File system traversal
2. Recursive file operations
3. Path-based filtering
4. Real-world scenarios
5. Error handling (permissions denied, not found, etc.)
6. Cross-platform behavior
7. Large directory handling
8. Concurrent operations
9. Metadata preservation
10. Edge cases (special files, devices, etc.)
11. Security validation
12. Performance tests

**Minimum test count:** 40 tests

## Integration Points
- Uses: Path manipulation (phase-13a)
- Uses: File I/O (phase-05)
- Uses: Security permissions (foundation/phase-15)
- Creates: File system utilities
- Output: Complete fs operations API

## Acceptance
- Directory operations functional
- File metadata accessible
- Temporary files work securely
- Symlink operations supported
- Cross-platform compatibility verified
- 40+ tests pass on all platforms
- Documentation complete
- No clippy warnings
- cargo test passes
- Interpreter/VM parity maintained
