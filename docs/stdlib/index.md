# Atlas Standard Library

Functions available in Atlas. All signatures verified against actual implementation.

## Categories

| Category | File | Description |
|----------|------|-------------|
| Core | [core.md](core.md) | print, len, str, typeof |
| Array | [array.md](array.md) | arrayPush, map, filter, reduce |
| HashMap | [hashmap.md](hashmap.md) | hashMapNew, hashMapGet, hashMapPut |
| String | [string.md](string.md) | split, trim, indexOf, replace |
| Math | [math.md](math.md) | abs, floor, ceil, sqrt, random |
| DateTime | [datetime.md](datetime.md) | dateTimeNow, dateTimeParse |
| File | [file.md](file.md) | readFile, writeFile, fileExists |
| HTTP | [http.md](http.md) | httpGet, httpPost |
| JSON | [json.md](json.md) | parseJSON, toJSON |
| Regex | [regex.md](regex.md) | regexNew, regexTest, regexFind |
| Async | [async.md](async.md) | spawn, await, sleep |
| Process | [process.md](process.md) | exec, spawnProcess |

## Naming Convention

Atlas uses `camelCase` for all stdlib functions:
- `arrayPush` not `array_push` or `push`
- `hashMapGet` not `hash_map_get` or `get`
- `parseJSON` not `parse_json` or `JSON.parse`

## Usage Pattern

```atlas
// Functions are global - no imports needed
let arr = [1, 2, 3];
let arr2 = arrayPush(arr, 4);  // Returns new array (CoW)
print(len(arr2));              // 4
```

## Note

This documentation is generated from codebase analysis.
If something doesn't work as documented, the docs are wrong - file an issue.
