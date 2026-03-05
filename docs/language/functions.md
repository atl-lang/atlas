# Functions

This document reflects actual parsing rules in `crates/atlas-runtime/src/parser/mod.rs` and `crates/atlas-runtime/src/parser/expr.rs`.

**Function Declarations**
```
fn name<T>(param: Type, ...) -> [own|borrow] ReturnType { ... }
```
- Type parameters (`<T, U>`) are optional.
- Parameters in declarations **require type annotations**.
- Return type annotation is optional. If omitted, the typechecker infers it.
- `own` and `borrow` are valid ownership annotations on return types. `shared` is **not** allowed on return types.
- Type predicates are supported: `-> bool is param: Type`.

Example (tested):
```atlas
fn add(a: number, b: number) -> number {
    return a + b;
}

fn is_string(value: string) -> bool is value: string {
    return value == value;
}
```

**Parameters and Ownership**
- Ownership annotations are optional and appear before the parameter name: `own name: Type`.
- Valid annotations: `own`, `borrow`, `shared`.

Example (tested):
```atlas
fn take(own data: HashMap<string, number>, borrow label: string, shared cache: HashMap<string, number>) -> void {
    data;
    label;
    cache;
    return;
}
```

**Anonymous Functions (Closures)**
```
fn(param, ...) -> ReturnType { block }
```
- Parameters in anonymous functions **may omit** type annotations.
- Ownership annotations are allowed: `fn(own x, borrow y: number) { ... }`.
- Return type annotation is optional.

Example (tested):
```atlas
let inc = fn(x: number) -> number {
    return x + 1;
};
```

**Blocks and Implicit Returns**
- Blocks support a tail expression without a trailing semicolon.
- The tail expression becomes the block value (implicit return).

Example (tested):
```atlas
fn choose(flag: bool) -> number {
    if flag {
        return 1;
    } else {
        return 2;
    }
}
```

**Known Issues (See `docs/known-issues.md`)**
- `fn main()` is not executed automatically; use top-level statements (H-068).
- Closures passed as callbacks do not persist global mutations (H-069).
