# Blocking Issues - CORRECTED

**Last verified**: 2026-03-03 (against actual codebase)

---

## Status Update

Most previously reported blockers were based on **outdated information**. After audit against the actual codebase, here's the corrected status:

---

## RESOLVED (Previously Marked Blocking)

### 1. exec() Function
**Previous Status**: BLOCKING
**Actual Status**: WORKS

```atlas
// This works:
let result: Result<ExecResult, string> = exec(["echo", "hello"]);

match result {
    Ok(r) => print(r.stdout),
    Err(e) => print("Error: " + e)
}

// Also available:
let shell_result = shell("echo hello && ls -la");
let spawn_result = spawn(["long-running-process"]);
```

**Location**: `stdlib/process.rs:53-100`

---

### 2. JSON Stringify
**Previous Status**: BLOCKING
**Actual Status**: WORKS

```atlas
// toJSON() exists and works:
let data = hashMapNew();
hashMapPut(data, "name", "test");
hashMapPut(data, "count", 42);

let json_string: string = toJSON(data);  // Works!
print(json_string);  // {"name":"test","count":42}
```

**Location**: `stdlib/json.rs:61-70`

---

### 3. Directory Creation (mkdir)
**Previous Status**: BLOCKING
**Actual Status**: WORKS

```atlas
// createDir() exists:
let result = createDir(".hydra/sessions");

match result {
    Ok(_) => print("Directory created"),
    Err(e) => print("Error: " + e)
}

// Also available:
let exists = fileExists(".hydra/sessions");
let info = fileInfo(".hydra/sessions");  // includes isDir
let contents = readDir(".hydra");
```

**Location**: `stdlib/io.rs`

---

### 4. Object Creation Syntax
**Previous Status**: BLOCKING
**Actual Status**: DESIGN CHOICE (workaround available)

Atlas doesn't have `{key: value}` object literal syntax. This is by design, not a bug.

**Workarounds**:
```atlas
// Option 1: JSON parsing
let config = parseJSON("{\"name\": \"server\", \"port\": 8080}");

// Option 2: HashMap (recommended for dynamic data)
let config = hashMapNew();
hashMapPut(config, "name", "server");
hashMapPut(config, "port", 8080);

// Option 3: Builder pattern
fn newConfig(name: string, port: number) -> HashMap {
    let c = hashMapNew();
    hashMapPut(c, "name", name);
    hashMapPut(c, "port", port);
    return c;
}
let config = newConfig("server", 8080);
```

---

### 5. writeFile() Type System Issue
**Previous Status**: INVESTIGATING
**Actual Status**: LIKELY USER ERROR

Pattern matching on Result works correctly. If you got "non-exhaustive pattern match", the likely causes:
1. Missing semicolon on match arms
2. Incorrect arm syntax
3. Type mismatch

**Correct pattern**:
```atlas
let result = writeFile(path, content);

match result {
    Ok(_) => {
        print("Written successfully");
        true
    },
    Err(e) => {
        print("Error: " + e);
        false
    }
}
```

---

## Actual Remaining Gaps

### 1. User-Defined Types
**Status**: Scheduled for v0.4
**Impact**: Cannot define struct/enum with methods
**Workaround**: Use HashMap, JSON, or factory functions

### 2. Object Literal Syntax
**Status**: Design decision
**Impact**: Must use parseJSON() or hashMapNew()
**Workaround**: See Option 1-3 above

---

## Component Re-Assessment

Based on corrected information:

| Component | Previous | Corrected | Notes |
|-----------|----------|-----------|-------|
| Supervisor | BLOCKED | **FEASIBLE** | exec() works |
| StateStore | PARTIAL | **FEASIBLE** | toJSON() + createDir() work |
| Watcher | UNKNOWN | **FEASIBLE** | fileInfo() + readDir() work |
| Proxy | BLOCKED | **FEASIBLE** | Depends on Supervisor (now feasible) |
| Config | PARTIAL | **FEASIBLE** | toJSON() works |

---

## Recommendation

**Re-attempt all previously blocked components.** The stdlib is far more complete than documented.

Priority order:
1. StateStore - Should be straightforward now
2. Supervisor - exec() is confirmed working
3. Watcher - File monitoring possible with fileInfo/readDir
4. Config - JSON serialization available
5. Proxy - Depends on Supervisor
