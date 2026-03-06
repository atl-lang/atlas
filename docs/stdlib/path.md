# Path Functions

Cross-platform path manipulation. All functions use OS-appropriate separators.

**Source:** `crates/atlas-runtime/src/stdlib/mod.rs` (path section)

## Path Construction & Parsing

| Function | Signature | Description |
|----------|-----------|-------------|
| `pathJoin(parts...)` | `(string...) -> string` | Join path segments with OS separator |
| `pathJoinArray(parts)` | `(string[]) -> string` | Join array of segments |
| `pathParse(path)` | `(string) -> record` | Parse into `{ dir, base, ext, name }` |
| `pathNormalize(path)` | `(string) -> string` | Resolve `.` and `..`, normalize separators |

## Path Components

| Function | Signature | Description |
|----------|-----------|-------------|
| `pathBasename(path)` | `(string) -> string` | File name with extension |
| `pathDirname(path)` | `(string) -> string` | Directory portion |
| `pathExtension(path)` | `(string) -> string` | File extension (with dot) |
| `pathParent(path)` | `(string) -> string` | Parent directory |
| `pathDrive(path)` | `(string) -> string` | Drive letter (Windows) or empty |

## Path Resolution

| Function | Signature | Description |
|----------|-----------|-------------|
| `pathAbsolute(path)` | `(string) -> string` | Convert to absolute path |
| `pathRelative(from, to)` | `(string, string) -> string` | Relative path between two paths |
| `pathCanonical(path)` | `(string) -> string` | Resolve symlinks + normalize |

## Path Testing

| Function | Signature | Description |
|----------|-----------|-------------|
| `pathIsAbsolute(path)` | `(string) -> bool` | Is absolute path? |
| `pathIsRelative(path)` | `(string) -> bool` | Is relative path? |
| `pathExists(path)` | `(string) -> bool` | Does path exist on disk? |
| `pathEquals(a, b)` | `(string, string) -> bool` | Are paths equivalent? |

## System Paths

| Function | Signature | Description |
|----------|-----------|-------------|
| `pathHomedir()` | `() -> string` | User home directory |
| `pathCwd()` | `() -> string` | Current working directory |
| `pathTempdir()` | `() -> string` | System temp directory |

## Platform

| Function | Signature | Description |
|----------|-----------|-------------|
| `pathSeparator()` | `() -> string` | OS path separator (`/` or `\`) |
| `pathDelimiter()` | `() -> string` | OS path list delimiter (`:` or `;`) |
| `pathExtSeparator()` | `() -> string` | Extension separator (`.`) |
| `pathToPlatform(path)` | `(string) -> string` | Convert to OS-native separators |
| `pathToPosix(path)` | `(string) -> string` | Convert to forward slashes |
| `pathToWindows(path)` | `(string) -> string` | Convert to backslashes |

**Count:** 24 functions
