# Match Expressions - CRITICAL

## Key Rule: Match is an Expression, Not a Statement

**You CANNOT use `return` inside match arms.** This is the #1 syntax error in Atlas.

## ❌ WRONG - Will Fail

```atlas
fn get_value(result: Result<number, string>) -> number {
    match result {
        Ok(value) => { return value; }      // ERROR: Expected expression
        Err(e) => { return 0; }              // ERROR: Expected expression
    }
}
```

## ✅ RIGHT - Match Returns Value

```atlas
fn get_value(result: Result<number, string>) -> number {
    let value: number = match result {
        Ok(v) => v,
        Err(e) => 0
    };
    return value;
}

// Or directly return the match expression:
fn get_value(result: Result<number, string>) -> number {
    return match result {
        Ok(v) => v,
        Err(e) => 0
    };
}
```

## ❌ WRONG - Multi-Statement Blocks

```atlas
let result = match x {
    Ok(v) => {
        print("Success!");    // Multiple statements not allowed
        v
    },
    Err(e) => {
        print("Failed!");
        0
    }
};
```

## ✅ RIGHT - Single Expressions Only

```atlas
let result = match x {
    Ok(v) => v,
    Err(e) => 0
};

// Handle side effects separately:
let is_ok = match x {
    Ok(v) => true,
    Err(e) => false
};

if (is_ok) {
    print("Success!");
}
```

## Pattern: Early Return vs Match

When you need early returns, check conditions before matching:

```atlas
fn load_state(path: string) -> Result<json, string> {
    let read_result = readFile(path);

    // Check if error first
    let has_error = match read_result {
        Ok(_) => false,
        Err(_) => true
    };

    if (has_error) {
        return Err("Failed to read file");
    }

    // Now safely extract value
    let content = match read_result {
        Ok(data) => data,
        Err(_) => ""  // Won't happen, but required
    };

    // Continue processing...
}
```

## Exhaustiveness

Match must be exhaustive (cover all cases):

```atlas
// Result<T, E> requires both Ok and Err:
match result {
    Ok(v) => v,
    Err(e) => default_value
}

// Use _ for default case:
match protocol {
    1 => "NDJSON",
    2 => "LSP",
    _ => "Unknown"
}
```

## Summary

- Match is an **expression** that returns a value
- **No `return` statements** inside match arms
- **Single expressions only** in each arm
- Must be **exhaustive** (all cases covered)
- Side effects should happen **outside** the match

**Inspiration**: Rust match expressions (exact same behavior)
