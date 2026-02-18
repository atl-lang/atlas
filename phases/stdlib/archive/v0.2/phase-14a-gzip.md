# Phase 14a: Gzip Compression

## 🚨 BLOCKERS
**REQUIRED:**
- File I/O (phase-05) ✅
- Path manipulation (phase-13a) ✅

## Objective
Implement gzip compression/decompression utilities for data compression, file compression, and streaming operations.

## Files
**Create:** `crates/atlas-runtime/src/stdlib/compression/mod.rs` (~100 lines)
**Create:** `crates/atlas-runtime/src/stdlib/compression/gzip.rs` (~300 lines)
**Update:** `Cargo.toml` (add flate2 = "1.0")
**Update:** `crates/atlas-runtime/src/stdlib/mod.rs` (register gzip functions)
**Tests:** `crates/atlas-runtime/tests/gzip_tests.rs` (~400 lines)

## Dependencies
- flate2 = "1.0" (gzip compression)
- File I/O stdlib (phase-05)
- Result types

## Implementation

### Gzip Compression Functions
Compress bytes with gzipCompress. Takes byte array, returns compressed bytes. Compression level setting (0-9). Default level 6. Compress string convenience function. Stream compression for large data. Memory-efficient processing. Return compressed size.

### Gzip Decompression Functions
Decompress bytes with gzipDecompress. Takes compressed bytes, returns original data. Validate gzip magic header. Handle corrupt data errors. Decompress to string helper. Stream decompression. Memory limits for safety.

### Compression Utilities
Get compression level. Set compression level. Calculate compression ratio. Benchmark compression speed. Round-trip validation. Format detection (gzip magic bytes). Streaming API for files.

### Error Handling
Corrupt data detection. Invalid compression level errors. Memory limit exceeded. IO errors during streaming. Clear error messages with context.

## Tests (25+ tests)

**Compression tests (8):**
1. Compress byte array
2. Compress string
3. Compression levels (0, 6, 9)
4. Large data compression
5. Empty data handling
6. Stream compression
7. Compression ratio calculation
8. Invalid level error

**Decompression tests (7):**
1. Decompress to bytes
2. Decompress to string
3. Corrupt data error
4. Invalid format error
5. Stream decompression
6. Large file decompression
7. Empty compressed data

**Round-trip tests (5):**
1. Compress + decompress bytes
2. Compress + decompress string
3. Large data round-trip
4. Different compression levels
5. UTF-8 text preservation

**Integration tests (5+):**
1. Compress file to file
2. Decompress file to file
3. Streaming large files
4. Memory efficiency
5. Real-world data patterns

**Minimum test count:** 25 tests

## Integration Points
- Uses: File I/O (phase-05)
- Creates: Gzip compression utilities
- Prepares: Foundation for tar.gz (phase-14b)
- Output: Data compression API

## Acceptance
- Gzip compress/decompress works
- Compression levels configurable
- Streaming for large files
- Error handling comprehensive
- 25+ tests pass
- No clippy warnings
- cargo test passes
- Interpreter/VM parity maintained
