# Audit: Standard Library

**Friction Level:** MEDIUM

## Coverage Assessment

The Atlas stdlib is comprehensive for the Hydra port:

| Go Package | Atlas Equivalent | Coverage |
|------------|------------------|----------|
| `encoding/json` | `Json.*`, `json*` functions | GOOD |
| `time` | `DateTime.*`, `sleep` | GOOD |
| `regexp` | `regex*` functions | GOOD |
| `os` | `getEnv`, `spawn`, `process*` | GOOD |
| `path/filepath` | `path*` functions | GOOD |
| `sync` | `Mutex`, `RwLock`, `Atomic` | EXISTS |
| `context` | Not present | MISSING |
| `errors` | `Result<T, string>` | DIFFERENT |
| `fmt` | `print`, `str()` | MINIMAL |

## Major Friction: Deprecated Functions

The stdlib is in transition from global functions to method syntax. This creates significant AI friction:

### JSON Functions
| Deprecated | Preferred | Status |
|------------|-----------|--------|
| `parseJSON(s)` | `Json.parse(s)` | Warning |
| `toJSON(v)` | `Json.stringify(v)` | Warning |
| `jsonGetString(j, k)` | `j.getString(k)` | Warning |

### String Functions
| Deprecated | Preferred | Status |
|------------|-----------|--------|
| `trim(s)` | `s.trim()` | Warning |
| `split(s, d)` | `s.split(d)` | Warning |
| `startsWith(s, p)` | `s.startsWith(p)` | Warning |
| `endsWith(s, p)` | `s.endsWith(p)` | Warning |
| `includes(s, p)` | `s.includes(p)` | Warning |

### Math Functions
| Deprecated | Preferred | Status |
|------------|-----------|--------|
| `floor(n)` | `Math.floor(n)` | Warning |
| `ceil(n)` | `Math.ceil(n)` | Warning |
| `min(a, b)` | `Math.min(a, b)` | Warning |
| `max(a, b)` | `Math.max(a, b)` | Warning |
| `abs(n)` | `Math.abs(n)` | Warning |

### DateTime Functions
| Deprecated | Preferred | Status |
|------------|-----------|--------|
| `dateTimeNow()` | `DateTime.now()` | Warning |
| `dateTimeToTimestamp(dt)` | `dt.timestamp()` | Warning |

### Collection Functions
| Deprecated | Preferred | Status |
|------------|-----------|--------|
| `hashMapNew()` | `HashMap.new()` | Warning |
| `hashMapGet(m, k)` | `m.get(k)` | Warning |
| `hashMapPut(m, k, v)` | `m.set(k, v)` | Warning |
| `arrayPush(arr, x)` | `arr.push(x)` | Warning |
| `arraySort(arr)` | `arr.sort()` | Warning |
| `filter(arr, fn)` | `arr.filter(fn)` | Warning |

## What Works Well

1. **Result<T, E> / Option<T>** - Clean error handling
2. **Regex support** - Full regex with named captures
3. **File I/O** - Comprehensive fs functions
4. **Process spawning** - spawn, processWait, processKill
5. **HTTP client** - httpGet, httpPost, etc.

## What's Missing for Hydra

1. **Context/cancellation** - No context.Context equivalent
2. **Goroutines** - spawn exists but not tested
3. **Channels** - channelSend/Receive exist but not tested
4. **select statement** - No equivalent to Go's select
5. **Signal handling** - Not present or not documented

## Documentation Quality

| Area | Quality | Notes |
|------|---------|-------|
| Function signatures | GOOD | Accurate types |
| Examples | VARIABLE | Some missing |
| Deprecation notices | GOOD | Warnings helpful |
| AI-GENERATION-NOTES.md | EXCELLENT | Must-read before coding |

## Recommendations

1. **P1:** Complete migration to method syntax, remove deprecated globals
2. **P2:** Add context/cancellation API
3. **P2:** Document async/channel usage with examples
4. **P3:** Add more printf-style formatting options
