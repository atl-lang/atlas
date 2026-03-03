# JSON Operations

## ✅ Available Functions

### parseJSON()
Parse JSON string to json type:

```atlas
let json_str: string = "{\"name\":\"Atlas\",\"version\":1}";
let result: Result<json, string> = parseJSON(json_str);

let data: json = match result {
    Ok(parsed) => parsed,
    Err(e) => parseJSON("{}")  // Default
};
```

**Returns**: `Result<json, string>`

### Type: json
Built-in JSON type:

```atlas
let data: json = parseJSON("{\"key\":\"value\"}");
```

**Note**: Use `json` type, NOT `JsonValue`

## ❌ NOT Available

### stringify() / JSON.stringify()
**Status**: Does NOT exist

**Workaround**: Work with JSON strings directly instead of json objects:

```atlas
// Instead of:
let obj: json = {key: "value"};
let str: string = stringify(obj);  // DOESN'T EXIST

// Do this:
let json_str: string = "{\"key\":\"value\"}";
// Work with strings, parse when needed
```

### jsonGet() / jsonAsString()
**Status**: May exist (seen in old test files)
**Recommendation**: Access fields directly if supported, or use parseJSON

## Patterns

### Store JSON as Strings

```atlas
// StateStore example - works with JSON strings:
fn save_state(path: string, json_str: string) -> Result<bool, string> {
    let write_result = writeFile(path, json_str);
    // ...
}

// Usage:
let state_json: string = "{\"count\":42}";
save_state(path, state_json);
```

### Parse When Needed

```atlas
fn load_and_parse(path: string) -> Result<json, string> {
    let read_result = readFile(path);

    let json_str: string = match read_result {
        Ok(content) => content,
        Err(_) => { return Err("Read failed"); }
    };

    return parseJSON(json_str);
}
```

## Summary

- **Parsing**: ✅ `parseJSON()` works well
- **Stringifying**: ❌ Not available
- **Workaround**: Use JSON strings throughout, parse only when needed
- **Type**: Use `json` not `JsonValue`
