# Match Expressions - The #1 Gotcha

**90% of Atlas syntax errors come from misunderstanding match expressions.**

---

## The Rule

**Match is an EXPRESSION that returns a value, not a STATEMENT.**

You CANNOT use `return` inside match arms. You CANNOT have multi-statement blocks.

---

## ❌ WRONG (Will Fail)

### Error 1: Using `return` Inside Match

```atlas
fn process(result: Result<string, string>) -> string {
    match result {
        Ok(data) => {
            return data;  // ERROR: Expected expression
        },
        Err(e) => {
            return "default";  // ERROR: Expected expression
        }
    }
}
```

**Error**: `Expected expression` at the `return` keyword position.

### Error 2: Multi-Statement Blocks

```atlas
fn handle(result: Result<number, string>) -> bool {
    match result {
        Ok(value) => {
            print("Success!");     // Multiple statements
            process(value);         // not allowed in
            true                    // match arms!
        },
        Err(e) => {
            print("Failed: " + e);
            false
        }
    }
}
```

**Error**: `Expected expression` or `Expected ';' after expression`.

---

## ✅ CORRECT Patterns

### Pattern 1: Direct Return of Match Expression

```atlas
fn process(result: Result<string, string>) -> string {
    return match result {
        Ok(data) => data,
        Err(e) => "default"
    };
}
```

**Why it works**: Match expression returns a value, which `return` then returns.

### Pattern 2: Bind Result, Then Use

```atlas
fn process(result: Result<string, string>) -> string {
    let value: string = match result {
        Ok(data) => data,
        Err(e) => "default"
    };

    return value;
}
```

### Pattern 3: Check Error First, Then Extract

For cases where you need early returns:

```atlas
fn load_config(path: string) -> Result<json, string> {
    let read_result: Result<string, string> = readFile(path);

    // Check if error occurred
    let has_error: bool = match read_result {
        Ok(_) => false,
        Err(_) => true
    };

    if (has_error) {
        return Err("Failed to read config file");
    }

    // Now extract value (we know it's Ok)
    let content: string = match read_result {
        Ok(data) => data,
        Err(_) => ""  // Won't happen, but required for exhaustiveness
    };

    // Continue processing
    return parseJSON(content);
}
```

### Pattern 4: Handle Side Effects Outside Match

```atlas
fn handle(result: Result<number, string>) -> bool {
    // Match returns simple value
    let is_success: bool = match result {
        Ok(value) => true,
        Err(e) => false
    };

    // Side effects OUTSIDE match
    if (is_success) {
        print("Success!");
        let val: number = match result {
            Ok(v) => v,
            Err(_) => 0
        };
        process(val);
    } else {
        print("Failed");
    }

    return is_success;
}
```

---

## Why Does This Happen?

Atlas match expressions follow **Rust semantics**:

```rust
// Rust (same behavior)
let x = match result {
    Ok(v) => v,        // Expression returns value
    Err(e) => 0        // Expression returns value
};
```

Match is NOT like a switch statement in C/Java/Go. It's an expression that evaluates to a value.

---

## Comparison to Other Languages

### C/Java/Go Switch (Statements)
```go
// Go switch - these ARE statements
switch result {
case OK:
    return value;     // ✅ return allowed
case ERROR:
    return default;   // ✅ return allowed
}
```

### Rust/Atlas Match (Expressions)
```atlas
// Atlas match - these are EXPRESSIONS
let x = match result {
    Ok(v) => v,       // ✅ expression returns value
    Err(e) => 0       // ✅ expression returns value
};
```

---

## Decision Tree

**I need to...**

### Return different values based on Result?
→ Use Pattern 1 or 2 (direct return or bind)

```atlas
return match result {
    Ok(v) => v,
    Err(_) => default
};
```

### Return early on error?
→ Use Pattern 3 (check error first)

```atlas
let has_error: bool = match result {
    Ok(_) => false,
    Err(_) => true
};

if (has_error) {
    return Err("early return");
}
```

### Do side effects (print, call functions)?
→ Use Pattern 4 (side effects outside match)

```atlas
let is_ok: bool = match result {
    Ok(_) => true,
    Err(_) => false
};

if (is_ok) {
    print("Success");
    process();
}
```

---

## Common Mistake Signatures

If you see these errors, you're using match wrong:

```
Expected expression
Expected ';' after expression
Non-exhaustive pattern match
```

**Solution**: Check that ALL match arms are single expressions with NO return statements.

---

## Quick Fix Guide

1. **Remove all `return` statements** from match arms
2. **Extract multi-statement logic** to separate functions or if/else blocks
3. **Bind the match result** to a variable
4. **Use the variable** after the match

---

## Examples from Hydra Atlas Battle Test

### Before (Failed)

```atlas
export fn is_jsonrpc_line(line: string) -> bool {
    let result: Result<json, string> = parseJSON(line);

    match result {
        Ok(data) => {
            return true;  // ERROR!
        },
        Err(_) => {
            return false; // ERROR!
        }
    }
}
```

### After (Works)

```atlas
export fn is_jsonrpc_line(line: string) -> bool {
    let result: Result<json, string> = parseJSON(line);

    let is_valid: bool = match result {
        Ok(data) => true,
        Err(_) => false
    };

    return is_valid;
}
```

---

## Summary

**Remember**:
- Match = **expression** (returns value)
- Switch = **statement** (executes code)

**Golden Rule**: If you write `return` inside a match arm, you're doing it wrong.

**Success Pattern**:
1. Match returns a value
2. Bind that value to a variable
3. Use the variable
4. Keep match arms simple (single expressions only)

---

**This gotcha accounts for 90% of initial Atlas syntax errors. Master this, and you'll write clean Atlas code.**
