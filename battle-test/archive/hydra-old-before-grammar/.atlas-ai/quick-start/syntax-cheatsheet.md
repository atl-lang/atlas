# Atlas Syntax Cheatsheet

**Copy-paste working patterns** from battle-tested code.

> **AI Agents**: See also `~/.claude/.../memory/MEMORY.md` for component-specific patterns

---

## Variables

```atlas
// Immutable (default)
let name: string = "Atlas";
let count: number = 42;
let flag: bool = true;

// Mutable (use mut)
let mut counter: number = 0;
counter = counter + 1;  // OK

// Type annotations REQUIRED
let x: number = 5;  // ✅
let x = 5;           // ❌ Type inference not available
```

---

## Functions

```atlas
// With return value
fn add(a: number, b: number) -> number {
    return a + b;
}

// No return value
fn greet(name: string) -> void {
    print("Hello, " + name);
}

// Export (for modules)
export fn public_function() -> string {
    return "exported";
}
```

---

## Result Type (Error Handling)

```atlas
// Return Result
fn divide(a: number, b: number) -> Result<number, string> {
    if (b == 0) {
        return Err("Division by zero");
    }
    return Ok(a / b);
}

// Handle Result - Pattern 1
let result: Result<number, string> = divide(10, 2);
let value: number = match result {
    Ok(v) => v,
    Err(e) => 0
};

// Handle Result - Pattern 2 (check error first)
let has_error: bool = match result {
    Ok(_) => false,
    Err(_) => true
};

if (has_error) {
    print("Error occurred");
    return Err("Failed");
}

let value: number = match result {
    Ok(v) => v,
    Err(_) => 0  // Won't happen, but required
};
```

---

## Match Expressions

**CRITICAL**: Match is an expression. NO return inside arms.

```atlas
// ✅ CORRECT - Match returns value
fn get_status(code: number) -> string {
    return match code {
        200 => "OK",
        404 => "Not Found",
        500 => "Server Error",
        _ => "Unknown"
    };
}

// ✅ CORRECT - Bind result first
fn process(r: Result<string, string>) -> string {
    let value: string = match r {
        Ok(data) => data,
        Err(e) => "default"
    };
    return value;
}

// ❌ WRONG - NO return inside match!
fn wrong(r: Result<string, string>) -> string {
    match r {
        Ok(data) => { return data; }   // ERROR!
        Err(e) => { return "default"; } // ERROR!
    }
}
```

---

## Strings

```atlas
// Concatenation
let greeting: string = "Hello, " + name + "!";

// Substring
let text: string = "Hello World";
let first: string = substring(text, 0, 5);  // "Hello"

// Trim whitespace
let dirty: string = "  spaces  ";
let clean: string = trim(dirty);  // "spaces"

// Check prefix
if (startsWith(text, "Hello")) {
    print("Starts with Hello");
}

// Split to array
let csv: string = "a,b,c";
let parts: array = split(csv, ",");  // ["a", "b", "c"]

// Length
let length: number = len(text);

// Number to string
let num_str: string = str(42);  // "42"
```

---

## Arrays

```atlas
// Create array
let numbers: array = [1, 2, 3, 4, 5];
let empty: array = [];

// Length
let count: number = len(numbers);

// Iterate
for item in numbers {
    print(str(item));
}

// Build array with mutable result
fn filter_positive(nums: array) -> array {
    let mut result: array = [];
    for num in nums {
        if (num > 0) {
            result = result + [num];  // Append
        }
    }
    return result;
}
```

---

## For Loops

```atlas
// For-in (iterate array)
let items: array = [1, 2, 3];
for item in items {
    print(str(item));
}

// Classic for loop
for (let mut i: number = 0; i < 10; i = i + 1) {
    print(str(i));
}

// With break/continue
for item in items {
    if (item == target) {
        break;
    }
    if (item < 0) {
        continue;
    }
    process(item);
}
```

---

## If Statements

```atlas
// Simple if
if (condition) {
    // do something
}

// If-else
if (x > 10) {
    print("Greater");
} else {
    print("Smaller or equal");
}

// Parentheses REQUIRED
if (x == 5) {  // ✅
    print("Five");
}

if x == 5 {    // ❌ Syntax error
    print("Five");
}
```

---

## File I/O

```atlas
// Read file
fn load_file(path: string) -> Result<string, string> {
    let result: Result<string, string> = readFile(path);

    // Check for errors
    let has_error: bool = match result {
        Ok(_) => false,
        Err(_) => true
    };

    if (has_error) {
        return Err("Failed to read file");
    }

    // Extract content
    let content: string = match result {
        Ok(data) => data,
        Err(_) => ""
    };

    return Ok(content);
}

// Write file (overwrites)
fn save_file(path: string, content: string) -> bool {
    let result: Result<null, string> = writeFile(path, content);

    let success: bool = match result {
        Ok(_) => true,
        Err(_) => false
    };

    return success;
}

// Check if file exists
fn file_exists(path: string) -> bool {
    let result: Result<string, string> = readFile(path);
    let exists: bool = match result {
        Ok(_) => true,
        Err(_) => false
    };
    return exists;
}
```

---

## JSON

```atlas
// Parse JSON
fn parse_config(json_str: string) -> Result<json, string> {
    let result: Result<json, string> = parseJSON(json_str);
    return result;
}

// Work with JSON strings (no stringify!)
fn save_state(path: string) -> bool {
    // Build JSON as string
    let state_json: string = "{\"count\":42,\"active\":true}";

    let result: Result<null, string> = writeFile(path, state_json);

    let success: bool = match result {
        Ok(_) => true,
        Err(_) => false
    };

    return success;
}
```

---

## Modules

```atlas
// module.atl
export fn helper(x: number) -> number {
    return x * 2;
}

// main.atl
import { helper } from "./module";

fn main() -> void {
    let result: number = helper(21);  // 42
    print(str(result));
}

main();
```

---

## Common Patterns

### Mutable String Building

```atlas
fn join_lines(lines: array, separator: string) -> string {
    let mut result: string = "";
    let mut first: bool = true;

    for line in lines {
        if (!first) {
            result = result + separator;
        }
        result = result + line;
        first = false;
    }

    return result;
}
```

### Filter Array

```atlas
fn filter_array(items: array, keep_positive: bool) -> array {
    let mut result: array = [];

    for item in items {
        if (keep_positive && item > 0) {
            result = result + [item];
        }
    }

    return result;
}
```

### Safe Unwrap with Default

```atlas
fn unwrap_or(result: Result<string, string>, default: string) -> string {
    let value: string = match result {
        Ok(data) => data,
        Err(_) => default
    };
    return value;
}

// Usage
let content: string = unwrap_or(readFile("config.json"), "{}");
```

---

## Entry Point

```atlas
// Define main function
fn main() -> void {
    print("Program starting");
    // Your code here
}

// Execute
main();
```

---

## Structs & Enums (Coming Soon)

```atlas
// Struct declaration
struct User {
    name: string,
    age: number
}

// Instantiation
let user = User { name: "Alice", age: 30 };

// Enum declaration
enum State {
    Stopped,
    Running,
    Failed
}

// Pattern matching
match state {
    State::Running => print("Active"),
    State::Stopped => print("Idle"),
    _ => print("Other")
}
```

## Comments

```atlas
// Single-line comment

/* Multi-line
   comment */
```

---

## Quick Reference Card

```atlas
// Variables
let x: type = value;           // Immutable
let mut x: type = value;       // Mutable

// Functions
fn name(param: type) -> type { return value; }

// Results
Ok(value)                      // Success
Err(message)                   // Error

// Match
match expr {
    pattern1 => value1,
    pattern2 => value2,
    _ => default
}

// Strings
trim(s)                        // Remove whitespace
substring(s, start, end)       // Extract
split(s, delim)                // To array
startsWith(s, prefix)          // Check prefix
len(s)                         // Length
str(num)                       // Number to string

// Arrays
len(arr)                       // Length
for item in arr { }            // Iterate

// JSON
parseJSON(str) -> Result<json, string>

// Files
readFile(path) -> Result<string, string>
writeFile(path, content) -> Result<null, string>

// I/O
print(message)                 // Console output
```

---

**Pro Tip**: Keep this file open while coding. 90% of Atlas code uses these patterns.
