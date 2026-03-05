# Compression Functions

Gzip, Tar, and Zip compression/decompression.

## Gzip Functions

### gzipCompress

```atlas
fn gzipCompress(data: string) -> Result<string, string>
```

Compresses data with Gzip.

**Parameters:**
- `data` - String to compress

**Returns:**
- `Ok(string)` - Gzip compressed data (hex encoded)
- `Err(string)` on error

### gzipDecompress

```atlas
fn gzipDecompress(compressed: string) -> Result<string, string>
```

Decompresses Gzip data.

**Parameters:**
- `compressed` - Gzip compressed data (hex encoded)

**Returns:**
- `Ok(string)` - Decompressed data
- `Err(string)` if invalid or corrupted

### gzipDecompressString

```atlas
fn gzipDecompressString(compressed: string) -> Result<string, string>
```

Decompresses Gzip data to string.

**Parameters:**
- `compressed` - Gzip compressed data

**Returns:**
- `Ok(string)` - Decompressed string
- `Err(string)` on error

### gzipIsGzip

```atlas
fn gzipIsGzip(data: string) -> bool
```

Checks if data is valid Gzip format.

**Parameters:**
- `data` - Data to check (hex encoded)

**Returns:** `bool` - True if valid Gzip magic bytes

### gzipCompressionRatio

```atlas
fn gzipCompressionRatio(original: string, compressed: string) -> number
```

Calculates compression ratio.

**Parameters:**
- `original` - Original data size
- `compressed` - Compressed data size

**Returns:** `number` - Ratio (compressed / original) as percentage

## Tar Functions

### tarCreate

```atlas
fn tarCreate(files: object) -> Result<string, string>
```

Creates Tar archive from files.

**Parameters:**
- `files` - Object mapping filenames to contents

**Returns:**
- `Ok(string)` - Tar data (hex encoded)
- `Err(string)` on error

**Example:**
```atlas
let archive = tarCreate({
  "file1.txt": "content1",
  "file2.txt": "content2"
})?;
```

### tarCreateGz

```atlas
fn tarCreateGz(files: object) -> Result<string, string>
```

Creates Tar.Gz (compressed Tar) archive.

**Parameters:**
- `files` - Object mapping filenames to contents

**Returns:**
- `Ok(string)` - Tar.Gz data (hex encoded)
- `Err(string)` on error

### tarExtract

```atlas
fn tarExtract(data: string) -> Result<object, string>
```

Extracts files from Tar archive.

**Parameters:**
- `data` - Tar data (hex encoded)

**Returns:**
- `Ok(object)` - Object mapping filenames to contents
- `Err(string)` if invalid or corrupted

### tarExtractGz

```atlas
fn tarExtractGz(data: string) -> Result<object, string>
```

Extracts files from Tar.Gz archive.

**Parameters:**
- `data` - Tar.Gz data (hex encoded)

**Returns:**
- `Ok(object)` - Object mapping filenames to contents
- `Err(string)` on error

### tarList

```atlas
fn tarList(data: string) -> Result<string[], string>
```

Lists files in Tar archive without extracting.

**Parameters:**
- `data` - Tar data (hex encoded)

**Returns:**
- `Ok(string[])` - Array of filenames
- `Err(string)` on error

### tarContains

```atlas
fn tarContains(data: string, filename: string) -> Result<bool, string>
```

Checks if file exists in Tar archive.

**Parameters:**
- `data` - Tar data (hex encoded)
- `filename` - Filename to check

**Returns:**
- `Ok(bool)` - True if file exists
- `Err(string)` on error

## Zip Functions

### zipCreate

```atlas
fn zipCreate(files: object) -> Result<string, string>
```

Creates Zip archive from files.

**Parameters:**
- `files` - Object mapping filenames to contents

**Returns:**
- `Ok(string)` - Zip data (hex encoded)
- `Err(string)` on error

### zipCreateWithComment

```atlas
fn zipCreateWithComment(files: object, comment: string) -> Result<string, string>
```

Creates Zip archive with comment.

**Parameters:**
- `files` - Object mapping filenames to contents
- `comment` - Archive comment string

**Returns:**
- `Ok(string)` - Zip data (hex encoded)
- `Err(string)` on error

### zipExtract

```atlas
fn zipExtract(data: string) -> Result<object, string>
```

Extracts all files from Zip archive.

**Parameters:**
- `data` - Zip data (hex encoded)

**Returns:**
- `Ok(object)` - Object mapping filenames to contents
- `Err(string)` if invalid or corrupted

### zipExtractFiles

```atlas
fn zipExtractFiles(data: string, filenames: string[]) -> Result<object, string>
```

Extracts specific files from Zip archive.

**Parameters:**
- `data` - Zip data (hex encoded)
- `filenames` - Array of files to extract

**Returns:**
- `Ok(object)` - Object with extracted files
- `Err(string)` on error

### zipList

```atlas
fn zipList(data: string) -> Result<string[], string>
```

Lists files in Zip archive without extracting.

**Parameters:**
- `data` - Zip data (hex encoded)

**Returns:**
- `Ok(string[])` - Array of filenames
- `Err(string)` on error

### zipContains

```atlas
fn zipContains(data: string, filename: string) -> Result<bool, string>
```

Checks if file exists in Zip archive.

**Parameters:**
- `data` - Zip data (hex encoded)
- `filename` - Filename to check

**Returns:**
- `Ok(bool)` - True if file exists
- `Err(string)` on error

### zipComment

```atlas
fn zipComment(data: string) -> Result<string?, string>
```

Gets comment from Zip archive.

**Parameters:**
- `data` - Zip data (hex encoded)

**Returns:**
- `Ok(string?)` - Comment or None if no comment
- `Err(string)` on error

### zipValidate

```atlas
fn zipValidate(data: string) -> Result<bool, string>
```

Validates Zip archive integrity.

**Parameters:**
- `data` - Zip data (hex encoded)

**Returns:**
- `Ok(bool)` - True if valid
- `Err(string)` if invalid or corrupted

### zipCompressionRatio

```atlas
fn zipCompressionRatio(original: number, compressed: number) -> number
```

Calculates compression ratio.

**Parameters:**
- `original` - Original data size
- `compressed` - Compressed data size

**Returns:** `number` - Ratio as percentage

## Example Usage

```atlas
// Gzip
let compressed = gzipCompress("Hello World")?;
let decompressed = gzipDecompress(compressed)?;
print(decompressed); // "Hello World"

// Tar
let files = {
  "readme.txt": "Hello",
  "data.json": "{}"
};
let archive = tarCreate(files)?;
let extracted = tarExtract(archive)?;

// Zip
let archive = zipCreate(files)?;
let list = zipList(archive)?;
print(list); // ["readme.txt", "data.json"]
```
