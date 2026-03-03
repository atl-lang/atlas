# Type System

## Basic Types

```atlas
let s: string = "text";
let n: number = 42;
let b: bool = true;
let a: array = [1, 2, 3];
let j: json = parseJSON("{}");
```

## Result Type

Rust-style Result for error handling:

```atlas
Result<T, E>  // Generic Result type

// Constructors:
Ok(value)     // Success case
Err(error)    // Error case
```

### Usage

```atlas
fn divide(a: number, b: number) -> Result<number, string> {
    if (b == 0) {
        return Err("Division by zero");
    }
    return Ok(a / b);
}

// Pattern match on Result:
let result: Result<number, string> = divide(10, 2);

let value: number = match result {
    Ok(v) => v,
    Err(e) => 0
};
```

### Common Result Types

```atlas
Result<string, string>   // readFile, load_state
Result<json, string>     // parseJSON
Result<null, string>     // writeFile (success returns null)
Result<bool, string>     // save_state, custom operations
Result<object, string>   // exec (if it works)
```

## Type Annotations Required

Must annotate variables and function parameters:

```atlas
// ✅ Required
let name: string = "Atlas";
fn add(a: number, b: number) -> number { ... }

// ❌ Won't work
let name = "Atlas";  // No type inference
```

## Type: json

Special type for JSON data:

```atlas
let data: json = parseJSON("{\"key\":\"value\"}");

// Note: Use 'json' not 'JsonValue'
```

**Limitations**:
- No known way to create json objects directly
- Must parse from strings
- No stringify back to string

## Type: object

Generic object type:

```atlas
let obj: object = {
    field: "value"
};
```

**Limitations**:
- Syntax unclear/brittle
- Complex objects cause errors
- Property access patterns unclear

**Recommendation**: Avoid when possible, use primitives.

## Type: array

Generic array type:

```atlas
let numbers: array = [1, 2, 3];
let words: array = split("a,b,c", ",");

// Access:
for item in numbers {
    print(str(item));
}
```

**Note**: No type parameters (not `array<number>`)

## Type: null

Used with writeFile and similar operations:

```atlas
let result: Result<null, string> = writeFile(path, content);

match result {
    Ok(_) => print("Success"),  // _ because null has no value
    Err(e) => print("Failed")
}
```

## Type Conversions

```atlas
// Number to String:
let n: number = 42;
let s: string = str(n);

// String to Number:
let s: string = "42";
let n: number = parseFloat(s);

// JSON parsing:
let json_str: string = "{...}";
let result: Result<json, string> = parseJSON(json_str);
```

## Function Signatures

```atlas
// No return value:
fn print_message(msg: string) -> void {
    print(msg);
}

// Returns value:
fn add(a: number, b: number) -> number {
    return a + b;
}

// Returns Result:
fn safe_divide(a: number, b: number) -> Result<number, string> {
    if (b == 0) {
        return Err("Division by zero");
    }
    return Ok(a / b);
}
```

## Generic Functions

Not yet observed - may not be supported:

```atlas
// Unknown if this works:
fn identity<T>(value: T) -> T {
    return value;
}
```

## Summary

**Well Supported**:
- Basic types (string, number, bool, array)
- Result type (Rust-style)
- Type annotations
- Pattern matching on types

**Limitations**:
- No type inference
- Generic types unclear
- Object type brittle
- json type limited (no stringify)

**Best Practices**:
- Always annotate types
- Use Result for errors
- Prefer primitives over objects
- Work with JSON strings, not json objects
