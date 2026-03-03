# Struct & Enum Support Status

**Updated**: 2026-03-03

## What Works ✅

### Struct Declarations
```atlas
struct User {
    name: string,
    age: number
}
```
- Parses correctly
- Provides code documentation
- No runtime type enforcement yet

### Struct Expressions
```atlas
let user = User { name: "Alice", age: 30 };
let point = Point { x: 10, y: 20 };
```
- Fully working
- Compiles to HashMap at runtime
- Access via `hashMapGet(user, "name")`

### Object Literals
```atlas
let obj = { name: "test", value: 42 };
```
- Fully working
- Same as struct expressions (HashMap)

### HashMap Serialization
```atlas
let user = User { name: "Alice", age: 30 };
let json = toJSON(user);  // {"name":"Alice","age":30}
```
- Fully working
- Supports nested arrays/objects

### Enum Declarations
```atlas
enum Color {
    Red,
    Green,
    Blue
}
```
- Parses correctly
- Provides code documentation

## What Doesn't Work Yet ❌

### Enum Variant Syntax
```atlas
// NOT SUPPORTED YET
let color = Color::Red;
let state = State::Running;
```
- Parser doesn't support `::` for custom enums
- Workaround: Use string constants or built-in Option/Result

### Struct Field Type Validation
```atlas
struct User { name: string }
// No error if you do:
let user = User { name: 123 };  // Accepts number!
```
- Struct expressions don't validate against declarations
- They just create HashMaps

## Recommended Patterns

### For Enums (Workaround)
```atlas
// Use string constants
let STATE_STOPPED: string = "STOPPED";
let STATE_RUNNING: string = "RUNNING";

fn get_state() -> string {
    return STATE_RUNNING;
}
```

### For Structs (Working)
```atlas
struct Config {
    host: string,
    port: number
}

let cfg = Config { host: "localhost", port: 8080 };
let host = unwrap(hashMapGet(cfg, "host"));
```

## Test Files

- `test_struct_basic.atl` - Basic struct expressions
- `test_struct_decl.atl` - Struct declarations + expressions
- `test_enum_decl.atl` - Enum declarations (parse only)
