# String Operations

## ✅ Available Functions

### substring()
Extract substring by start and end indices:

```atlas
let text: string = "Hello World";
let first: string = substring(text, 0, 5);  // "Hello"
```

**Note**: Uses `substring(str, start, end)` NOT `substr()`

### trim()
Remove leading/trailing whitespace:

```atlas
let text: string = "  hello  ";
let clean: string = trim(text);  // "hello"
```

### startsWith()
Check if string starts with prefix:

```atlas
let text: string = "[DEBUG] message";
if (startsWith(text, "[DEBUG]")) {
    // true
}
```

### split()
Split string into array:

```atlas
let text: string = "a,b,c";
let parts: array = split(text, ",");  // ["a", "b", "c"]

let lines: string = "line1\nline2\nline3";
let line_array: array = split(lines, "\n");
```

### len()
Get string length:

```atlas
let text: string = "Hello";
let length: number = len(text);  // 5
```

### str()
Convert number to string:

```atlas
let num: number = 42;
let text: string = str(num);  // "42"
```

### Concatenation
Use `+` operator:

```atlas
let greeting: string = "Hello, " + name + "!";
let path: string = base + "/" + file;
```

## ❌ NOT Available

### replace()
**Status**: May exist, needs verification

### toLowerCase() / toUpperCase()
**Status**: Not confirmed

### indexOf() / lastIndexOf()
**Status**: `indexOf()` exists (seen in transport code)

### charAt()
**Workaround**: Use `substring(str, i, i+1)`

### format() / sprintf()
**Workaround**: Use string concatenation

## Working Example

```atlas
fn process_line(line: string) -> bool {
    let trimmed: string = trim(line);

    if (len(trimmed) == 0) {
        return false;
    }

    let first_char: string = substring(trimmed, 0, 1);

    if (startsWith(trimmed, "[DEBUG]")) {
        return false;
    }

    return true;
}
```
