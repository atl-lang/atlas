# Compression Functions

Gzip, Tar, and Zip compression/decompression.

## Gzip Functions

### gzipCompress

```atlas
fn gzipCompress(data: string | number[], level?: number) : number[]
```

Compresses data with Gzip.

**Parameters:**
- `data` - String or byte array (0-255) to compress
- `level` - Compression level 0-9 (default 6)

**Returns:** `number[]` - Gzip-compressed bytes

### gzipDecompress

```atlas
fn gzipDecompress(compressed: number[]) : number[]
```

Decompresses Gzip data to bytes.

**Parameters:**
- `compressed` - Gzip-compressed bytes

**Returns:** `number[]` - Decompressed bytes

### gzipDecompressString

```atlas
fn gzipDecompressString(compressed: number[]) : string
```

Decompresses Gzip data to UTF-8 string.

**Parameters:**
- `compressed` - Gzip-compressed bytes

**Returns:** `string` - Decompressed string

### gzipIsGzip

```atlas
fn gzipIsGzip(data: number[]) : bool
```

Checks if data is valid Gzip format.

**Parameters:**
- `data` - Data to check (byte array)

**Returns:** `bool` - True if valid Gzip magic bytes

### gzipCompressionRatio

```atlas
fn gzipCompressionRatio(original: number, compressed: number) : number
```

Calculates compression ratio.

**Parameters:**
- `original` - Original data size in bytes
- `compressed` - Compressed data size in bytes

**Returns:** `number` - Ratio (original / compressed)

## Tar Functions

### tarCreate

```atlas
fn tarCreate(sources: string[], output: string) : Null
```

Creates Tar archive from files and directories.

**Parameters:**
- `sources` - Array of file or directory paths to include
- `output` - Output `.tar` file path

**Returns:** `Null`

**Example:**
```atlas
tarCreate(["/tmp/file1.txt", "/tmp/file2.txt"], "/tmp/archive.tar");
```

### tarCreateGz

```atlas
fn tarCreateGz(sources: string[], output: string, level?: number) : Null
```

Creates Tar.Gz (compressed Tar) archive.

**Parameters:**
- `sources` - Array of file or directory paths to include
- `output` - Output `.tar.gz` file path
- `level` - Compression level 0-9 (default 6)

**Returns:** `Null`

### tarExtract

```atlas
fn tarExtract(tarPath: string, outputDir: string) : string[]
```

Extracts files from Tar archive.

**Parameters:**
- `tarPath` - Path to `.tar` file
- `outputDir` - Directory to extract into

**Returns:** `string[]` - Extracted file paths

### tarExtractGz

```atlas
fn tarExtractGz(tarGzPath: string, outputDir: string) : string[]
```

Extracts files from Tar.Gz archive.

**Parameters:**
- `tarGzPath` - Path to `.tar.gz` file
- `outputDir` - Directory to extract into

**Returns:** `string[]` - Extracted file paths

### tarList

```atlas
fn tarList(tarPath: string) : object[]
```

Lists files in Tar archive without extracting.

**Parameters:**
- `tarPath` - Path to `.tar` file

**Returns:** `object[]` - Entries with `path`, `size`, and `type`

### tarContains

```atlas
fn tarContains(tarPath: string, filePath: string) : bool
```

Checks if file exists in Tar archive.

**Parameters:**
- `tarPath` - Path to `.tar` file
- `filePath` - Path inside the archive to check

**Returns:** `bool` - True if file exists

## Zip Functions

### zipCreate

```atlas
fn zipCreate(sources: string[], output: string, level?: number) : Null
```

Creates Zip archive from files and directories.

**Parameters:**
- `sources` - Array of file or directory paths to include
- `output` - Output `.zip` file path
- `level` - Compression level 0-9 (default 6; 0 = store)

**Returns:** `Null`

### zipCreateWithComment

```atlas
fn zipCreateWithComment(sources: string[], output: string, comment: string, level?: number) : Null
```

Creates Zip archive with comment.

**Parameters:**
- `sources` - Array of file or directory paths to include
- `output` - Output `.zip` file path
- `comment` - Archive comment string
- `level` - Compression level 0-9 (default 6; 0 = store)

**Returns:** `Null`

### zipExtract

```atlas
fn zipExtract(zipPath: string, outputDir: string) : string[]
```

Extracts all files from Zip archive.

**Parameters:**
- `zipPath` - Path to `.zip` file
- `outputDir` - Directory to extract into

**Returns:** `string[]` - Extracted file paths

### zipExtractFiles

```atlas
fn zipExtractFiles(zipPath: string, outputDir: string, files: string[]) : string[]
```

Extracts specific files from Zip archive.

**Parameters:**
- `zipPath` - Path to `.zip` file
- `outputDir` - Directory to extract into
- `files` - Array of entry names to extract

**Returns:** `string[]` - Extracted file paths

### zipList

```atlas
fn zipList(zipPath: string) : object[]
```

Lists files in Zip archive without extracting.

**Parameters:**
- `zipPath` - Path to `.zip` file

**Returns:** `object[]` - Entries with `name`, `size`, `compressedSize`, `isDir`, `method`

### zipContains

```atlas
fn zipContains(zipPath: string, entryName: string) : bool
```

Checks if entry exists in Zip archive.

**Parameters:**
- `zipPath` - Path to `.zip` file
- `entryName` - Entry name to check

**Returns:** `bool` - True if entry exists

### zipComment

```atlas
fn zipComment(zipPath: string) : string
```

Gets comment from Zip archive.

**Parameters:**
- `zipPath` - Path to `.zip` file

**Returns:** `string` - Comment (empty if none)

### zipValidate

```atlas
fn zipValidate(zipPath: string) : bool
```

Validates Zip archive integrity.

**Parameters:**
- `zipPath` - Path to `.zip` file

**Returns:** `bool` - True if valid

### zipCompressionRatio

```atlas
fn zipCompressionRatio(zipPath: string) : number
```

Calculates compression ratio.

**Parameters:**
- `zipPath` - Path to `.zip` file

**Returns:** `number` - Ratio (compressed / original)

## Example Usage

```atlas
// Gzip
let compressed = gzipCompress("Hello World");
let decompressed = gzipDecompressString(compressed);
print(decompressed); // "Hello World"

// Tar
let sources = ["/tmp/readme.txt", "/tmp/data.json"];
tarCreate(sources, "/tmp/archive.tar");
let extracted = tarExtract("/tmp/archive.tar", "/tmp/extracted");

// Zip
zipCreate(sources, "/tmp/archive.zip");
let list = zipList("/tmp/archive.zip");
print(list); // [{name: "...", size: 123, compressedSize: 45, isDir: false, method: "deflated"}, ...]
```
