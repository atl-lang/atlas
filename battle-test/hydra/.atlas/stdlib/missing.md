# Standard Library - Corrected Status

**Last verified**: 2026-03-03 (against actual codebase)

---

## Previously "Missing" - NOW CONFIRMED WORKING

### JSON
- `toJSON(value)` - Convert any value to JSON string
- `parseJSON(str)` - Parse JSON string to value

### File System
- `createDir(path)` - Create directory (was `mkdir`)
- `fileExists(path)` - Check file existence (was `exists`)
- `removeFile(path)` - Delete file
- `removeDir(path)` - Delete directory
- `readDir(path)` - List directory contents (was `listDir`)
- `fileInfo(path)` - Get file metadata including isDir

### Process Execution
- `exec(args)` - Execute command with args array
- `shell(cmd)` - Execute shell command string
- `spawn(args)` - Spawn background process

### String Operations
- `replace(str, old, new)` - String replacement
- `toLowerCase(str)` - Convert to lowercase
- `toUpperCase(str)` - Convert to uppercase
- `charAt(str, idx)` - Get character at index
- `indexOf(str, substr)` - Find substring position
- `includes(str, substr)` - Check if contains
- `endsWith(str, suffix)` - Check suffix
- `padStart(str, len, char)` - Pad start
- `padEnd(str, len, char)` - Pad end
- `join(arr, delim)` - Join array to string

### Collections
- `hashMapNew()` - Create new HashMap
- `hashMapPut(map, key, val)` - Add/update entry
- `hashMapGet(map, key)` - Get value by key
- `hashMapRemove(map, key)` - Remove entry
- `hashMapKeys(map)` - Get all keys
- `hashMapValues(map)` - Get all values
- `hashMapSize(map)` - Get size
- `hashMapForEach(map, fn)` - Iterate
- `hashSetNew()` - Create new HashSet
- `hashSetAdd(set, val)` - Add value
- `hashSetRemove(set, val)` - Remove value
- `hashSetContains(set, val)` - Check membership
- `hashSetSize(set)` - Get size

### Arrays
- `push(arr, val)` - Add to end
- `pop(arr)` - Remove from end
- `shift(arr)` - Remove from start
- `unshift(arr, val)` - Add to start
- `reverse(arr)` - Reverse in place
- `concat(arr1, arr2)` - Concatenate
- `slice(arr, start, end)` - Slice array
- `sort(arr)` - Sort array
- `flatten(arr)` - Flatten nested arrays
- `map(arr, fn)` - Transform elements
- `filter(arr, fn)` - Filter elements
- `reduce(arr, fn, init)` - Reduce to value
- `flatMap(arr, fn)` - Map and flatten
- `forEach(arr, fn)` - Iterate
- `some(arr, fn)` - Check if any match
- `every(arr, fn)` - Check if all match
- `find(arr, fn)` - Find first match
- `findIndex(arr, fn)` - Find index of first match

### DateTime
- `dateTimeNow()` - Current timestamp
- `sleep(ms)` - Delay execution
- `dateTimeFromTimestamp(ts)` - From Unix timestamp
- `dateTimeFromComponents(y, m, d, h, m, s)` - From components
- Plus timezone support, arithmetic, formatting

### HTTP
- `httpRequest(url, opts)` - Full HTTP request
- `httpRequestGet(url)` - GET request
- `httpRequestPost(url, body)` - POST request
- `httpRequestPut(url, body)` - PUT request
- `httpRequestDelete(url)` - DELETE request
- `httpRequestPatch(url, body)` - PATCH request
- Plus header, timeout, query param builders

### Regex
- `regexNew(pattern)` - Create regex
- `regexNewWithFlags(pattern, flags)` - With flags
- `regexEscape(str)` - Escape special chars
- `regexIsMatch(re, str)` - Test match
- `regexFind(re, str)` - Find first match
- `regexFindAll(re, str)` - Find all matches
- `regexCaptures(re, str)` - Capture groups
- `regexCapturesNamed(re, str)` - Named captures

### Cryptography
- `sha256(data)` - SHA-256 hash
- `sha512(data)` - SHA-512 hash
- `blake3Hash(data)` - Blake3 hash
- `hmacSha256(key, data)` - HMAC-SHA256
- `hmacSha256Verify(key, data, sig)` - Verify HMAC
- `aes256GcmEncrypt(key, nonce, plaintext)` - AES-GCM encrypt
- `aes256GcmDecrypt(key, nonce, ciphertext)` - AES-GCM decrypt
- `randomBytes(len)` - Secure random bytes
- `randomString(len)` - Secure random string
- `base64Encode(data)` - Base64 encode
- `base64Decode(str)` - Base64 decode

### Networking
- `tcpConnect(host, port)` - TCP client connection
- `tcpWrite(conn, data)` - Write to TCP
- `tcpRead(conn)` - Read from TCP
- `tcpClose(conn)` - Close TCP connection
- `tcpListen(port)` - TCP server listener
- `tcpAccept(listener)` - Accept connection
- `udpBind(port)` - UDP socket
- `udpSendTo(sock, addr, data)` - Send UDP
- `udpRecvFrom(sock)` - Receive UDP
- Plus TLS and WebSocket support

### Type Checking
- `isNumber(val)` - Type check
- `isString(val)` - Type check
- `isBool(val)` - Type check
- `isArray(val)` - Type check
- `isNull(val)` - Type check
- `typeof(val)` - Get type name

### Math
- `round(n)` - Round to integer
- `floor(n)` - Floor
- `ceil(n)` - Ceiling
- `sqrt(n)` - Square root
- `abs(n)` - Absolute value
- `min(a, b)` - Minimum
- `max(a, b)` - Maximum
- `random()` - Random 0-1
- `pow(base, exp)` - Power
- Plus trig functions, constants

---

## Actually Missing (Confirmed)

### Type System Features (v0.4)
- User-defined `struct` types
- User-defined `enum` types
- Trait implementations for custom types
- Method definitions on types

### Syntax
- Object literal syntax `{key: value}` - use `parseJSON()` or `hashMapNew()`

---

## Workarounds

### Object Literals
```atlas
// Instead of: let obj = {name: "test"};
let obj = parseJSON("{\"name\": \"test\"}");
// OR
let obj = hashMapNew();
hashMapPut(obj, "name", "test");
```

### Custom Types
```atlas
// Instead of struct, use HashMap
fn createConfig(name: string, port: number) -> HashMap {
    let cfg = hashMapNew();
    hashMapPut(cfg, "name", name);
    hashMapPut(cfg, "port", port);
    return cfg;
}
```

---

## Summary

**439 stdlib functions are registered and tested.**

The previous "missing" analysis was outdated. Atlas v0.2 has comprehensive stdlib coverage for:
- JSON (serialize + parse)
- File system (full CRUD + directory ops)
- Process execution
- Strings (20+ methods)
- Collections (HashMap, HashSet)
- Arrays (HOFs: map, filter, reduce)
- DateTime
- HTTP client
- Cryptography
- Networking (TCP, UDP, TLS, WebSocket)
- Regex
- Math
- Type checking
