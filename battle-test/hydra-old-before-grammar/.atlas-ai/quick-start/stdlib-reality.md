# Stdlib Reality Check - CORRECTED

**Verified against codebase**: 2026-03-03
**Atlas version**: v0.2
**Total stdlib functions**: 439 registered

---

## Summary

Atlas v0.2 has a **comprehensive stdlib**. Previous documentation was severely outdated.

---

## CONFIRMED WORKING

### Strings (20+ functions)
| Function | Signature | Status |
|----------|-----------|--------|
| `substring(str, start, end)` | `string -> string` | Works |
| `trim(str)` | `string -> string` | Works |
| `startsWith(str, prefix)` | `string -> bool` | Works |
| `endsWith(str, suffix)` | `string -> bool` | Works |
| `split(str, delim)` | `string -> array` | Works |
| `join(arr, delim)` | `array -> string` | Works |
| `len(str)` | `string -> number` | Works |
| `str(num)` | `number -> string` | Works |
| `replace(str, old, new)` | `string -> string` | Works |
| `toLowerCase(str)` | `string -> string` | Works |
| `toUpperCase(str)` | `string -> string` | Works |
| `indexOf(str, substr)` | `string -> number` | Works |
| `charAt(str, idx)` | `string -> string` | Works |
| `includes(str, substr)` | `string -> bool` | Works |
| `padStart(str, len, char)` | `string -> string` | Works |
| `padEnd(str, len, char)` | `string -> string` | Works |

### JSON
| Function | Signature | Status |
|----------|-----------|--------|
| `parseJSON(str)` | `string -> Result<json, string>` | Works |
| `toJSON(value)` | `any -> string` | Works |

### File I/O
| Function | Signature | Status |
|----------|-----------|--------|
| `readFile(path)` | `string -> Result<string, string>` | Works |
| `writeFile(path, content)` | `(string, string) -> Result<null, string>` | Works |
| `appendFile(path, content)` | `(string, string) -> Result<null, string>` | Works |
| `createDir(path)` | `string -> Result<null, string>` | Works |
| `fileExists(path)` | `string -> bool` | Works |
| `removeFile(path)` | `string -> Result<null, string>` | Works |
| `removeDir(path)` | `string -> Result<null, string>` | Works |
| `readDir(path)` | `string -> Result<array, string>` | Works |
| `fileInfo(path)` | `string -> Result<FileInfo, string>` | Works |

### Process Execution
| Function | Signature | Status |
|----------|-----------|--------|
| `exec(args)` | `array -> Result<ExecResult, string>` | Works |
| `shell(cmd)` | `string -> Result<ExecResult, string>` | Works |
| `spawn(args)` | `array -> Result<Process, string>` | Works |

### Arrays (20+ functions)
| Function | Signature | Status |
|----------|-----------|--------|
| `len(arr)` | `array -> number` | Works |
| `push(arr, val)` | `(array, T) -> array` | Works |
| `pop(arr)` | `array -> Option<T>` | Works |
| `shift(arr)` | `array -> Option<T>` | Works |
| `unshift(arr, val)` | `(array, T) -> array` | Works |
| `reverse(arr)` | `array -> array` | Works |
| `concat(arr1, arr2)` | `(array, array) -> array` | Works |
| `slice(arr, start, end)` | `array -> array` | Works |
| `sort(arr)` | `array -> array` | Works |
| `flatten(arr)` | `array -> array` | Works |
| `map(arr, fn)` | `(array, fn) -> array` | Works |
| `filter(arr, fn)` | `(array, fn) -> array` | Works |
| `reduce(arr, fn, init)` | `(array, fn, T) -> T` | Works |
| `flatMap(arr, fn)` | `(array, fn) -> array` | Works |
| `forEach(arr, fn)` | `(array, fn) -> void` | Works |
| `some(arr, fn)` | `(array, fn) -> bool` | Works |
| `every(arr, fn)` | `(array, fn) -> bool` | Works |
| `find(arr, fn)` | `(array, fn) -> Option<T>` | Works |
| `findIndex(arr, fn)` | `(array, fn) -> number` | Works |

### HashMap
| Function | Signature | Status |
|----------|-----------|--------|
| `hashMapNew()` | `() -> HashMap` | Works |
| `hashMapPut(map, key, val)` | `(HashMap, string, T) -> HashMap` | Works |
| `hashMapGet(map, key)` | `(HashMap, string) -> Option<T>` | Works |
| `hashMapRemove(map, key)` | `(HashMap, string) -> HashMap` | Works |
| `hashMapKeys(map)` | `HashMap -> array` | Works |
| `hashMapValues(map)` | `HashMap -> array` | Works |
| `hashMapSize(map)` | `HashMap -> number` | Works |
| `hashMapForEach(map, fn)` | `(HashMap, fn) -> void` | Works |
| `hashMapMap(map, fn)` | `(HashMap, fn) -> HashMap` | Works |
| `hashMapFilter(map, fn)` | `(HashMap, fn) -> HashMap` | Works |

### HashSet
| Function | Signature | Status |
|----------|-----------|--------|
| `hashSetNew()` | `() -> HashSet` | Works |
| `hashSetAdd(set, val)` | `(HashSet, T) -> HashSet` | Works |
| `hashSetRemove(set, val)` | `(HashSet, T) -> HashSet` | Works |
| `hashSetContains(set, val)` | `(HashSet, T) -> bool` | Works |
| `hashSetSize(set)` | `HashSet -> number` | Works |
| `hashSetMap(set, fn)` | `(HashSet, fn) -> HashSet` | Works |
| `hashSetFilter(set, fn)` | `(HashSet, fn) -> HashSet` | Works |

### DateTime
| Function | Signature | Status |
|----------|-----------|--------|
| `dateTimeNow()` | `() -> DateTime` | Works |
| `sleep(ms)` | `number -> void` | Works |
| `dateTimeFromTimestamp(ts)` | `number -> DateTime` | Works |
| `dateTimeFromComponents(...)` | `(y,m,d,h,m,s) -> DateTime` | Works |

### HTTP Client
| Function | Signature | Status |
|----------|-----------|--------|
| `httpRequest(url, opts)` | `(string, opts) -> Result<Response, string>` | Works |
| `httpRequestGet(url)` | `string -> Result<Response, string>` | Works |
| `httpRequestPost(url, body)` | `(string, string) -> Result<Response, string>` | Works |
| `httpRequestPut(url, body)` | `(string, string) -> Result<Response, string>` | Works |
| `httpRequestDelete(url)` | `string -> Result<Response, string>` | Works |
| `httpRequestPatch(url, body)` | `(string, string) -> Result<Response, string>` | Works |

### Regex
| Function | Signature | Status |
|----------|-----------|--------|
| `regexNew(pattern)` | `string -> Result<Regex, string>` | Works |
| `regexNewWithFlags(pattern, flags)` | `(string, string) -> Result<Regex, string>` | Works |
| `regexEscape(str)` | `string -> string` | Works |
| `regexIsMatch(re, str)` | `(Regex, string) -> bool` | Works |
| `regexFind(re, str)` | `(Regex, string) -> Option<Match>` | Works |
| `regexFindAll(re, str)` | `(Regex, string) -> array` | Works |
| `regexCaptures(re, str)` | `(Regex, string) -> Option<array>` | Works |
| `regexCapturesNamed(re, str)` | `(Regex, string) -> Option<HashMap>` | Works |

### Cryptography
| Function | Signature | Status |
|----------|-----------|--------|
| `sha256(data)` | `string -> string` | Works |
| `sha512(data)` | `string -> string` | Works |
| `blake3Hash(data)` | `string -> string` | Works |
| `hmacSha256(key, data)` | `(string, string) -> string` | Works |
| `hmacSha256Verify(key, data, sig)` | `(string, string, string) -> bool` | Works |
| `aes256GcmEncrypt(key, nonce, data)` | `(...) -> Result<string, string>` | Works |
| `aes256GcmDecrypt(key, nonce, data)` | `(...) -> Result<string, string>` | Works |
| `base64Encode(data)` | `string -> string` | Works |
| `base64Decode(str)` | `string -> Result<string, string>` | Works |
| `randomBytes(len)` | `number -> string` | Works |
| `randomString(len)` | `number -> string` | Works |

### Networking
| Function | Signature | Status |
|----------|-----------|--------|
| `tcpConnect(host, port)` | `(string, number) -> Result<TcpConn, string>` | Works |
| `tcpWrite(conn, data)` | `(TcpConn, string) -> Result<number, string>` | Works |
| `tcpRead(conn)` | `TcpConn -> Result<string, string>` | Works |
| `tcpClose(conn)` | `TcpConn -> void` | Works |
| `tcpListen(port)` | `number -> Result<TcpListener, string>` | Works |
| `tcpAccept(listener)` | `TcpListener -> Result<TcpConn, string>` | Works |
| `udpBind(port)` | `number -> Result<UdpSocket, string>` | Works |
| `udpSendTo(sock, addr, data)` | `(...) -> Result<number, string>` | Works |
| `udpRecvFrom(sock)` | `UdpSocket -> Result<(string, string), string>` | Works |

### Math
| Function | Signature | Status |
|----------|-----------|--------|
| `round(n)` | `number -> number` | Works |
| `floor(n)` | `number -> number` | Works |
| `ceil(n)` | `number -> number` | Works |
| `sqrt(n)` | `number -> number` | Works |
| `abs(n)` | `number -> number` | Works |
| `min(a, b)` | `(number, number) -> number` | Works |
| `max(a, b)` | `(number, number) -> number` | Works |
| `random()` | `() -> number` | Works |
| `pow(base, exp)` | `(number, number) -> number` | Works |
| `sin(n)`, `cos(n)`, `tan(n)` | `number -> number` | Works |

### Type Checking
| Function | Signature | Status |
|----------|-----------|--------|
| `typeof(val)` | `any -> string` | Works |
| `isNumber(val)` | `any -> bool` | Works |
| `isString(val)` | `any -> bool` | Works |
| `isBool(val)` | `any -> bool` | Works |
| `isArray(val)` | `any -> bool` | Works |
| `isNull(val)` | `any -> bool` | Works |

### Control Flow
| Construct | Status |
|-----------|--------|
| `for item in array { }` | Works |
| `for i in 0..n { }` | Works |
| `if (cond) { } else { }` | Works |
| `match expr { arms }` | Works |
| `Result<T, E>` / `Option<T>` | Works |
| `?` operator (try) | Works |

### I/O
| Function | Status |
|----------|--------|
| `print(val)` | Works |
| `println(val)` | Works |
| `eprint(val)` | Works |
| `eprintln(val)` | Works |

---

## NOT IMPLEMENTED (Actual Gaps)

### Type System Features (v0.4)
- User-defined `struct` types
- User-defined `enum` types
- Method definitions on custom types
- Trait implementations for custom types

### Syntax
- Object literal syntax `{key: value}` - use `parseJSON()` or `hashMapNew()`

---

## Quick Decision Tree

| Need to... | Solution | Status |
|------------|----------|--------|
| Parse JSON | `parseJSON(str)` | Works |
| Stringify JSON | `toJSON(value)` | Works |
| Read file | `readFile(path)` | Works |
| Write file | `writeFile(path, content)` | Works |
| Create directory | `createDir(path)` | Works |
| Check file exists | `fileExists(path)` | Works |
| List directory | `readDir(path)` | Works |
| Delete file | `removeFile(path)` | Works |
| String replace | `replace(str, old, new)` | Works |
| String lowercase | `toLowerCase(str)` | Works |
| String indexOf | `indexOf(str, substr)` | Works |
| Array map | `map(arr, fn)` | Works |
| Array filter | `filter(arr, fn)` | Works |
| Array reduce | `reduce(arr, fn, init)` | Works |
| HashMap | `hashMapNew()` + `hashMapPut/Get` | Works |
| HashSet | `hashSetNew()` + `hashSetAdd` | Works |
| Current time | `dateTimeNow()` | Works |
| Sleep/delay | `sleep(ms)` | Works |
| Regex match | `regexNew(pattern)` + `regexIsMatch(re, str)` | Works |
| HTTP GET | `httpRequestGet(url)` | Works |
| HTTP POST | `httpRequestPost(url, body)` | Works |
| SHA256 hash | `sha256(data)` | Works |
| Base64 encode | `base64Encode(data)` | Works |
| TCP connect | `tcpConnect(host, port)` | Works |
| Execute process | `exec(args)` | Works |
| Custom type | Use HashMap + factory function | Workaround |

---

## Bottom Line

**Atlas v0.2 stdlib is production-ready** for most use cases. The only significant gap is user-defined types (struct/enum), scheduled for v0.4.

Previously reported "missing" functions were outdated information. The stdlib has 439 functions covering JSON, file I/O, networking, HTTP, crypto, regex, collections, and more.
