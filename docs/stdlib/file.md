# File System Functions

File I/O, directory operations, and path utilities.

## File Operations

### readFile

```atlas
fn readFile(path: string) -> Result<string, string>
```

Reads entire file as string.

**Parameters:**
- `path` - File path

**Returns:**
- `Ok(string)` - File contents
- `Err(string)` - If file not found or read fails

### writeFile

```atlas
fn writeFile(path: string, content: string) -> Result<Null, string>
```

Writes string to file, creating or overwriting.

**Parameters:**
- `path` - File path
- `content` - Content to write

**Returns:**
- `Ok(Null)` on success
- `Err(string)` on failure

### appendFile

```atlas
fn appendFile(path: string, content: string) -> Result<Null, string>
```

Appends string to end of file.

**Parameters:**
- `path` - File path
- `content` - Content to append

**Returns:**
- `Ok(Null)` on success
- `Err(string)` on failure

### fileExists

```atlas
fn fileExists(path: string) -> bool
```

Checks if file exists.

**Parameters:**
- `path` - File path

**Returns:** `bool`

### removeFile

```atlas
fn removeFile(path: string) -> Result<Null, string>
```

Deletes a file.

**Parameters:**
- `path` - File path

**Returns:**
- `Ok(Null)` on success
- `Err(string)` if file not found or deletion fails

## Directory Operations

### readDir

```atlas
fn readDir(path: string) -> Result<string[], string>
```

Lists files and directories in a directory.

**Parameters:**
- `path` - Directory path

**Returns:**
- `Ok(string[])` - Array of entry names
- `Err(string)` if directory not found

### createDir

```atlas
fn createDir(path: string) -> Result<Null, string>
```

Creates a directory.

**Parameters:**
- `path` - Directory path

**Returns:**
- `Ok(Null)` on success
- `Err(string)` on failure (including if parent doesn't exist)

**Note:** Parent directories must exist. Use `fsM kdirp` for recursive creation.

### removeDir

```atlas
fn removeDir(path: string) -> Result<Null, string>
```

Removes an empty directory.

**Parameters:**
- `path` - Directory path

**Returns:**
- `Ok(Null)` on success
- `Err(string)` if not empty or doesn't exist

## File Metadata

### fileInfo

```atlas
fn fileInfo(path: string) -> Result<object, string>
```

Gets file metadata.

**Parameters:**
- `path` - File path

**Returns:**
- `Ok(object)` with fields:
  - `size: number` - File size in bytes
  - `mtime: number` - Last modified timestamp
  - `ctime: number` - Creation/change timestamp
  - `atime: number` - Last accessed timestamp
  - `isDir: bool` - Is directory
  - `isFile: bool` - Is file
  - `isSymlink: bool` - Is symbolic link
- `Err(string)` if file not found

### pathJoin

```atlas
fn pathJoin(paths: string[]) -> string
```

Joins path components into a single path.

**Parameters:**
- `paths` - Array of path segments

**Returns:** `string` - Joined path

**Note:** Normalizes path separators for the platform

## Advanced File System Functions

### fsWalk

```atlas
fn fsWalk(path: string) -> Result<string[], string>
```

Recursively lists all files and directories.

**Parameters:**
- `path` - Root directory

**Returns:**
- `Ok(string[])` - Sorted array of all paths
- `Err(string)` on error

### fsReaddir

```atlas
fn fsReaddir(path: string) -> Result<string[], string>
```

Lists entries in directory (non-recursive).

**Parameters:**
- `path` - Directory path

**Returns:**
- `Ok(string[])` - Array of entry names
- `Err(string)` on error

### fsMkdir

```atlas
fn fsMkdir(path: string) -> Result<Null, string>
```

Creates a directory.

**Parameters:**
- `path` - Directory path

**Returns:**
- `Ok(Null)` on success
- `Err(string)` if parent doesn't exist

### fsMkdirp

```atlas
fn fsMkdirp(path: string) -> Result<Null, string>
```

Creates directory and parent directories if needed.

**Parameters:**
- `path` - Directory path

**Returns:**
- `Ok(Null)` on success
- `Err(string)` on failure

### fsRmdir

```atlas
fn fsRmdir(path: string) -> Result<Null, string>
```

Removes empty directory.

**Parameters:**
- `path` - Directory path

**Returns:**
- `Ok(Null)` on success
- `Err(string)` if not empty or doesn't exist

### fsRmdirRecursive

```atlas
fn fsRmdirRecursive(path: string) -> Result<Null, string>
```

Recursively removes directory and all contents.

**Parameters:**
- `path` - Directory path

**Returns:**
- `Ok(Null)` on success
- `Err(string)` on failure

### fsSize

```atlas
fn fsSize(path: string) -> Result<number, string>
```

Gets file size in bytes.

**Parameters:**
- `path` - File path

**Returns:**
- `Ok(number)` - Size in bytes
- `Err(string)` if file not found

### fsMtime

```atlas
fn fsMtime(path: string) -> Result<number, string>
```

Gets last modified time as Unix timestamp.

**Parameters:**
- `path` - File path

**Returns:**
- `Ok(number)` - Seconds since epoch
- `Err(string)` on error

### fsCtime

```atlas
fn fsCtime(path: string) -> Result<number, string>
```

Gets creation/change time as Unix timestamp.

**Parameters:**
- `path` - File path

**Returns:**
- `Ok(number)` - Seconds since epoch
- `Err(string)` on error

### fsAtime

```atlas
fn fsAtime(path: string) -> Result<number, string>
```

Gets last accessed time as Unix timestamp.

**Parameters:**
- `path` - File path

**Returns:**
- `Ok(number)` - Seconds since epoch
- `Err(string)` on error

### fsIsDir

```atlas
fn fsIsDir(path: string) -> bool
```

Checks if path is a directory.

**Parameters:**
- `path` - Path to check

**Returns:** `bool`

### fsIsFile

```atlas
fn fsIsFile(path: string) -> bool
```

Checks if path is a regular file.

**Parameters:**
- `path` - Path to check

**Returns:** `bool`

### fsIsSymlink

```atlas
fn fsIsSymlink(path: string) -> Result<bool, string>
```

Checks if path is a symbolic link.

**Parameters:**
- `path` - Path to check

**Returns:**
- `Ok(bool)` - True if symlink
- `Err(string)` on error

### fsPermissions

```atlas
fn fsPermissions(path: string) -> Result<number, string>
```

Gets file permissions as octal number.

**Parameters:**
- `path` - File path

**Returns:**
- `Ok(number)` - Permissions (e.g., 0o644)
- `Err(string)` on error

### fsSymlink

```atlas
fn fsSymlink(target: string, link: string) -> Result<Null, string>
```

Creates a symbolic link.

**Parameters:**
- `target` - Target path
- `link` - Link path to create

**Returns:**
- `Ok(Null)` on success
- `Err(string)` on failure

### fsReadlink

```atlas
fn fsReadlink(path: string) -> Result<string, string>
```

Reads symbolic link target.

**Parameters:**
- `path` - Symlink path

**Returns:**
- `Ok(string)` - Target path
- `Err(string)` if not a symlink

### fsTmpfile

```atlas
fn fsTmpfile() -> Result<string, string>
```

Creates temporary file and returns its path.

**Returns:**
- `Ok(string)` - Temporary file path
- `Err(string)` on failure

**Note:** File is deleted on program exit

### fsTmpdir

```atlas
fn fsTmpdir() -> Result<string, string>
```

Returns system temporary directory path.

**Returns:**
- `Ok(string)` - Temp directory path
- `Err(string)` if unavailable

### fsTmpfileNamed

```atlas
fn fsTmpfileNamed(prefix: string) -> Result<string, string>
```

Creates temporary file with name prefix.

**Parameters:**
- `prefix` - File name prefix

**Returns:**
- `Ok(string)` - Temporary file path
- `Err(string)` on failure
