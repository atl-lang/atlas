# Atlas Standard Library Documentation

Complete reference for all built-in functions in Atlas. This documentation is generated from the actual implementation in `crates/atlas-runtime/src/stdlib/`.

**Last Updated:** 2026-03-05
**Version:** v0.2+

## Module Overview

### Core Functions
- **File:** `core.md`
- **Functions:** print, len, str
- **Purpose:** Basic runtime operations

### String Functions
- **File:** `string.md`
- **Functions:** split, join, trim, indexOf, includes, toUpperCase, toLowerCase, substring, charAt, repeat, replace, padStart, padEnd, startsWith, endsWith
- **Count:** 20 functions
- **Purpose:** String manipulation and Unicode support

### Array Functions
- **File:** `array.md`
- **Functions:** arrayPush, arrayPop, arrayShift, arrayUnshift, arrayReverse, arraySort, concat, flatten, arrayIndexOf, arrayLastIndexOf, arrayIncludes, slice, map, filter, reduce, forEach, find, findIndex, flatMap, some, every, sort, sortBy
- **Count:** 23 functions
- **Purpose:** Pure array operations (Copy-on-Write semantics)

### Math Functions
- **File:** `math.md`
- **Functions:** abs, floor, ceil, round, min, max, sqrt, pow, log, sin, cos, tan, asin, acos, atan, clamp, sign, random
- **Count:** 18 functions
- **Purpose:** IEEE 754 compliant mathematical operations

### JSON Functions
- **File:** `json.md`
- **Functions:** parseJSON, toJSON, isValidJSON, prettifyJSON, minifyJSON, jsonAsString, jsonAsNumber, jsonAsBool, jsonGetString, jsonGetNumber, jsonGetBool, jsonGetArray, jsonGetObject, jsonIsNull
- **Count:** 14 functions
- **Purpose:** JSON parsing, validation, and manipulation

### Type System Functions
- **File:** `types.md`
- **Functions:** typeof, isString, isNumber, isBool, isNull, isArray, isFunction, isObject, isType, hasField, hasMethod, hasTag, toString, toNumber, toBool, parseInt, parseFloat, Some, None, Ok, Err, is_some, is_none, is_ok, is_err, result_map, result_map_err, result_and_then, result_or_else, unwrap, unwrap_or
- **Count:** 34+ functions
- **Purpose:** Type checking, conversion, Option/Result constructors

### Collections Functions
- **File:** `collections.md`
- **Categories:**
  - HashMap: new, fromEntries, put, copy, get, remove, has, size, isEmpty, clear, keys, values, entries, forEach, map, filter (16 functions)
  - HashSet: new, fromArray, add, remove, has, size, isEmpty, clear, union, intersection, difference, symmetricDifference, isSubset, isSuperset, toArray, forEach, map, filter (18 functions)
  - Queue: new, enqueue, dequeue, peek, size, isEmpty, clear, toArray (8 functions)
  - Stack: new, push, pop, peek, size, isEmpty, clear, toArray (8 functions)
- **Total:** 50 functions
- **Purpose:** Data structures with shared/copy-on-write semantics

### Reflection Functions
- **File:** `reflect.md`
- **Functions:** reflect_typeof, reflect_is_callable, reflect_is_primitive, reflect_same_type, reflect_get_length, reflect_is_empty, reflect_type_describe, reflect_clone, reflect_value_to_string, value_to_string, reflect_deep_equals, reflect_get_function_name, reflect_get_function_arity
- **Count:** 13 functions
- **Purpose:** Runtime inspection and introspection helpers

### File System Functions
- **File:** `file.md`
- **Functions:** readFile, writeFile, appendFile, fileExists, removeFile, readDir, createDir, removeDir, fileInfo, pathJoin, fsWalk, fsReaddir, fsMkdir, fsMkdirp, fsRmdir, fsRmdirRecursive, fsSize, fsMtime, fsCtime, fsAtime, fsIsDir, fsIsFile, fsIsSymlink, fsPermissions, fsSymlink, fsReadlink, fsTmpfile, fsTmpdir, fsTmpfileNamed
- **Count:** 30+ functions
- **Purpose:** File I/O, directory operations, metadata access

### Regular Expression Functions
- **File:** `regex.md`
- **Functions:** regexNew, regexNewWithFlags, regexIsMatch, regexTest, regexFind, regexFindAll, regexCaptures, regexCapturesNamed, regexMatchIndices, regexReplace, regexReplaceAll, regexReplaceWith, regexReplaceAllWith, regexSplit, regexSplitN, regexEscape
- **Count:** 16 functions
- **Purpose:** Pattern matching and text manipulation

### DateTime Functions
- **File:** `datetime.md`
- **Categories:**
  - Construction: dateTimeNow, dateTimeFromTimestamp, dateTimeFromComponents
  - Parsing: dateTimeParseIso, dateTimeParse, dateTimeParseRfc3339, dateTimeParseRfc2822, dateTimeTryParse
  - Components: dateTimeYear, dateTimeMonth, dateTimeDay, dateTimeHour, dateTimeMinute, dateTimeSecond, dateTimeWeekday, dateTimeDayOfYear
  - Arithmetic: dateTimeAddSeconds, dateTimeAddMinutes, dateTimeAddHours, dateTimeAddDays
  - Comparison: dateTimeCompare, dateTimeDiff
  - Conversion: dateTimeToTimestamp, dateTimeToIso, dateTimeToRfc3339, dateTimeToRfc2822, dateTimeFormat, dateTimeToCustom
  - Timezone: dateTimeUtc, dateTimeToUtc, dateTimeToLocal, dateTimeToTimezone, dateTimeGetTimezone, dateTimeGetOffset, dateTimeInTimezone
  - Duration (HashMap): durationFromSeconds, durationFromMinutes, durationFromHours, durationFromDays, durationFormat
- **Total:** 45+ functions
- **Purpose:** Date/time creation, parsing, arithmetic, and formatting

### HTTP Functions
- **File:** `http.md`
- **Functions:** httpRequest, httpGet, httpPost, httpPut, httpPatch, httpDelete, httpSetBody, httpSetHeader, httpSetQuery, httpSetAuth, httpSetUserAgent, httpSetTimeout, httpSetFollowRedirects, httpSetMaxRedirects, httpSend, httpSendAsync, httpGetJson, httpPostJson, httpPostAsync, httpGetAsync, httpDeleteAsync, httpPutAsync, httpStatus, httpStatusText, httpBody, httpContentType, httpContentLength, httpHeader, httpHeaders, httpIsSuccess, httpIsRedirect, httpIsClientError, httpIsServerError, httpCheckPermission, httpParseJson
- **Count:** 35+ functions
- **Purpose:** HTTP client with request building and response handling

### Process Functions
- **File:** `process.md`
- **Functions:** spawn, shell, processStdout, processStderr, processStdin, processOutput, processWait, processKill, processIsRunning, getEnv, setEnv, unsetEnv, listEnv, getPid, getCwd
- **Count:** 15 functions
- **Purpose:** Process spawning, execution, and environment access

### Network Functions
- **File:** `net.md`
- **Functions:** tcpConnect, tcpWrite, tcpRead, tcpReadBytes, tcpClose, tcpSetTimeout, tcpSetNodelay, tcpLocalAddr, tcpRemoteAddr, tcpListen, tcpAccept, tcpListenerAddr, tcpListenerClose, udpBind, udpSend, udpReceive, udpSetTimeout, udpClose, udpLocalAddr, tlsConnect, tlsWrite, tlsRead, tlsClose
- **Count:** 24 functions
- **Purpose:** TCP, UDP, and TLS networking

### Encoding Functions
- **File:** `encoding.md`
- **Categories:**
  - Base64: base64Encode, base64Decode, base64UrlEncode, base64UrlDecode
  - Hex: hexEncode, hexDecode
  - URL: urlEncode, urlDecode
  - Hashing: sha256, sha512, blake3Hash
  - HMAC: hmacSha256, hmacSha256Verify
  - AES-GCM: aesGcmGenerateKey, aesGcmEncrypt, aesGcmDecrypt
- **Total:** 18 functions
- **Purpose:** Encoding, hashing, and cryptographic operations

### Compression Functions
- **File:** `compression.md`
- **Categories:**
  - Gzip: gzipCompress, gzipDecompress, gzipDecompressString, gzipIsGzip, gzipCompressionRatio
  - Tar: tarCreate, tarCreateGz, tarExtract, tarExtractGz, tarList, tarContains
  - Zip: zipCreate, zipCreateWithComment, zipExtract, zipExtractFiles, zipList, zipContains, zipComment, zipValidate, zipCompressionRatio
- **Total:** 19 functions
- **Purpose:** Archive creation, extraction, and compression

### Async and Concurrency Functions
- **File:** `async.md`
- **Categories:**
  - Futures: futureNew, futureResolve, futureReject, futureThen, futureCatch, futureRace, futureAll, futureIsResolved, futureIsRejected, futureIsPending, await
  - Channels: channelUnbounded, channelBounded, channelSend, channelReceive, channelIsClosed, channelSelect
  - Timers: sleep, timer, timeout, interval
  - Synchronization: asyncMutex, asyncMutexGet, asyncMutexSet
  - Tasks: taskId, taskName, taskStatus, taskJoin, taskCancel
- **Total:** 30+ functions
- **Purpose:** Async/await, futures, channels, and concurrent programming

### Sync Primitives
- **File:** `sync.md`
- **Functions:** rwLockNew, rwLockRead, rwLockWrite, rwLockTryRead, rwLockTryWrite, semaphoreNew, semaphoreAcquire, semaphoreTryAcquire, semaphoreRelease, semaphoreAvailable, atomicNew, atomicLoad, atomicStore, atomicAdd, atomicSub, atomicCompareExchange
- **Count:** 16 functions
- **Purpose:** RwLock, semaphore, and atomic counters

### WebSocket Functions
- **File:** `websocket.md`
- **Functions:** wsConnect, wsSend, wsSendBinary, wsReceive, wsPing, wsClose
- **Count:** 6 functions
- **Purpose:** WebSocket client operations

### Path Functions
- **File:** `path.md`
- **Functions:** pathJoin, pathJoinArray, pathParse, pathNormalize, pathAbsolute, pathRelative, pathParent, pathBasename, pathDirname, pathExtension, pathIsAbsolute, pathIsRelative, pathExists, pathCanonical, pathEquals, pathHomedir, pathCwd, pathTempdir, pathSeparator, pathDelimiter, pathExtSeparator, pathDrive, pathToPlatform, pathToPosix, pathToWindows
- **Count:** 24 functions
- **Purpose:** Cross-platform path manipulation, resolution, and system paths

### Testing Functions
- **File:** `test.md`
- **Functions:** assert, assertEqual, assertNotEqual, assertFalse, assertEmpty, assertLength, assertContains, assertSome, assertNone, assertOk, assertErr, assertThrows, assertNoThrow
- **Count:** 14 functions
- **Purpose:** Unit testing assertions and verification

## Statistics

- **Total Modules Documented:** 20
- **Total Functions:** 500+ functions
- **Documentation Files:** 16+ markdown files
- **Code Modules:** 30 (in crates/atlas-runtime/src/stdlib/)

## Implementation Details

### Copy-on-Write (CoW) Semantics
Array functions return new arrays rather than modifying in-place:
```atlas
let arr = [1, 2, 3];
arr = arrayPush(arr, 4); // Must rebind
```

### Shared Mutation (HashMap/HashSet)
Collections use Arc<Mutex<...>> for shared mutation:
```atlas
let map = hashMapNew();
map = hashMapPut(map, "key", "value");
```

### Option/Result Pattern
Type-safe error handling via Option<T> and Result<T,E>:
```atlas
let opt = Some(42);
let val = unwrap(opt)?; // Extract value or propagate error
```

## Notes

- All function signatures are exact from the Atlas implementation
- Type annotations follow Atlas syntax
- Some functions are aliased (same function, multiple names)
- Functions may return Result<T,E> for error handling
- Async functions return Future<T> instead of blocking
- All string operations are Unicode-aware using Rust's UTF-8 handling
