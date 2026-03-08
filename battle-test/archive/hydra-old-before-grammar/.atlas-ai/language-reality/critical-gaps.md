# Atlas v0.2 - Actual State

**Verified against codebase**: 2026-03-03
**Last update**: Object literals, struct expressions, HashMap serialization added

---

## Summary

Atlas v0.2 is **feature-complete for most use cases**. The stdlib has 439 registered functions.

---

## Actual Gaps (Only 1 Confirmed)

### 1. Async Process Spawning
**Status**: Confirmed gap
**Impact**: Cannot spawn background processes, only synchronous exec()
**Workaround**: None for daemon management; use exec() for short-lived commands

```atlas
// exec() works but blocks until complete
let result = exec(["echo", "hello"]);

// Cannot spawn background processes:
// spawnProcess(["server", "--daemon"])  // NOT AVAILABLE
```

---

## RESOLVED - Object Literals & Struct Expressions (2026-03-03)

### Object Literal Syntax - NOW WORKS ✅
```atlas
// This NOW works in Atlas v0.2!
let obj = { name: "test", value: 42 };

// toJSON() also works with HashMaps now
let json_str = toJSON(obj);  // Returns: {"name":"test","value":42}
```

### Struct Expressions - NOW WORKS ✅
```atlas
// Struct expressions also work (compile to HashMap)
let user = User { name: "Alice", age: 30 };

// Access via hashMapGet
let name = unwrap(hashMapGet(user, "name"));
```

### Struct/Enum Declarations - Parser Support ✅
```atlas
// These parse correctly (no runtime methods yet)
struct Point {
    x: number,
    y: number
}

enum Color {
    Red,
    Green,
    Blue
}
```

---

## What Actually EXISTS

### JSON - FULLY IMPLEMENTED
| Function | Status |
|----------|--------|
| `toJSON(value)` | Works (including HashMap!) |
| `parseJSON(str)` | Works |

### Process Execution - PARTIAL
| Function | Status |
|----------|--------|
| `exec(args)` | Works (sync only) |
| `shell(cmd)` | Works |
| Async spawn | NOT AVAILABLE |

### File System - FULLY IMPLEMENTED
| Function | Status |
|----------|--------|
| `createDir(path)` | Works |
| `fileExists(path)` | Works |
| `removeFile(path)` | Works |
| `readDir(path)` | Works |
| `fileInfo(path)` | Works |

### Collections - FULLY IMPLEMENTED
| Feature | Status |
|---------|--------|
| Object literals `{k: v}` | Works ✅ |
| Struct expressions `T {k: v}` | Works ✅ |
| HashMap functions | Works |
| HashSet functions | Works |
| toJSON(HashMap) | Works ✅ |

### String Operations - FULLY IMPLEMENTED
| Function | Status |
|----------|--------|
| `replace(str, old, new)` | Works |
| `toLowerCase(str)` | Works |
| `toUpperCase(str)` | Works |
| `indexOf(str, substr)` | Works |
| `charAt(str, idx)` | Works |
| Plus 15+ more | Works |

### HTTP Client - FULLY IMPLEMENTED
| Function | Status |
|----------|--------|
| `httpRequest(url, opts)` | Works |
| `httpRequestGet(url)` | Works |
| `httpRequestPost(url, body)` | Works |

### Cryptography - FULLY IMPLEMENTED
| Function | Status |
|----------|--------|
| `sha256(data)` | Works |
| `sha512(data)` | Works |
| `blake3Hash(data)` | Works |
| `hmacSha256(key, data)` | Works |
| `aes256GcmEncrypt/Decrypt` | Works |
| `base64Encode/Decode` | Works |

### Networking - FULLY IMPLEMENTED
| Function | Status |
|----------|--------|
| TCP (connect, read, write, listen, accept) | Works |
| UDP | Works |
| TLS | Works |
| WebSocket | Works |

---

## Bottom Line

**Atlas v0.2 is systems-programming capable.**

| Capability | Status |
|------------|--------|
| Object literals | ✅ Works |
| Struct expressions | ✅ Works |
| HashMap serialization | ✅ Works |
| JSON operations | ✅ Works |
| Process execution | ✅ Sync only |
| File system ops | ✅ Works |
| HTTP client | ✅ Works |
| Networking | ✅ Works |
| Cryptography | ✅ Works |
| Async process spawn | ❌ Not available |

**Recommendation**: Re-evaluate blocked components - most should now be feasible.
