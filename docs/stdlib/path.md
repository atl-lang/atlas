# path namespace

Cross-platform path manipulation utilities. Equivalent in scope to Node.js's `path` module.

All functions are pure string operations unless they need to resolve against the filesystem (e.g. `path.absolute`, `path.canonical`). Platform separators are handled automatically.

---

## Construction and Parsing

### path.join

```atlas
path.join(...segments: string[]): string
```

Join path segments with the platform-specific separator. Empty segments are skipped. Returns `"."` for an empty input.

```atlas
let p = path.join("/usr", "local", "bin");
// "/usr/local/bin"

let rel = path.join("src", "utils", "helpers.atl");
// "src/utils/helpers.atl"
```

### path.parse

```atlas
path.parse(path: string): object
```

Parse a path string into its components. Returns an object with:

| Field  | Description                         | Example for `/home/user/data.json` |
|--------|-------------------------------------|------------------------------------|
| `root` | Root component                      | `"/"`                              |
| `dir`  | Directory portion                   | `"/home/user"`                     |
| `base` | Filename with extension             | `"data.json"`                      |
| `ext`  | Extension without dot               | `"json"`                           |
| `name` | Filename without extension          | `"data"`                           |

```atlas
let parts = path.parse("/home/user/data.json");
console.log(parts.dir);   // "/home/user"
console.log(parts.name);  // "data"
console.log(parts.ext);   // "json"
```

### path.normalize

```atlas
path.normalize(path: string): string
```

Remove redundant separators and resolve `.` and `..` components. Always uses forward slashes in output for cross-platform consistency.

```atlas
let clean = path.normalize("/usr/local/../bin/./atlas");
// "/usr/bin/atlas"
```

### path.absolute

```atlas
path.absolute(path: string): string
```

Resolve a relative path against the current working directory. If `path` is already absolute, returns it unchanged.

```atlas
let abs = path.absolute("src/main.atl");
// "/home/user/project/src/main.atl"
```

### path.resolve

```atlas
path.resolve(path: string): string
```

Alias for `path.absolute`. Resolves relative paths against cwd.

### path.relative

```atlas
path.relative(from: string, to: string): string
```

Compute the relative path needed to navigate from `from` to `to`. Both paths are made absolute before computing the relative path.

```atlas
let rel = path.relative("/home/user/a", "/home/user/b/file.txt");
// "../b/file.txt"
```

### path.parent

```atlas
path.parent(path: string): Option<string>
```

Return the parent directory as `Some(string)`, or `None` for root or empty paths.

```atlas
match path.parent("/usr/local/bin") {
    Some(p) => console.log(p),  // "/usr/local"
    None => console.log("no parent"),
}
```

### path.basename

```atlas
path.basename(path: string): string
```

Return the filename portion including extension.

```atlas
let name = path.basename("/home/user/data.json");
// "data.json"
```

### path.dirname

```atlas
path.dirname(path: string): string
```

Return the directory portion (everything except the filename).

```atlas
let dir = path.dirname("/home/user/data.json");
// "/home/user"
```

### path.extension

```atlas
path.extension(path: string): string
```

Return the file extension without the dot. Returns empty string if there is no extension.

```atlas
let ext = path.extension("archive.tar.gz");
// "gz"
```

---

## Validation and Comparison

### path.isAbsolute

```atlas
path.isAbsolute(path: string): bool
```

Return `true` if `path` starts from the filesystem root.

```atlas
path.isAbsolute("/usr/bin");   // true
path.isAbsolute("src/main");   // false
```

### path.isRelative

```atlas
path.isRelative(path: string): bool
```

Return `true` if `path` is not absolute.

### path.exists

```atlas
path.exists(path: string): bool
```

Return `true` if `path` exists on the filesystem (resolves symlinks).

```atlas
if path.exists("config.json") {
    // load it
}
```

### path.canonical

```atlas
path.canonical(path: string): string
```

Return the canonical absolute path, resolving all symlinks. Throws if the path does not exist.

```atlas
let real = path.canonical("./relative/../link");
```

### path.equals

```atlas
path.equals(path1: string, path2: string): bool
```

Compare two paths for equality. On Windows the comparison is case-insensitive; on Unix it is case-sensitive.

```atlas
if path.equals(a, b) {
    console.log("same path");
}
```

---

## Utilities

### path.homedir

```atlas
path.homedir(): string
```

Return the current user's home directory. Throws if it cannot be determined.

```atlas
let home = path.homedir();
let config = path.join(home, ".config", "atlas");
```

### path.cwd

```atlas
path.cwd(): string
```

Return the current working directory.

```atlas
let here = path.cwd();
```

### path.tempdir

```atlas
path.tempdir(): string
```

Return the system temporary directory path (e.g. `/tmp`).

```atlas
let tmp = path.tempdir();
```

### path.separator

```atlas
path.separator(): string
```

Return the platform-specific path separator (`"/"` on Unix, `"\\"` on Windows).

### path.delimiter

```atlas
path.delimiter(): string
```

Return the platform-specific PATH environment variable delimiter (`":"` on Unix, `";"` on Windows).

### path.drive

```atlas
path.drive(path: string): string
```

Return the drive letter component on Windows (e.g. `"C:"`). Returns empty string on non-Windows platforms or paths without a drive.

### path.toPlatform

```atlas
path.toPlatform(path: string): string
```

Convert all separators to the platform-native separator.

### path.toPosix

```atlas
path.toPosix(path: string): string
```

Convert all separators to forward slashes (POSIX style).

```atlas
let posix = path.toPosix("src\\utils\\main.atl");
// "src/utils/main.atl"
```

### path.toWindows

```atlas
path.toWindows(path: string): string
```

Convert all separators to backslashes (Windows style).

---

## Patterns

```atlas
// Build a config file path portably
fn configPath(borrow name: string): string {
    return path.join(path.homedir(), ".config", "myapp", name + ".json");
}

// Walk and filter source files
let srcDir = path.join(path.cwd(), "src");
let files = fs.walk(srcDir);
let atlasFiles = fs.filterEntries(files, "*.atl");
for file in atlasFiles {
    let abs = path.join(srcDir, file);
    let ext = path.extension(file);
    console.log(path.basename(file) + " (" + ext + ")");
}
```
