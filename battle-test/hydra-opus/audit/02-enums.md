# Audit: Enums

**Friction Level:** LOW

## What Worked Well

1. **Enum declaration** - Clean syntax
2. **Variant expressions** - `EnumName::Variant` works
3. **Pattern matching** - `match` on enums is excellent
4. **Unit variants** - Simple state enums easy

## Example (worked first try)

```atlas
enum ServerState {
    Stopped,
    Starting,
    Running,
    Restarting,
    Failed
}

fn server_state_to_string(state: ServerState) -> string {
    return match state {
        ServerState::Stopped => "stopped",
        ServerState::Starting => "starting",
        ServerState::Running => "running",
        ServerState::Restarting => "restarting",
        ServerState::Failed => "failed"
    };
}

let state = ServerState::Running;
print(server_state_to_string(state));  // "running"
```

## Friction Points

### 1. No enum value comparison without custom impl
**Go has:** `iota` generates comparable integers
**Atlas:** Must implement comparison manually

```atlas
// Can't do this directly
if state > ServerState::Running { ... }

// Must match or convert
fn state_ordinal(s: ServerState) -> number {
    return match s {
        ServerState::Stopped => 0,
        ServerState::Starting => 1,
        ServerState::Running => 2,
        ServerState::Restarting => 3,
        ServerState::Failed => 4
    };
}
```

**Impact:** Minor - explicit is better for state machines

### 2. Struct variants not constructible (documented limitation)

```atlas
enum Status {
    Tagged { code: number }  // Can declare...
}

// But can't construct with:
// let s = Status::Tagged { code: 42 };  // Not supported
```

**Impact:** Must use tuple variants instead

### 3. No derive for common traits

**Rust has:** `#[derive(Debug, Clone, PartialEq)]`
**Atlas:** Manual implementation required

**Impact:** More boilerplate for utility traits

## Pattern Matching Quality

Pattern matching is one of Atlas's strongest features:

```atlas
let result: Result<number, string> = Ok(42);

let value = match result {
    Ok(n) => n,
    Err(e) => {
        print(`Error: ${e}`);
        0
    }
};
```

- Exhaustiveness checking works
- Guard expressions supported
- Binding works (`Ok(n)` captures n)

## AI Generation Notes

AI would correctly generate enum definitions and basic matching. Main issues:
1. Might try struct variant construction
2. Might expect `==` to work on enums without comparison impl

## Comparison to Go

| Aspect | Go | Atlas | Better |
|--------|----|----|--------|
| Definition | `const` + `iota` | `enum {}` | Atlas |
| Associated data | Not built-in | Tuple variants | Atlas |
| Pattern matching | `switch` | `match` | Atlas |
| Exhaustive check | Manual | Automatic | Atlas |
| Comparison | Built-in (int) | Manual | Go |
