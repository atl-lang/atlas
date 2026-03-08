# Atlas Stdlib Method Conventions (D-032)

**Status:** LOCKED — all B10 phases implement this contract exactly  
**Decision:** D-032  
**Diagnostic:** AT9000 — emitted on any deprecated global name call

---

## Mental Model: TypeScript

Atlas stdlib follows the **TypeScript mental model**. AI generates TypeScript cold and correctly. Every naming and dispatch decision in this doc was made to minimize the delta between "what AI generates instinctively" and "what Atlas accepts."

If you're implementing a phase and wonder "should this be a method or a global?" — ask: what would TypeScript do?

---

## Rule 1 — Built-in Value Types Use Dot-Method Syntax

> **⚠ CoW SEMANTICS — READ FIRST:**
> All collection mutation methods (`push`, `pop`, `set`, `add`, `remove`, etc.) return a **new collection**.
> You MUST rebind the result or the mutation is silently discarded.
> ```atlas
> arr.push(x)        // ❌ WRONG — arr is unchanged, mutation lost
> arr = arr.push(x)  // ✅ CORRECT — rebind the result
> ```
> This applies to Array, HashMap, HashSet, Queue, and Stack.

Every built-in collection and string exposes its operations as methods:

```atlas
// ✅ Correct (B10+)
arr = arr.push(x)          // CoW: mutation returns new array — always rebind
arr = arr.pop()            // CoW: rebind required
arr.map(fn(x) { x * 2 })  // closures use fn(param) { expr } syntax
arr.filter(fn(x) { x > 0 })
arr.len()
arr.includes(x)

str.split(",")
str.trim()
str.toUpperCase()
str.includes("hello")
str.len()

map.get("key")
map.put("key", value)
map.has("key")
map.remove("key")
map.keys()
map.size()

set.add(x)
set.has(x)
set.remove(x)
set.size()
set.toArray()

queue.enqueue(x)
queue.dequeue()
queue.peek()
queue.size()

stack.push(x)
stack.pop()
stack.peek()
stack.size()

// ❌ Deprecated (emit AT9000, still work)
arrayPush(arr, x)
hashMapGet(map, "key")
hashSetHas(set, x)
```

**Types covered:** `Array`, `String`, `HashMap`, `HashSet`, `Queue`, `Stack`

---

## Rule 2 — Static Namespaces Use PascalCase

Global operations that don't belong to a value use `Namespace.method()`:

```atlas
// ✅ Correct (B10+)
Json.parse(s)
Json.stringify(v)
Json.isValid(s)
Json.prettify(s)

Math.sqrt(x)
Math.abs(x)
Math.floor(x)
Math.ceil(x)
Math.round(x)
Math.min(a, b)
Math.max(a, b)
Math.pow(base, exp)
Math.log(x)
Math.sin(x)
Math.cos(x)
Math.tan(x)
Math.clamp(x, lo, hi)
Math.sign(x)
Math.random()

File.read(path)
File.write(path, content)
File.append(path, content)
File.exists(path)
File.remove(path)
File.createDir(path)
File.removeDir(path)

Process.spawn(cmd, args)
Process.exit(code)
Process.cwd()

DateTime.now()
// DateTime instances have methods: dt.year(), dt.month(), dt.day(), dt.hour(), dt.format(fmt), dt.add(dur)

Path.join(a, b)
Path.dirname(p)
Path.basename(p)
Path.exists(p)

Env.get(name)
Env.set(key, value)
Env.unset(key)

Http.get(url)
Http.post(url, body)
Http.put(url, body)
Http.delete(url)
Http.patch(url, body)

Net.tcpConnect(host, port)
Net.tcpListen(port)
Net.udpBind(port)

Crypto.sha256(s)
Crypto.sha512(s)
Crypto.blake3(s)
Crypto.aesEncrypt(data, key)
Crypto.aesDecrypt(data, key)
Crypto.generateKey()

Regex.test(pattern, s)
Regex.match(pattern, s)
Regex.matchAll(pattern, s)
Regex.replace(pattern, s, replacement)
Regex.split(pattern, s)

// ❌ Deprecated (emit AT9000, still work)
parseJSON(s)
toJSON(v)
readFile(path)
writeFile(path, content)
dateTimeNow()
getEnv(name)
setEnv(key, value)
httpGet(url)
sha256(s)
regexTest(pattern, s)
```

---

## Rule 3 — Math Dual Availability (Exception)

Math functions are available both as `Math.method()` **and** as bare globals (no deprecation warning):

```atlas
// Both forms are valid — no AT9000 emitted
Math.sqrt(x)   // ✅ preferred
sqrt(x)        // ✅ also valid (not deprecated)
```

This exception exists because math expressions are frequently written inline and the bare form is unambiguous.

---

## Rule 4 — Static Constructors

Some value types expose static constructors on their namespace:

```atlas
HashMap.new()              // empty HashMap
HashMap.fromEntries(arr)   // build from [[key, value], ...] array
HashSet.new()              // empty HashSet
```

---

## Rule 5 — AT9000 Warning Contract

- AT9000 is a **warning**, never an error
- Old global names **continue to work** — zero breaking changes
- Warning text format: `Deprecated: use X.method() instead. See docs/stdlib/METHOD-CONVENTIONS.md`
- AT9000 is emitted by the **typechecker** (not the interpreter/VM) — once per call site
- Suppression: `@allow(deprecated_stdlib)` on a statement suppresses AT9000 for that line

---

## Phase Dependency Map (B10)

```
P01 (H-122) — THIS DOC + AT9000 registration  ← you are here
P02 (H-123) — Interpreter MemberExpr dispatch (foundation)
P03 (H-124) — VM MemberExpr dispatch (parity with P02)
P04 (H-125) — Array method surface
P05 (H-126) — String method surface
P06 (H-127) — HashMap + HashSet + Queue + Stack method surface
P07 (H-128) — Static: Json, Math, Env
P08 (H-129) — Static: File, Process, DateTime, Path
P09 (H-130) — Static: Net, Http, Crypto, Regex
P10 (H-131) — AT9000 deprecation warnings on all old globals
P11 (H-132) — Typechecker integration (return type resolution)
P12 (H-133) — Battle test gate (target: 8+/10 AI generation score)
```

**Strict order:** P02/P03 must complete before P04–P06 (dispatch plumbing first).  
P07–P09 can be parallelized after P03. P10 after all surfaces done. P11 after P10. P12 last.

---

## Full Deprecation Mapping

### Array

| Old global | New method |
|-----------|------------|
| `arrayPush(arr, x)` | `arr.push(x)` |
| `arrayPop(arr)` | `arr.pop()` |
| `arrayShift(arr)` | `arr.shift()` |
| `arrayUnshift(arr, x)` | `arr.unshift(x)` |
| `arrayIncludes(arr, x)` | `arr.includes(x)` |
| `arrayIndexOf(arr, x)` | `arr.indexOf(x)` |
| `arrayLastIndexOf(arr, x)` | `arr.lastIndexOf(x)` |
| `arrayReverse(arr)` | `arr.reverse()` |
| `arraySort(arr)` | `arr.sort()` |
| `arraySortBy(arr, f)` | `arr.sortBy(f)` |
| `arraySlice(arr, s, e)` | `arr.slice(s, e)` |
| `arrayFlat(arr)` | `arr.flat()` |
| `arrayFlatMap(arr, f)` | `arr.flatMap(f)` |
| `arrayConcat(a, b)` | `a.concat(b)` |
| `arrayLen(arr)` | `arr.len()` |
| `arrayIsEmpty(arr)` | `arr.isEmpty()` |
| `arrayFind(arr, f)` | `arr.find(f)` |
| `arrayFindIndex(arr, f)` | `arr.findIndex(f)` |
| `arraySome(arr, f)` | `arr.some(f)` |
| `arrayEvery(arr, f)` | `arr.every(f)` |
| `arrayForEach(arr, f)` | `arr.forEach(f)` |
| `map(arr, f)` | `arr.map(f)` |
| `filter(arr, f)` | `arr.filter(f)` |
| `reduce(arr, f, init)` | `arr.reduce(f, init)` |

### String

| Old global | New method |
|-----------|------------|
| `split(s, sep)` | `s.split(sep)` |
| `trim(s)` | `s.trim()` |
| `trimStart(s)` | `s.trimStart()` |
| `trimEnd(s)` | `s.trimEnd()` |
| `toUpperCase(s)` | `s.toUpperCase()` |
| `toLowerCase(s)` | `s.toLowerCase()` |
| `stringIncludes(s, sub)` | `s.includes(sub)` |
| `startsWith(s, prefix)` | `s.startsWith(prefix)` |
| `endsWith(s, suffix)` | `s.endsWith(suffix)` |
| `replace(s, a, b)` | `s.replace(a, b)` |
| `replaceAll(s, a, b)` | `s.replaceAll(a, b)` |
| `indexOf(s, sub)` | `s.indexOf(sub)` |
| `lastIndexOf(s, sub)` | `s.lastIndexOf(sub)` |
| `charAt(s, i)` | `s.charAt(i)` |
| `substring(s, start, end)` | `s.substring(start, end)` |
| `stringSlice(s, start, end)` | `s.slice(start, end)` |
| `repeat(s, n)` | `s.repeat(n)` |
| `padStart(s, n, c)` | `s.padStart(n, c)` |
| `padEnd(s, n, c)` | `s.padEnd(n, c)` |
| `len(s)` | `s.len()` |

### HashMap

| Old global | New method |
|-----------|------------|
| `hashMapNew()` | `hashMapNew()` *(not yet deprecated — no replacement syntax exists, D-033)* |
| `hashMapGet(map, k)` | `map.get(k)` |
| `hashMapPut(map, k, v)` | `map.put(k, v)` |
| `hashMapRemove(map, k)` | `map.remove(k)` |
| `hashMapHas(map, k)` | `map.has(k)` |
| `hashMapKeys(map)` | `map.keys()` |
| `hashMapValues(map)` | `map.values()` |
| `hashMapEntries(map)` | `map.entries()` |
| `hashMapSize(map)` | `map.size()` |
| `hashMapIsEmpty(map)` | `map.isEmpty()` |
| `hashMapClear(map)` | `map.clear()` |
| `hashMapForEach(map, f)` | `map.forEach(f)` |
| `hashMapMap(map, f)` | `map.map(f)` |
| `hashMapFilter(map, f)` | `map.filter(f)` |

### HashSet

| Old global | New method |
|-----------|------------|
| `hashSetNew()` | `HashSet.new()` |
| `hashSetAdd(set, x)` | `set.add(x)` |
| `hashSetRemove(set, x)` | `set.remove(x)` |
| `hashSetHas(set, x)` | `set.has(x)` |
| `hashSetSize(set)` | `set.size()` |
| `hashSetIsEmpty(set)` | `set.isEmpty()` |
| `hashSetToArray(set)` | `set.toArray()` |
| `hashSetForEach(set, f)` | `set.forEach(f)` |

### Static Namespaces

| Old global | New namespace form |
|-----------|-------------------|
| `parseJSON(s)` | `Json.parse(s)` |
| `toJSON(v)` | `Json.stringify(v)` |
| `isValidJSON(s)` | `Json.isValid(s)` |
| `prettifyJSON(s)` | `Json.prettify(s)` |
| `sqrt(x)` | `Math.sqrt(x)` (no warning — dual) |
| `abs(x)` | `Math.abs(x)` (no warning — dual) |
| `floor(x)` | `Math.floor(x)` (no warning — dual) |
| `readFile(path)` | `File.read(path)` |
| `writeFile(path, s)` | `File.write(path, s)` |
| `appendFile(path, s)` | `File.append(path, s)` |
| `fileExists(path)` | `File.exists(path)` |
| `removeFile(path)` | `File.remove(path)` |
| `createDir(path)` | `File.createDir(path)` |
| `removeDir(path)` | `File.removeDir(path)` |
| `spawnProcess(cmd, args)` | `Process.spawn(cmd, args)` |
| `exit(code)` | `Process.exit(code)` |
| `cwd()` | `Process.cwd()` |
| `dateTimeNow()` | `DateTime.now()` |
| `pathJoin(a, b)` | `Path.join(a, b)` |
| `dirname(p)` | `Path.dirname(p)` |
| `basename(p)` | `Path.basename(p)` |
| `getEnv(name)` | `Env.get(name)` |
| `setEnv(k, v)` | `Env.set(k, v)` |
| `unsetEnv(k)` | `Env.unset(k)` |
| `httpGet(url)` | `Http.get(url)` |
| `httpPost(url, body)` | `Http.post(url, body)` |
| `tcpConnect(host, port)` | `Net.tcpConnect(host, port)` |
| `sha256(s)` | `Crypto.sha256(s)` |
| `sha512(s)` | `Crypto.sha512(s)` |
| `regexTest(p, s)` | `Regex.test(p, s)` |
| `regexFind(p, s)` | `Regex.match(p, s)` |
| `regexFindAll(p, s)` | `Regex.matchAll(p, s)` |
| `regexReplace(p, s, r)` | `Regex.replace(p, s, r)` |
| `regexSplit(p, s)` | `Regex.split(p, s)` |
