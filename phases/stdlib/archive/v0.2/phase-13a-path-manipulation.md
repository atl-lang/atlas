# Phase 13a: Path Manipulation

## 🚨 BLOCKERS
**REQUIRED:** String stdlib (phase-01) ✅

## Objective
Implement comprehensive path manipulation API enabling cross-platform path operations - construction, parsing, validation, and utilities (Node.js path module equivalent).

## Files
**Create:** `crates/atlas-runtime/src/stdlib/path.rs` (~700 lines)
**Update:** `crates/atlas-runtime/src/stdlib/mod.rs` (register path module)
**Tests:** `crates/atlas-runtime/tests/path_tests.rs` (~600 lines)

## Dependencies
- String manipulation (phase-01)
- std::path for cross-platform paths
- Security permissions (foundation/phase-15)

## Implementation

### Path Construction and Parsing
Join path components with join function. Parse path into components with parse. Normalize path removing dots and redundant separators. Absolute path conversion with absolute. Relative path computation with relative. Parent directory with parent. File name extraction with basename. Directory name with dirname. File extension with extension. Platform-specific separator handling.

### Path Comparison and Validation
Compare paths for equality. Check if path is absolute or relative. Validate path syntax. Check path exists with exists function. Canonical path resolution resolving symlinks. Case sensitivity handling per platform. Path prefix and suffix checking.

### Path Utilities
Home directory with homedir. Current working directory with cwd. System temp directory with tempdir. Path separator constant. Directory separator constant. Extension separator. Drive letter extraction (Windows). UNC path support (Windows). Path escaping for shell commands.

### Cross-Platform Compatibility
Handle Windows vs Unix path differences. Convert between path formats. Drive letters on Windows. UNC paths on Windows. Case insensitivity on Windows. Path length limits per platform.

## Tests (TDD - Use rstest)

**Path construction (10 tests):**
1. Join path components
2. Parse path
3. Normalize path
4. Absolute path conversion
5. Relative path computation
6. Parent directory
7. Basename extraction
8. Dirname extraction
9. Extension extraction
10. Platform separator handling

**Path validation (6 tests):**
1. Check path is absolute
2. Check path is relative
3. Path exists check
4. Canonical path resolution
5. Path comparison
6. Case sensitivity handling

**Path utilities (6 tests):**
1. Home directory
2. Current working directory
3. System temp directory
4. Path separator constant
5. Extension separator
6. Drive letter extraction (Windows)

**Cross-platform (7 tests):**
1. Windows path handling
2. Unix path handling
3. Path format conversion
4. UNC path support
5. Case insensitivity Windows
6. Path length limits
7. Real-world path scenarios

**Integration (5+ tests):**
1. Complex path operations
2. Path-based filtering
3. Cross-platform scripts
4. Edge cases (empty, dots, slashes)
5. Unicode paths

**Minimum test count:** 40 tests

## Integration Points
- Uses: String stdlib (phase-01)
- Uses: Security permissions (foundation/phase-15)
- Creates: Path manipulation API
- Output: Cross-platform path operations (no I/O)

## Acceptance
- Path construction and parsing work
- Path validation functional
- Path utilities comprehensive
- Cross-platform compatibility verified
- 40+ tests pass on all platforms
- Documentation with cross-platform notes
- No clippy warnings
- cargo test passes
- Interpreter/VM parity maintained
