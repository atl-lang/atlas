# Closures and Anonymous Functions

Atlas closures are anonymous functions written with the `fn` keyword. They capture their enclosing scope and can be stored in variables, passed as arguments, and returned from functions.

## Syntax

```atlas
fn(param: Type, param: Type): ReturnType { body }
```

The return type annotation is optional. When omitted, the type is inferred:

```atlas
// With return type
let double = fn(x: number): number { return x * 2; };

// Inferred return type
let greet = fn(name: string) { return "Hello, " + name; };
```

Parameters require a name and type annotation, following the same rules as named functions. Ownership annotations (`borrow`, `own`, `share`) are supported; `borrow` is the implicit default when omitted.

## Calling Closures

Closures are called the same way as any function:

```atlas
let add = fn(a: number, b: number): number { return a + b; };
let result = add(3, 4);  // 7
```

## Closure Captures

Closures capture variables from their enclosing scope. Atlas closures are lexically scoped — they see the bindings that exist at the point of definition:

```atlas
let base = 10;
let addBase = fn(x: number): number { return x + base; };
console.log(addBase(5).toString());  // 15
```

Closures capture by reference to the enclosing frame. Mutations to captured mutable bindings are visible:

```atlas
let mut count = 0;
let increment = fn(): void { count = count + 1; };
increment();
increment();
console.log(count.toString());  // 2
```

## Type Signatures

The type of a closure is a function type: `(ParamTypes) -> ReturnType`.

```atlas
// Explicit type annotation on the variable
let transform: (number) -> number = fn(x: number): number { return x * x; };

// Function types in struct/type alias definitions
type Handler = (string) -> void;
type Mapper<T, U> = (T) -> U;
```

Function types in type annotations use `->` syntax:

```atlas
// Parameter typed as a function
fn apply(f: (number) -> number, x: number): number {
    return f(x);
}
```

## Higher-Order Functions

Functions that take or return closures are first-class in Atlas.

### Passing Closures as Arguments

```atlas
fn apply(f: (number) -> number, x: number): number {
    return f(x);
}

let result = apply(fn(x: number): number { return x * 3; }, 7);
// result = 21
```

```atlas
fn runTwice(f: () -> void): void {
    f();
    f();
}

runTwice(fn(): void { console.log("hello"); });
// hello
// hello
```

### Closures over Stdlib Array Methods

Many stdlib methods accept closures. For example, `array.map` and `array.filter`:

```atlas
let nums = [1, 2, 3, 4, 5];

let doubled = nums.map(fn(x: number): number { return x * 2; });
// [2, 4, 6, 8, 10]

let evens = nums.filter(fn(x: number): boolean { return x % 2 == 0; });
// [2, 4]
```

### Returning Closures

Functions can return closures. The return type is a function type:

```atlas
fn makeAdder(base: number): (number) -> number {
    return fn(x: number): number { return x + base; };
}

let add5 = makeAdder(5);
console.log(add5(10).toString());  // 15
console.log(add5(3).toString());   // 8
```

## Closures Stored in Variables

Closures stored in variables can be reassigned (if mutable) and passed around like any other value:

```atlas
let mut handler = fn(msg: string): void { console.log("default: " + msg); };

// Reassign to a different closure
handler = fn(msg: string): void { console.log("custom: " + msg); };

handler("test");  // custom: test
```

## Default Parameters in Closures

Closures support default parameter values, following the same syntax as named functions (B39):

```atlas
let greet = fn(name: string = "World"): string {
    return "Hello, " + name + "!";
};

console.log(greet());         // Hello, World!
console.log(greet("Atlas"));  // Hello, Atlas!
```

## Closures with Ownership Annotations

Ownership annotations on closure parameters follow the same semantics as named function parameters (D-040). `borrow` is the default when no annotation is written:

```atlas
// These are equivalent:
let f = fn(x: number): number { return x + 1; };
let g = fn(borrow x: number): number { return x + 1; };

// Take ownership:
let consume = fn(own s: string): string { return s + "!"; };

// Shared reference:
let read = fn(share data: string): number { return data.length(); };
```

## Immediate Invocation

A closure can be defined and called in the same expression:

```atlas
let result = fn(x: number): number { return x * x; }(7);
// result = 49
```

## Closures as Struct Fields

Closures can be stored inside structs, enabling callback-style patterns:

```atlas
struct EventHandler {
    on_click: (string) -> void,
    on_key: (number) -> void,
}

let handler = EventHandler {
    on_click: fn(id: string): void { console.log("clicked: " + id); },
    on_key: fn(code: number): void { console.log("key: " + code.toString()); },
};

handler.on_click("button-1");
handler.on_key(65);
```

## Summary

| Feature | Syntax |
|---------|--------|
| Define closure | `fn(x: T): R { body }` |
| Inferred return | `fn(x: T) { body }` |
| Closure type | `(T) -> R` |
| Parameter default | `fn(x: T = default) { body }` |
| Ownership on param | `fn(borrow x: T)`, `fn(own x: T)`, `fn(share x: T)` |
| Store in variable | `let f: (T) -> R = fn(...) { ... };` |
| Pass as argument | `apply(fn(x: number): number { x + 1 }, 5)` |
| Return a closure | `fn make(): (T) -> R { return fn(x: T): R { ... }; }` |
