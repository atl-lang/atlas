# Atlas Standard Library

All stdlib functions use `namespace.method()` syntax (D-021, D-049).

## Namespaces

### Core

| Namespace | Description |
|-----------|-------------|
| `console` | Output: log, error, warn, debug |
| `test` | Testing assertions |

### Math & Types

| Namespace | Description |
|-----------|-------------|
| `Math` | Math functions: sqrt, abs, sin, cos, floor, ceil, round, pow, log, random |

### Data

| Namespace | Description |
|-----------|-------------|
| `Json` | JSON: parse, stringify, minify, keys |
| `Encoding` | Base64, hex encoding/decoding |
| `Regex` | Regular expressions: test, match, replace, split |

### I/O

| Namespace | Description |
|-----------|-------------|
| `io` | Console I/O: readLine, readLinePrompt |
| `file` | File operations: read, write, append, exists, remove, rename, copy, readAsync, writeAsync, appendAsync, renameAsync, copyAsync |
| `Path` | Path manipulation: join, dirname, basename, extname, resolve |

### System

| Namespace | Description |
|-----------|-------------|
| `Env` | Environment variables: get, set, unset |
| `process` | Process control: spawn, exec, shell, cwd, pid, exit |
| `DateTime` | Date/time: now, parse, format |

### Network

| Namespace | Description |
|-----------|-------------|
| `Http` | HTTP client: get, post, put, delete, patch |
| `Net` | TCP/UDP: tcpConnect, tcpListen, udpBind |
| `Crypto` | Cryptography: sha256, sha512, hmac |

### Compression

| Namespace | Description |
|-----------|-------------|
| `Gzip` | Gzip: compress, decompress |
| `Tar` | Tar archives: create, extract, list |
| `Zip` | Zip archives: create, extract, list |

### Async

| Namespace | Description |
|-----------|-------------|
| `task` | Tasks: spawn, join, sleep |
| `future` | Futures: resolve, reject, all, race |
| `sync` | Synchronization: atomic, rwLock, semaphore, channel |

### Reflection & Database

| Namespace | Description |
|-----------|-------------|
| `reflect` | Type introspection: typeOf, fields, isCallable, isPrimitive, sameType, clone |
| `sqlite` | SQLite database: open, execute, query, close |

## Instance Methods

Values have methods called with dot syntax.

### Primitives

| Type | Methods |
|------|---------|
| `number` | toString, toFixed, toInt |
| `bool` | toString |
| `string` | length, charAt, substring, indexOf, split, trim, toUpperCase, toLowerCase, startsWith, endsWith, replace, includes, repeat, padStart, padEnd |

### Collections

| Type | Methods |
|------|---------|
| `array` | length, push, pop, shift, unshift, slice, concat, indexOf, includes, reverse, join, map, filter, reduce, forEach, find, findIndex, some, every, sort, flat, flatMap |
| `HashMap` | get, set, has, delete, keys, values, entries, size, clear |
| `HashSet` | add, has, delete, size, clear, values |
| `Queue` | enqueue, dequeue, peek, size, isEmpty |
| `Stack` | push, pop, peek, size, isEmpty |

### Wrapped Types

| Type | Methods |
|------|---------|
| `Option<T>` | isSome, isNone, unwrap, unwrapOr, map, andThen |
| `Result<T,E>` | isOk, isErr, unwrap, unwrapErr, unwrapOr, map, mapErr, andThen |
| `DateTime` | year, month, day, hour, minute, second, timestamp, format, addDays, addHours |
| `Regex` | test, find, findAll, replace, split |
| `Future<T>` | then, catch, finally |
| `ProcessOutput` | stdout, stderr, exitCode, success |

## Casing Convention (D-049)

- Lowercase: `console`, `test`, `io`, `file`, `task`, `future`, `sync`, `process`, `reflect`, `sqlite`
- PascalCase: `Math`, `Json`, `Path`, `Env`, `DateTime`, `Http`, `Net`, `Crypto`, `Encoding`, `Regex`, `Gzip`, `Tar`, `Zip`
