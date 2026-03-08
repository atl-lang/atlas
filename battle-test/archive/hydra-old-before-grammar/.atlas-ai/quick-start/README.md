# 5-Minute Quick Start for AI Agents

**Read this BEFORE writing any Atlas code.**
**CORRECTED**: 2026-03-03 (post-codebase audit)

---

## CRITICAL UPDATE

Previous documentation claimed many stdlib functions don't exist. **This was wrong.**

Atlas v0.2 has **439 stdlib functions**. See [stdlib-reality.md](./stdlib-reality.md) for the full list.

---

## Critical Syntax Rules

### 1. Match is an Expression - NO return inside match arms

```atlas
// WRONG - Will fail
fn get_value(r: Result<number, string>) -> number {
    match r {
        Ok(v) => { return v; }    // ERROR!
        Err(_) => { return 0; }   // ERROR!
    }
}

// RIGHT - Match returns value
fn get_value(r: Result<number, string>) -> number {
    return match r {
        Ok(v) => v,
        Err(_) => 0
    };
}

// ALSO RIGHT - Block expressions work
fn get_value(r: Result<number, string>) -> number {
    match r {
        Ok(v) => {
            print("Got value");
            v   // No semicolon = block value
        },
        Err(_) => 0
    }
}
```

### 2. Immutable by Default (Rust-style)

```atlas
let x: number = 5;      // Immutable
let mut y: number = 5;  // Mutable
y = 10;                 // OK
x = 10;                 // ERROR
```

### 3. Type Inference Works

```atlas
let name = "Atlas";     // Inferred as string
let count = 42;         // Inferred as number
let items = [1, 2, 3];  // Inferred as number[]
```

### 4. Semicolons Required

```atlas
let x = 42;
print("hello");
return x;
```

---

## Stdlib - What Actually Works

### JSON (both directions)
```atlas
let data = parseJSON("{\"name\": \"test\"}")?;
let json_str = toJSON(data);  // THIS EXISTS!
```

### File I/O (full CRUD)
```atlas
readFile(path)                    // Read
writeFile(path, content)          // Write
appendFile(path, content)         // Append
createDir(path)                   // mkdir - THIS EXISTS!
fileExists(path)                  // Check existence - THIS EXISTS!
removeFile(path)                  // Delete file
removeDir(path)                   // Delete directory
readDir(path)                     // List directory
fileInfo(path)                    // Get metadata
```

### Process Execution (all work)
```atlas
exec(["echo", "hello"])           // THIS WORKS!
shell("echo hello && ls")         // THIS WORKS!
spawn(["long-process"])           // THIS WORKS!
```

### Strings (20+ methods)
```atlas
replace(str, "old", "new")        // THIS EXISTS!
toLowerCase(str)                   // THIS EXISTS!
toUpperCase(str)                   // THIS EXISTS!
indexOf(str, "substr")             // THIS EXISTS!
charAt(str, 0)                     // THIS EXISTS!
includes(str, "substr")
startsWith(str, "prefix")
endsWith(str, "suffix")
trim(str)
split(str, ",")
join(arr, ",")
substring(str, 0, 5)
padStart(str, 10, "0")
padEnd(str, 10, " ")
```

### Arrays (20+ methods + HOFs)
```atlas
push(arr, val)
pop(arr)
shift(arr)
unshift(arr, val)
map(arr, (x) => x * 2)
filter(arr, (x) => x > 0)
reduce(arr, (acc, x) => acc + x, 0)
sort(arr)
reverse(arr)
slice(arr, 0, 3)
concat(arr1, arr2)
flatten(arr)
find(arr, (x) => x == target)
some(arr, (x) => x > 0)
every(arr, (x) => x > 0)
```

### HashMap (fully implemented)
```atlas
let map = hashMapNew();
hashMapPut(map, "key", "value");
let val = hashMapGet(map, "key");
hashMapRemove(map, "key");
let keys = hashMapKeys(map);
let size = hashMapSize(map);
```

### DateTime
```atlas
let now = dateTimeNow();
sleep(1000);  // milliseconds
```

### HTTP Client
```atlas
let resp = httpRequestGet("https://api.example.com")?;
let resp = httpRequestPost(url, body)?;
let resp = httpRequestPut(url, body)?;
let resp = httpRequestDelete(url)?;
```

### Regex
```atlas
let re = regexNew("[0-9]+")?;
let matches = regexIsMatch(re, "test123");
let found = regexFind(re, "test123");
let all = regexFindAll(re, "a1b2c3");
```

### Crypto
```atlas
let hash = sha256("data");
let hmac = hmacSha256(key, data);
let encoded = base64Encode(data);
let decoded = base64Decode(str)?;
```

### Networking
```atlas
let conn = tcpConnect("localhost", 8080)?;
tcpWrite(conn, "hello");
let data = tcpRead(conn)?;
tcpClose(conn);

let listener = tcpListen(8080)?;
let client = tcpAccept(listener)?;
```

---

## Only Missing (v0.4)

### 1. User-Defined Types
```atlas
// Can't do this yet
struct Config {
    name: string,
    port: number
}

// Workaround: Use HashMap
let config = hashMapNew();
hashMapPut(config, "name", "server");
hashMapPut(config, "port", 8080);
```

### 2. Object Literal Syntax
```atlas
// Can't do this
let obj = {name: "test", value: 42};

// Workaround: parseJSON or hashMapNew
let obj = parseJSON("{\"name\": \"test\", \"value\": 42}");
```

---

## Working Patterns

### Error Handling with ?
```atlas
fn load_config(path: string) -> Result<HashMap, string> {
    let content = readFile(path)?;
    let data = parseJSON(content)?;
    Ok(data)
}
```

### HashMap as Struct Alternative
```atlas
fn createUser(name: string, age: number) -> HashMap {
    let user = hashMapNew();
    hashMapPut(user, "name", name);
    hashMapPut(user, "age", age);
    return user;
}

let user = createUser("Alice", 30);
let name = hashMapGet(user, "name");
```

### File Operations
```atlas
fn ensureDir(path: string) -> Result<null, string> {
    if (!fileExists(path)) {
        createDir(path)?;
    }
    Ok(null)
}
```

### Process Execution
```atlas
fn runCommand(cmd: string) -> Result<string, string> {
    let result = shell(cmd)?;
    Ok(result.stdout)
}
```

---

## Running Atlas Code

```bash
atlas check file.atl    # Syntax check
atlas run file.atl      # Execute
atlas repl              # Interactive
```

---

## Checklist Before Writing Code

- [ ] Understand match expressions (no return inside, use block values)
- [ ] Know that 439 stdlib functions exist (not minimal!)
- [ ] Use `let mut` for mutable variables
- [ ] Use HashMap for struct-like data
- [ ] Use `?` for error propagation

---

## Next Steps

1. **Full Stdlib**: See [stdlib-reality.md](./stdlib-reality.md) - 439 functions
2. **Complete Grammar**: See [../reference/GRAMMAR.md](../reference/GRAMMAR.md)
3. **Actual Gaps**: See [../language-reality/critical-gaps.md](../language-reality/critical-gaps.md) - only 2

---

**Key Insight**: Atlas v0.2 is **feature-complete** for most use cases. Only user-defined types are missing. Previous documentation was outdated.
