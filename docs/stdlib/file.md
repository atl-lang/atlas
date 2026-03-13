# file namespace

File system operations: directory management, file metadata, symlinks, temp files, and file watching.

Basic read/write/append/delete/exists operations live in the `io` namespace (see `io.md`). This namespace covers everything else.

---

## Directory Operations

### fs.mkdir

```atlas
fs.mkdir(path: string): Result<null, string>
```

Create a single directory. Fails if the parent does not exist or the directory already exists.

```atlas
match fs.mkdir("/tmp/myapp") {
    Ok(_) => console.log("created"),
    Err(e) => console.log("failed: " + e),
}
```

### fs.mkdirp

```atlas
fs.mkdirp(path: string): Result<null, string>
```

Create a directory and all required parent directories (equivalent to `mkdir -p`). Succeeds silently if the directory already exists.

```atlas
match fs.mkdirp("/tmp/myapp/data/cache") {
    Ok(_) => console.log("ready"),
    Err(e) => console.log(e),
}
```

### fs.rmdir

```atlas
fs.rmdir(path: string): null
```

Remove an empty directory. Throws if the directory is not empty or does not exist.

```atlas
fs.rmdir("/tmp/myapp/empty-dir");
```

### fs.rmdirRecursive

```atlas
fs.rmdirRecursive(path: string): null
```

Remove a directory and all its contents recursively (equivalent to `rm -rf`). Use with caution.

```atlas
fs.rmdirRecursive("/tmp/myapp/old-build");
```

### fs.readdir

```atlas
fs.readdir(path: string): string[]
```

List the names of entries in a directory (names only, not full paths). Throws on permission error or if path does not exist.

```atlas
let entries = fs.readdir("/tmp/myapp");
for name in entries {
    console.log(name);
}
```

### fs.walk

```atlas
fs.walk(path: string): string[]
```

Recursively walk a directory tree and return all file paths relative to `path`.

```atlas
let files = fs.walk("/tmp/myapp");
for file in files {
    console.log(file);  // e.g. "data/config.json"
}
```

### fs.filterEntries

```atlas
fs.filterEntries(entries: string[], pattern: string): string[]
```

Filter an array of file names by a glob pattern. Supports `*` as a wildcard.

```atlas
let all = fs.readdir("/tmp/myapp/logs");
let logs = fs.filterEntries(all, "*.log");
```

### fs.sortEntries

```atlas
fs.sortEntries(entries: string[]): string[]
```

Sort an array of file names alphabetically (case-insensitive).

```atlas
let sorted = fs.sortEntries(fs.readdir("/tmp/myapp"));
```

---

## File Metadata

### fs.size

```atlas
fs.size(path: string): number
```

Return the file size in bytes. Throws if the path does not exist.

```atlas
let bytes = fs.size("/var/log/app.log");
console.log(bytes.toString() + " bytes");
```

### fs.mtime

```atlas
fs.mtime(path: string): number
```

Return the last-modified time as a Unix timestamp (seconds since epoch, fractional).

```atlas
let modified = fs.mtime("config.json");
```

### fs.ctime

```atlas
fs.ctime(path: string): number
```

Return the creation time as a Unix timestamp.

### fs.atime

```atlas
fs.atime(path: string): number
```

Return the last-access time as a Unix timestamp.

### fs.permissions

```atlas
fs.permissions(path: string): number
```

Return the Unix file mode bits (e.g. `493` = `0o755`). On non-Unix platforms returns `0o444` for read-only files or `0o666` for read-write files.

```atlas
let mode = fs.permissions("script.sh");
console.log(mode.toString());  // 493
```

### fs.isDir

```atlas
fs.isDir(path: string): bool
```

Return `true` if `path` exists and is a directory.

### fs.isFile

```atlas
fs.isFile(path: string): bool
```

Return `true` if `path` exists and is a regular file.

### fs.isSymlink

```atlas
fs.isSymlink(path: string): bool
```

Return `true` if `path` is a symbolic link.

### fs.inode

```atlas
fs.inode(path: string): number
```

Return the inode number. Unix only — throws on other platforms.

---

## Temporary Files and Directories

Temporary files and directories are **not** automatically deleted when the program exits. You must clean them up explicitly.

### fs.tmpfile

```atlas
fs.tmpfile(): string
```

Create a new empty temporary file and return its path.

```atlas
let tmp = fs.tmpfile();
// write to tmp, then clean up
fs.rmdir(tmp);
```

### fs.tmpfileNamed

```atlas
fs.tmpfileNamed(prefix: string): string
```

Create a temporary file with the given name prefix.

```atlas
let tmp = fs.tmpfileNamed("upload_");
// e.g. /tmp/upload_1710000000000.tmp
```

### fs.tmpdir

```atlas
fs.tmpdir(): string
```

Create a new temporary directory and return its path.

```atlas
let dir = fs.tmpdir();
```

### fs.getTempDir

```atlas
fs.getTempDir(): string
```

Return the system's temporary directory path (e.g. `/tmp` on Unix).

```atlas
let tmp = fs.getTempDir();
console.log(tmp);
```

---

## Symlink Operations

### fs.symlink

```atlas
fs.symlink(target: string, link: string): null
```

Create a symbolic link at `link` pointing to `target`. On Windows, requires elevated privileges.

```atlas
fs.symlink("/usr/local/bin/atlas", "/usr/bin/atlas");
```

### fs.readlink

```atlas
fs.readlink(path: string): string
```

Return the path that the symlink at `path` points to (one hop, not fully resolved).

```atlas
let target = fs.readlink("/usr/bin/atlas");
```

### fs.resolveSymlink

```atlas
fs.resolveSymlink(path: string): string
```

Follow the entire symlink chain and return the final canonical absolute path.

```atlas
let real = fs.resolveSymlink("/usr/bin/python");
```

---

## Async File Operations

Async variants return `Future<T>`. Use `await` to get the result.

### readFileAsync

```atlas
readFileAsync(path: string): Future<string>
```

Read a file asynchronously. Requires read permission on the resolved path.

```atlas
let content = await readFileAsync("data.json");
```

### writeFileAsync

```atlas
writeFileAsync(path: string, content: string): Future<null>
```

Write content to a file asynchronously. Creates parent directories if needed. Requires write permission.

```atlas
await writeFileAsync("output.txt", "hello\n");
```

### appendFileAsync

```atlas
appendFileAsync(path: string, content: string): Future<null>
```

Append to a file asynchronously. Creates the file if it does not exist.

```atlas
await appendFileAsync("log.txt", "entry\n");
```

### file.renameAsync

```atlas
file.renameAsync(src: string, dst: string): Future<null>
```

Move/rename a file asynchronously. Requires write permission on both paths.

```atlas
await file.renameAsync("old.txt", "new.txt");
```

### file.copyAsync

```atlas
file.copyAsync(src: string, dst: string): Future<null>
```

Copy a file asynchronously. Requires read on source, write on destination.

```atlas
await file.copyAsync("template.json", "config.json");
```

---

## File Watching

### fsWatch

```atlas
fsWatch(path: string): Watcher
```

Watch a file or directory for changes. Directories are watched recursively. Throws if `path` does not exist.

```atlas
let watcher = fsWatch("./src");
```

### fsWatchNext

```atlas
fsWatchNext(watcher: Watcher): Future<object>
```

Await the next change event from a watcher. The resolved value is:

```
{
    kind: string,   // "create" | "modify" | "remove" | "access" | "any" | "other"
    detail: string, // detailed event description
    paths: string[] // affected file paths
}
```

```atlas
let watcher = fsWatch("config.json");
while true {
    let event = await fsWatchNext(watcher);
    if event.kind == "modify" {
        console.log("changed: " + event.paths[0]);
        reload();
    }
}
```
