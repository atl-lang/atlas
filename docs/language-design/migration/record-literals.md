# Migration: Object literals → record literals

**Decision:** D-009 (Disambiguate {} with record keyword)
**Effort:** Simple prefix addition

---

## Rule

```atlas
// Before (v0.2)
{ key: value, key2: value2 }

// After (v0.3)
record { key: value, key2: value2 }
```

Add `record` prefix to all object literals.

---

## What Changes

| v0.2 | v0.3 | Notes |
|------|------|-------|
| `{ key: val }` | `record { key: val }` | Anonymous record |
| `{ }` | `{ }` | Empty BLOCK (returns null) |
| `TypeName { field: val }` | `TypeName { field: val }` | Struct literal (unchanged) |

---

## Examples

### Simple object
```atlas
// Before
let config = { host: "localhost", port: 8080 };

// After
let config = record { host: "localhost", port: 8080 };
```

### Nested objects
```atlas
// Before
let user = {
    name: "Alice",
    address: {
        city: "Portland",
        zip: "97201"
    }
};

// After
let user = record {
    name: "Alice",
    address: record {
        city: "Portland",
        zip: "97201"
    }
};
```

### In function calls
```atlas
// Before
sendRequest({ method: "POST", body: data });

// After
sendRequest(record { method: "POST", body: data });
```

### Return values
```atlas
// Before
fn getConfig() -> { host: string, port: number } {
    return { host: "localhost", port: 8080 };
}

// After
fn getConfig() -> { host: string, port: number } {
    return record { host: "localhost", port: 8080 };
}
```

---

## What Does NOT Change

### Block expressions
```atlas
// Blocks are unchanged
let result = {
    let x = compute();
    x * 2
};
// Still works - this is a block, not an object
```

### Struct literals (already have type prefix)
```atlas
// Struct literals already have type prefix
let user = User { name: "Alice", age: 30 };
// No change needed
```

### Structural types in type position
```atlas
// Type annotations unchanged
fn process(data: { name: string, age: number }) { }
// The { } in type position is a structural type, not a literal
```

---

## How to Identify Object Literals

Object literals in v0.2 are `{ }` in EXPRESSION position with `key: value` pairs:

```atlas
// Object literal (needs record prefix)
let x = { key: value };
         ^^^^^^^^^^^^^ expression position, has key: value

// Block expression (no change)
let x = { let a = 1; a + 1 };
         ^^^^^^^^^^^^^^^^^^^ has statements, not key: value

// Struct literal (no change)
let x = TypeName { field: value };
        ^^^^^^^^ has type prefix already
```

---

## Verification

This is harder to verify automatically because `{ }` is valid for blocks.

Manual check: Search for `= {` or `( {` patterns and verify each is either:
- A block (has statements/expressions, not key: value)
- A struct literal (has TypeName prefix)
- Updated to `record {`

```bash
# Find potential object literals (manual review needed)
grep -rE '=\s*\{[^}]*:' --include="*.atl" .
```
