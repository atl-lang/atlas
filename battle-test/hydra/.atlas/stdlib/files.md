# File I/O

## ✅ Available Functions

### writeFile()
Write string to file:

```atlas
let content: string = "Hello, file!";
let result: Result<null, string> = writeFile("/path/to/file.txt", content);

match result {
    Ok(_) => print("Written"),
    Err(e) => print("Error: " + e)
}
```

**Signature**: `writeFile(path: string, content: string) -> Result<null, string>`

**Behavior**:
- Creates file if doesn't exist
- Overwrites if exists
- **Requires parent directory to exist** (doesn't create dirs)
- Returns `Ok(null)` on success

### readFile()
Read file as string:

```atlas
let result: Result<string, string> = readFile("/path/to/file.txt");

let content: string = match result {
    Ok(data) => data,
    Err(e) => ""
};
```

**Signature**: `readFile(path: string) -> Result<string, string>`

**Behavior**:
- Returns `Ok(content)` if file exists
- Returns `Err(...)` if file doesn't exist or can't be read
- Reads entire file as string

## ⚠️ Known Issues

### Parent Directory Must Exist

```atlas
// This will FAIL if .hydra/sessions/ doesn't exist:
writeFile(".hydra/sessions/state.json", "{}");

// Error: "Failed to resolve parent path: No such file or directory"
```

**Workaround**: Create directories first (outside Atlas, using shell):
```bash
mkdir -p .hydra/sessions
```

### No Directory Creation

Atlas doesn't appear to have `mkdir()` or `createDir()` function.

### Pattern Matching Quirks

```atlas
// This sometimes causes "non-exhaustive pattern match":
let result = writeFile(path, content);
match result {
    Ok(_) => true,
    Err(e) => false
}
```

**Status**: Investigating type system behavior

## Working Pattern

```atlas
fn save_data(path: string, content: string) -> bool {
    let write_result: Result<null, string> = writeFile(path, content);

    let success: bool = match write_result {
        Ok(_) => true,
        Err(_) => false
    };

    return success;
}

fn load_data(path: string) -> Result<string, string> {
    return readFile(path);
}
```

## File Checking

Use readFile to check if file exists:

```atlas
fn file_exists(path: string) -> bool {
    let result: Result<string, string> = readFile(path);

    let exists: bool = match result {
        Ok(_) => true,
        Err(_) => false
    };

    return exists;
}
```

## Summary

- **Reading**: ✅ `readFile()` works
- **Writing**: ⚠️ Works but requires parent directories to exist
- **Directories**: ❌ No mkdir/createDir function found
- **Workaround**: Create directories externally before running Atlas code
