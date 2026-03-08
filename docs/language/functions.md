# Functions

This document reflects actual parsing rules in `crates/atlas-runtime/src/parser/mod.rs` and `crates/atlas-runtime/src/parser/expr.rs`.

**Function Declarations**
```
fn name<T>(param: Type, ...) [-> [own|borrow] ReturnType] { ... }
```
- Type parameters (`<T, U>`) are optional.
- Parameters in declarations **require type annotations**.
- Return type annotation is **optional** — the typechecker infers return types from all code paths. Use `-> void` for functions that return nothing.
- If a return type is explicitly annotated, `own` and `borrow` are valid ownership annotations. `share` is **not** allowed on return types.
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
- Ownership annotations are **mandatory** and appear before the parameter name: `own name: Type`.
- Valid annotations: `own`, `borrow`, `share`. Missing annotation is a parse error (AT1007).
- See `docs/language/ownership.md` for semantics and decision tree. (D-034)
- `mut` keyword before a parameter name allows reassigning it inside the function body: `fn f(mut a: number) { a = 5; }`
- Parameters without `mut` are immutable — assignment produces AT3003.

Example (tested):
```atlas
fn take(own data: HashMap<string, number>, borrow label: string, share cache: HashMap<string, number>) -> void {
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
- Ownership annotations are **mandatory** on closure parameters too: `fn(own x: T, borrow y: number) { ... }`.
- Return type annotation is optional for closures (inferred from context), **required** for named functions.

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

