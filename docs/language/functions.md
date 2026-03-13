# Atlas Functions

Functions are declared with the `fn` keyword. Named functions require an explicit return type annotation. Anonymous functions (closures) may omit it.

---

## Named Function Declarations

```atlas
fn add(x: number, y: number): number {
    return x + y;
}

fn greet(name: string): void {
    console.log("Hello, " + name + "!");
}
```

**Syntax:**
```
fn <name>[<TypeParams>](<params>): <ReturnType> { <body> }
```

**Return type is required.** The colon separator (`:`) is mandatory for named functions. Using `->` (Rust style) is rejected with a clear error; the compiler points you to the correct `: T` form.

**`void` for procedures.** Functions that do not return a value must declare `: void`.

---

## Parameters

### Basic parameters

```atlas
fn scale(value: number, factor: number): number {
    return value * factor;
}
```

Every parameter requires `name: Type`. No parameter can be left untyped in named functions.

### Default parameters

```atlas
fn greet(name: string = "World"): void {
    console.log("Hello, " + name + "!");
}

greet();        // "Hello, World!"
greet("Atlas"); // "Hello, Atlas!"
```

Default parameters must follow required parameters.

### Rest parameters (variadic)

```atlas
fn sum(...values: number[]): number {
    let mut total = 0;
    for v in values {
        total += v;
    }
    return total;
}

sum(1, 2, 3);   // 6
```

The rest parameter must be last. Prefix is `...name: T[]`. No ownership annotation or default value is allowed on a rest parameter.

### Mutable parameters

```atlas
fn increment(mut n: number): number {
    n += 1;
    return n;
}
```

`mut` marks the parameter as locally mutable. It does not affect the caller's binding.

---

## Ownership Annotations (D-040)

Ownership annotations on parameters control how values are passed across function boundaries. The Rust runtime enforces these semantics transparently.

| Annotation | Meaning | When to write it |
|---|---|---|
| _(none)_ | `borrow` — caller retains ownership | The common case — write nothing |
| `borrow` | Explicit borrow | Useful for documentation clarity |
| `own` | Transfer — caller's binding becomes invalid | When the function consumes the value |
| `share` | Shared reference (`Arc<T>`) — both caller and callee hold it | For concurrent shared access |

```atlas
// Borrow is implicit — TypeScript-style works as-is
fn process(data: string): string {
    return data.toUpperCase();
}

// Explicit own: caller loses the binding after this call
fn consume(own buffer: Buffer): void {
    buffer.flush();
}

// Explicit share: caller and callee both keep a reference
fn cache(share conn: Connection): void {
    store(conn);
}
```

**For AI generation:** Write parameters without any ownership annotation. The implicit `borrow` default covers virtually all cases. Add `own` or `share` only when the ownership transfer IS the point of the function.

---

## Generic Functions

```atlas
fn identity<T>(x: T): T {
    return x;
}

fn first<T>(arr: T[]): Option<T> {
    if arr.length() == 0 {
        return None;
    }
    return Some(arr[0]);
}
```

### Trait bounds on type parameters

Use `extends` for bounds (TypeScript-style, D-026, D-039):

```atlas
// Single bound
fn print_it<T extends Printable>(item: T): void {
    item.print();
}

// Multiple bounds (& separator)
fn copy_it<T extends Reader & Writer>(x: T): void {
    let data = x.read();
    x.write(data);
}
```

**Bound syntax:** `T extends Bound1 & Bound2`. Multiple type parameters: `<T, U extends Display>`.

---

## Anonymous Functions (Closures)

Anonymous functions use the same `fn` keyword but without a name. Return type annotation is optional.

```atlas
// With return type
let double: (number) => number = fn(x: number): number {
    return x * 2;
};

// Without return type — inferred
let triple = fn(x: number) { return x * 3; };

// Immediately passed as argument
let results = items.map(fn(x: number): number { return x * 2; });
```

**Syntax:**
```
fn(<params>)[: ReturnType] { <body> }
```

Anonymous function parameters follow the same `name: Type` convention, with optional ownership annotation. Default parameter values are supported in closures too.

**Arrow functions (`=>`) do not exist in Atlas.** There is no `(x) => x * 2` syntax. Use `fn(x: number) { return x * 2; }` or a tail expression:

```atlas
// Tail expression (no return keyword — last expression is the value)
let double = fn(x: number): number { x * 2 };
```

---

## Tail Expressions (Implicit Returns)

A block's last expression, when written without a trailing semicolon, becomes the block's return value. This works in function bodies and in any block:

```atlas
fn add(a: number, b: number): number {
    a + b   // no semicolon — this is the return value
}

fn classify(n: number): string {
    if n > 0 {
        "positive"   // tail of the if branch
    } else if n < 0 {
        "negative"
    } else {
        "zero"
    }
}
```

An explicit `return` statement is also always valid. Both styles are correct Atlas.

---

## Async Functions

Prefix with `async`. The return type must be `Result<T, E>` or another future-compatible type.

```atlas
async fn fetch_user(id: string): Result<User, string> {
    let response = await http.get("/users/" + id);
    return Json.parse<User>(response.body());
}
```

`async` must immediately precede `fn`. `async` is not a standalone keyword.

Inside an async function, use `await` to resolve a future:

```atlas
let data = await network.request(url);
```

---

## `self` in impl Methods

Methods inside `impl` blocks receive the struct instance as `self`. The type annotation for `self` is optional — the impl block provides the context.

```atlas
impl Point {
    fn distance(self): number {
        math.sqrt(self.x * self.x + self.y * self.y).unwrap()
    }

    fn scale(self, factor: number): Point {
        return Point { x: self.x * factor, y: self.y * factor };
    }
}
```

Ownership annotations apply to `self` too:

```atlas
impl Buffer {
    fn consume(own self): void { ... }    // takes ownership of self
    fn inspect(borrow self): string { ... } // borrows self (default)
}
```

---

## Visibility Modifiers

Functions declared at the top level or inside impl/trait blocks can carry a visibility modifier:

| Modifier | Scope |
|---|---|
| _(none)_ | Private — not exported |
| `pub` | Public — exported / accessible from other modules |
| `private` | Explicitly file-private |
| `internal` | Module-internal |

```atlas
pub fn exported(): void { }
fn internal_helper(): void { }
```

Functions inside `export { }` blocks are implicitly public.

---

## `export` and `import`

```atlas
// math.atl
export fn add(x: number, y: number): number {
    return x + y;
}

// main.atl
import { add } from "./math";
let result = add(1, 2);
```

`export fn` is the standard pattern. Top-level `export { name }` form is also supported.

---

## Type Predicates

Named functions can declare a type predicate for type narrowing:

```atlas
fn isString(x: string | number): bool is x: string {
    return typeof x == "string";
}
```

After a call to `isString(val)`, the typechecker narrows `val` to `string` in the truthy branch.

---

## `defer`

`defer` schedules work to run when the enclosing scope exits, in LIFO order:

```atlas
fn with_cleanup(): void {
    defer { console.log("cleanup"); }
    defer { console.log("also cleanup"); }

    // ... rest of function
    // On exit: "also cleanup" runs first, then "cleanup"
}

// Single-expression form
defer resource.close();
```

---

## Gotchas

**Return type is required for named functions.** Omitting the `: ReturnType` part is a compile error. Use `: void` for procedures.

**`->` is rejected.** Write `: ReturnType` (colon), not `-> ReturnType` (arrow). The error message includes a migration hint.

**No arrow function syntax.** There is no `(x) => expr` shorthand. Use `fn(x: T) { expr }` with a tail expression.

**`async` must precede `fn` directly.** `async let`, `async { }`, etc. are not valid. Only `async fn` exists.

**Expression statements require semicolons.** `f(x)` as a standalone statement must be written `f(x);`. Without the semicolon the parser cannot determine if it is a statement or a tail expression and will fail.

**Rest parameter must be last.** `fn foo(...rest: number[], extra: string)` is an error.
