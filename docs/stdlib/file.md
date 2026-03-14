# file namespace

File system operations: directory management, file metadata, symlinks, temp files, and file watching.

Basic read/write/append/delete/exists operations live in the `io` namespace (see `io.md`). This namespace covers everything else.

---

## Directory Operations

### file.mkdir

```atlas
file.mkdir(path: string): Result<null, string>
```

Create a single directory. Fails if the parent does not exist or the directory already exists.

```atlas
match file.mkdir("/tmp/myapp") {
    Ok(_) => console.log("created"),
    Err(e) => console.log("failed: " + e),
}
```

### file.mkdirp

```atlas
file.mkdirp(path: string): Result<null, string>
```

Create a directory and all required parent directories (equivalent to `mkdir -p`). Succeeds silently if the directory already exists.

```atlas
match file.mkdirp("/tmp/myapp/data/cache") {
    Ok(_) => console.log("ready"),
    Err(e) => console.log(e),
}
```

### file.rmdir

```atlas
file.rmdir(path: string): null
```

Remove an empty directory. Throws if the directory is not empty or does not exist.

```atlas
file.rmdir("/tmp/myapp/empty-dir");
```

### file.rmdirRecursive

```atlas
file.rmdirRecursive(path: string): null
```

Remove a directory and all its contents recursively (equivalent to `rm -rf`). Use with caution.

```atlas
file.rmdirRecursive("/tmp/myapp/old-build");
```

### file.readdir

```atlas
file.readdir(path: string): string[]
```

List the names of entries in a directory (names only, not full paths). Throws on permission error or if path does not exist.

```atlas
let entries = file.readdir("/tmp/myapp");
for name in entries {
    console.log(name);
}
```

### file.walk

```atlas
file.walk(path: string): string[]
```

Recursively walk a directory tree and return all file paths relative to `path`.

```atlas
let files = file.walk("/tmp/myapp");
for file in files {
    console.log(file);  // e.g. "data/config.json"
}
```

### file.filterEntries

```atlas
file.filterEntries(entries: string[], pattern: string): string[]
```

Filter an array of file names by a glob pattern. Supports `*` as a wildcard.

```atlas
let all = file.readdir("/tmp/myapp/logs");
let logs = file.filterEntries(all, "*.log");
```

### file.sortEntries

```atlas
file.sortEntries(entries: string[]): string[]
```

Sort an array of file names alphabetically (case-insensitive).

```atlas
let sorted = file.sortEntries(file.readdir("/tmp/myapp"));
```

---

## File Metadata

### file.size

```atlas
file.size(path: string): number
```

Return the file size in bytes. Throws if the path does not exist.

```atlas
let bytes = file.size("/var/log/app.log");
console.log(bytes.toString() + " bytes");
```

### file.mtime

```atlas
file.mtime(path: string): number
```

Return the last-modified time as a Unix timestamp (seconds since epoch, fractional).

```atlas
let modified = file.mtime("config.json");
```

### file.ctime

```atlas
file.ctime(path: string): number
```

Return the creation time as a Unix timestamp.

### file.atime

```atlas
file.atime(path: string): number
```

Return the last-access time as a Unix timestamp.

### file.permissions

```atlas
file.permissions(path: string): number
```

Return the Unix file mode bits (e.g. `493` = `0o755`). On non-Unix platforms returns `0o444` for read-only files or `0o666` for read-write files.

```atlas
let mode = file.permissions("script.sh");
console.log(mode.toString());  // 493
```

### file.isDir

```atlas
file.isDir(path: string): bool
```

Return `true` if `path` exists and is a directory.

### file.isFile

```atlas
file.isFile(path: string): bool
```

Return `true` if `path` exists and is a regular file.

### file.isSymlink

```atlas
file.isSymlink(path: string): bool
```

Return `true` if `path` is a symbolic link.

### file.inode

```atlas
file.inode(path: string): number
```

Return the inode number. Unix only — throws on other platforms.

---

## Temporary Files and Directories

Temporary files and directories are **not** automatically deleted when the program exits. You must clean them up explicitly.

### file.tmpfile

```atlas
file.tmpfile(): string
```

Create a new empty temporary file and return its path.

```atlas
let tmp = file.tmpfile();
// write to tmp, then clean up
file.rmdir(tmp);
```

### file.tmpfileNamed

```atlas
file.tmpfileNamed(prefix: string): string
```

Create a temporary file with the given name prefix.

```atlas
let tmp = file.tmpfileNamed("upload_");
// e.g. /tmp/upload_1710000000000.tmp
```

### file.tmpdir

```atlas
file.tmpdir(): string
```

Create a new temporary directory and return its path.

```atlas
let dir = file.tmpdir();
```

### file.getTempDir

```atlas
file.getTempDir(): string
```

Return the system's temporary directory path (e.g. `/tmp` on Unix).

```atlas
let tmp = file.getTempDir();
console.log(tmp);
```

---

## Symlink Operations

### file.symlink

```atlas
file.symlink(target: string, link: string): null
```

Create a symbolic link at `link` pointing to `target`. On Windows, requires elevated privileges.

```atlas
file.symlink("/usr/local/bin/atlas", "/usr/bin/atlas");
```

### file.readlink

```atlas
file.readlink(path: string): string
```

Return the path that the symlink at `path` points to (one hop, not fully resolved).

```atlas
let target = file.readlink("/usr/bin/atlas");
```

### file.resolveSymlink

```atlas
file.resolveSymlink(path: string): string
```

Follow the entire symlink chain and return the final canonical absolute path.

```atlas
let real = file.resolveSymlink("/usr/bin/python");
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
