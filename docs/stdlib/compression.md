# Compression — Gzip, Tar, and Zip

Namespaces: `Gzip`, `Tar`, `Zip` (PascalCase, D-049)

The compression module is implemented in
`crates/atlas-runtime/src/stdlib/compression/` and provides:

- `Gzip` — gzip compression and decompression (fully implemented)
- `Tar` — tar archive creation and extraction (module exists; see status below)
- `Zip` — zip archive creation and extraction (module exists; see status below)

---

## Gzip Namespace

All `Gzip` functions work with **byte arrays** (`number[]` where each element is 0–255).
Strings are accepted as input to `Gzip.compress()` directly.

### `Gzip.compress(data: string | number[], level?: number) -> number[]`

Compress `data` using gzip. Returns the compressed bytes as a `number[]`.

- `data`: a `string` (treated as UTF-8 bytes) or a `number[]` (raw byte array, values 0–255)
- `level`: compression level `0`–`9` (default `6`). Level `0` = store only (no compression),
  level `9` = maximum compression.

```atlas
let bytes = Gzip.compress("Hello, world!", 6);
console.log(bytes.length().toString() + " compressed bytes");
```

---

### `Gzip.decompress(compressed: number[]) -> number[]`

Decompress gzip data. Takes a byte array and returns the uncompressed bytes as a
`number[]`. Validates the gzip magic header before decompressing.

```atlas
let original = Gzip.decompress(compressedBytes);
```

**Errors:**
- `IoError` if the input does not begin with the gzip magic header (`0x1f 0x8b`)
- `IoError` if decompression fails

---

### `Gzip.decompressString(compressed: number[]) -> string`

Decompress gzip data and interpret the result as a UTF-8 string.

```atlas
let text = Gzip.decompressString(compressedBytes);
console.log(text);
```

**Errors:**
- Same as `Gzip.decompress()`, plus `IoError` if the decompressed bytes are not valid UTF-8

---

### `Gzip.isGzip(data: number[]) -> bool`

Check if a byte array starts with the gzip magic header (`0x1f 0x8b`). Does not validate
the full stream — only checks the first two bytes.

```atlas
let compressed = Gzip.compress("test");
let check = Gzip.isGzip(compressed); // true
let raw = [1, 2, 3];
let check2 = Gzip.isGzip(raw); // false
```

---

### `Gzip.compressionRatio(originalSize: number, compressedSize: number) -> number`

Calculate the compression ratio as `originalSize / compressedSize`. Returns `0.0` if
`compressedSize` is zero.

```atlas
let data = "some long string to compress...";
let compressed = Gzip.compress(data);
let ratio = Gzip.compressionRatio(data.length, compressed.length);
console.log("Ratio: " + ratio.toString());
```

---

## Gzip Examples

### Compress and decompress a string

```atlas
let original = "This is the data to compress. It can be quite long.";
let bytes = Gzip.compress(original, 6);
let restored = Gzip.decompressString(bytes);
// restored == original
```

### Verify compressed data before decompressing

```atlas
fn safeDecompress(data: number[]): string {
    if !Gzip.isGzip(data) {
        return "not gzip data";
    }
    return Gzip.decompressString(data);
}
```

### Maximum vs. no compression

```atlas
let fast = Gzip.compress(bigData, 1);   // fast, larger output
let best = Gzip.compress(bigData, 9);   // slow, smallest output
let store = Gzip.compress(bigData, 0);  // no compression, just gzip wrapper
```

---

## Byte Array Convention

All gzip functions use `number[]` to represent raw bytes. Each element must be in the
range `0`–`255`. Passing values outside this range results in an `IoError`.

Atlas does not have a native `Bytes` or `Buffer` type. Byte arrays are plain `number[]`.

---

## Tar and Zip

The `Tar` and `Zip` sub-modules exist at:
- `crates/atlas-runtime/src/stdlib/compression/tar.rs`
- `crates/atlas-runtime/src/stdlib/compression/zip.rs`

These modules are compiled into the runtime. Check `docs/stdlib/index.md` and the
runtime source for current registration status — namespace dispatch keys and call
signatures are defined when the functions are registered in `stdlib/mod.rs`.

---

## Error Behavior

| Condition | Error type |
|-----------|-----------|
| Compression level > 9 | `IoError` |
| Input byte value out of range (0–255) | `IoError` |
| Missing gzip magic header in decompress | `IoError` |
| Decompression failure | `IoError` |
| Decompressed bytes not valid UTF-8 (decompressString) | `IoError` |
| Input not a byte array | `TypeError` |
| Input not a string or byte array (compress) | `TypeError` |
