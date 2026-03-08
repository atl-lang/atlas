# Workarounds for Common Issues

**Updated**: 2026-03-03 (post-codebase audit)

**Note**: Many previously documented "workarounds" are no longer needed because the functions actually exist. This file now contains only **actual** workarounds needed.

---

## Syntax Workarounds (Still Needed)

### Match Expression Returns

**Issue**: Cannot use `return` inside match arms

**Solution**: Match is an expression - use it as a value

```atlas
// WRONG
fn get_value(r: Result<number, string>) -> number {
    match r {
        Ok(v) => { return v; }  // ERROR
        Err(_) => { return 0; }
    }
}

// RIGHT - return the match expression
fn get_value(r: Result<number, string>) -> number {
    return match r {
        Ok(v) => v,
        Err(_) => 0
    };
}

// ALSO RIGHT - block expressions work
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

---

### Multi-Statement Logic in Match

**Issue**: Need multiple statements in match arms

**Solution**: Use block expressions (tail expression without semicolon)

```atlas
// Works with block expressions
let result = match value {
    Ok(v) => {
        print("Processing...");
        let doubled = v * 2;
        doubled  // No semicolon = return value
    },
    Err(e) => {
        print("Error: " + e);
        0
    }
};
```

---

## Feature Workarounds (Still Needed)

### Object Literal Syntax

**Issue**: No `{key: value}` object literal syntax

**Solution**: Use `parseJSON()` or `hashMapNew()`

```atlas
// Can't do this
let obj = {name: "test", value: 42};

// Option 1: Parse JSON string
let obj = parseJSON("{\"name\": \"test\", \"value\": 42}")?;

// Option 2: Use HashMap (better for dynamic data)
let obj = hashMapNew();
hashMapPut(obj, "name", "test");
hashMapPut(obj, "value", 42);
```

---

### User-Defined Types (Struct/Enum)

**Issue**: No user-defined struct or enum types (v0.4 feature)

**Solution**: Use HashMap with factory functions

```atlas
// Can't do this
struct User {
    name: string,
    age: number
}

// Workaround: Factory function + HashMap
fn createUser(name: string, age: number) -> HashMap {
    let user = hashMapNew();
    hashMapPut(user, "name", name);
    hashMapPut(user, "age", age);
    return user;
}

fn getUserName(user: HashMap) -> Option<string> {
    return hashMapGet(user, "name");
}

// Usage
let user = createUser("Alice", 30);
let name = getUserName(user);
```

---

## NO LONGER NEEDED (Functions Exist!)

The following "workarounds" from previous documentation are **obsolete** because the functions actually exist:

### JSON Serialization
```atlas
// toJSON() EXISTS - use it directly!
let json_str = toJSON(data);
```

### File Existence Check
```atlas
// fileExists() EXISTS - use it directly!
let exists = fileExists(path);
```

### Directory Creation
```atlas
// createDir() EXISTS - use it directly!
createDir(".hydra/sessions")?;
```

### Process Execution
```atlas
// exec() EXISTS and WORKS!
let result = exec(["echo", "hello"])?;
print(result.stdout);

// shell() also works
let result = shell("echo hello && ls")?;
```

### String Replace
```atlas
// replace() EXISTS!
let new_str = replace(str, "old", "new");
```

### String Case Conversion
```atlas
// These all exist!
let lower = toLowerCase(str);
let upper = toUpperCase(str);
let idx = indexOf(str, "sub");
let char = charAt(str, 0);
```

### HashMap/HashSet
```atlas
// Fully implemented!
let map = hashMapNew();
hashMapPut(map, "key", "value");
let val = hashMapGet(map, "key");
```

### DateTime
```atlas
// All exist!
let now = dateTimeNow();
sleep(1000);
```

---

## Summary

**Actual workarounds needed**:
1. Match arms must be expressions (use block expressions for multi-statement)
2. No object literal syntax (use parseJSON or hashMapNew)
3. No user-defined types (use HashMap + factory functions)

**Everything else works** - see stdlib-reality.md for the full list of 439 available functions.
