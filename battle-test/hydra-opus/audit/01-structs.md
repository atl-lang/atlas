# Audit: Structs

**Friction Level:** LOW

## What Worked Well

1. **Nested struct definitions** - Clean and intuitive
2. **Field access syntax** - `obj.field` works as expected
3. **Struct literal syntax** - `StructName { field: value }` clear
4. **Optional trailing commas** - Good for formatting

## Example (worked first try)

```atlas
struct Registry {
    version: string,
    defaults: Defaults,
    servers: HashMap<string, ServerConfig>
}

struct Defaults {
    debounce_ms: number,
    graceful_shutdown_ms: number
}

let reg = Registry {
    version: "1.0",
    defaults: Defaults {
        debounce_ms: 500,
        graceful_shutdown_ms: 2000
    },
    servers: hashMapNew()
};
```

## Friction Points

### 1. No struct methods without traits
**Go has:** Methods on structs directly
**Atlas requires:** Free functions or trait impl

```go
// Go
func (s *Server) Start() error { ... }
```

```atlas
// Atlas - must use free function
fn server_start(s: Server) -> Result<void, string> { ... }
```

**Impact:** More verbose, but explicit about ownership.
**Workaround:** Use naming convention `type_method()`

### 2. No anonymous structs in expressions
**Go has:** `struct{ name string }{ name: "test" }`
**Atlas:** Must define named struct first

**Impact:** Minor - forces better code organization

### 3. Struct field reassignment requires `let mut`

```atlas
// Won't compile
let s = Server { state: Stopped };
s.state = Running;  // Error: immutable

// Must do
let mut s = Server { state: Stopped };
s.state = Running;  // OK
```

**Impact:** LOW - Rust-like semantics, expected

## AI Generation Notes

AI would correctly generate struct definitions on first try. The main issue is method syntax - AI trained on Go/Rust will try `impl Server { fn start(self) }` which requires a trait in Atlas.

## Comparison to Go

| Aspect | Go | Atlas | Better |
|--------|----|----|--------|
| Definition | `type X struct {}` | `struct X {}` | Atlas (cleaner) |
| Methods | Attached to type | Free functions/traits | Go (more ergonomic) |
| Embedding | `struct { Base }` | Not supported | Go |
| Anonymous | Supported | Not supported | Go |
| Field privacy | Capitalization | Not supported | Go |
