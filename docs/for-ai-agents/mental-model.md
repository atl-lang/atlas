# Atlas Mental Model

Understanding this file will save you from 90% of mistakes when generating Atlas code.

## The Core Identity

Atlas is **two things combined:**

```
TypeScript surface  +  Rust runtime model
─────────────────────────────────────────
Module system       →  TypeScript
Type annotations    →  TypeScript
Generics syntax     →  TypeScript (<T extends Foo & Bar>)
Import/export       →  TypeScript
Class-like traits   →  TypeScript style, Rust semantics

Value semantics     →  Rust (CoW, ownership)
Error handling      →  Rust (Result/Option)
Memory model        →  Rust-inspired (no GC pauses)
Concurrency         →  Rust/Tokio (async/await, channels)
```

**Decision filter:** "TypeScript has an answer → use it exactly. TypeScript has no answer → design Atlas-native, minimal tokens."

## Execution Model (Critical)

```
Source (.atlas)
     ↓
  Lexer (token.rs)
     ↓
  Parser (parser/) → AST
     ↓
  Binder (binder.rs) → name resolution
     ↓
  TypeChecker (typechecker/) → type verification
     ↓
  Compiler (compiler/) → bytecode
     ↓
  Optimizer (optimizer/) → constant folding, dead code
     ↓
  VM (vm/mod.rs) → execution
```

**There is NO interpreter.** Every program goes through this entire pipeline. `atlas run` = compile + execute, not "interpret."

## Value Semantics (CoW)

Atlas collections use **Copy-on-Write**. Mutation methods return a new value — they do NOT mutate in place.

```atlas
// WRONG — result discarded
let m = new Map<string, number>();
m.set("key", 1);           // m is still empty!
console.log(m.get("key")); // null

// CORRECT — capture the return value
let m = new Map<string, number>();
let m2 = m.set("key", 1);
console.log(m2.get("key")); // Some(1)

// Arrays too
let arr = [1, 2, 3];
arr.push(4);        // WRONG — discarded
let arr2 = arr.push(4);  // OK? Actually arrays use different semantics
// Check docs/stdlib/array.md for exact array mutation behavior
```

This applies to: `Map`, `Set`, `Queue`, `Stack`. All collection mutations return the updated collection.

## Type System Mental Model

Atlas types map almost directly from TypeScript:

| TypeScript | Atlas | Notes |
|-----------|-------|-------|
| `string` | `string` | identical |
| `number` | `number` | identical (f64) |
| `boolean` | `bool` | slightly different name |
| `null` | `null` | identical |
| `undefined` | — | doesn't exist in Atlas |
| `void` | `void` | for functions with no return |
| `never` | `never` | unreachable code |
| `T[]` | `T[]` | identical |
| `[T, U]` | `(T, U)` | tuple: parens not brackets |
| `T \| U` | `T \| U` | identical |
| `T & U` | `T & U` | intersection (type level only) |
| `<T extends Foo>` | `<T extends Foo>` | identical |
| `Map<K, V>` | `Map<K, V>` | but construct with `new Map<K,V>()` |
| `Promise<T>` | `Future<T>` | Atlas name for async values |
| `null \| T` | `Option<T>` | prefer Option over union |
| — | `Result<T, E>` | explicit error handling |

## Ownership Annotations

Atlas has ownership annotations but they're **invisible in everyday code**. You only write them in low-level/systems code:

```atlas
// Everyday — write nothing (defaults to borrow)
fn process(data: string): string { ... }

// Systems-level — explicit ownership
fn consume(own data: string): string { ... }
fn read(borrow data: string): string { ... }
fn share(share data: string): string { ... }
```

D-040: "bare params default to borrow — write nothing for the common case."

## Error Handling Model

Atlas uses **Result and Option**, not exceptions:

```atlas
// Option<T> — value might not exist
let found: Option<string> = map.get("key");
let value = found.unwrapOr("default");

// Result<T, E> — operation might fail
let result: Result<string, string> = fs.readFile("path");
match result {
  Ok(content) => process(content),
  Err(e) => console.error(e),
}

// ? operator — propagate errors up
fn parse_config(path: string): Result<Config, string> {
  let text = fs.readFile(path)?;   // returns Err if failed
  let json = Json.parse(text)?;    // returns Err if failed
  Ok(build_config(json))
}
```

## Module System

Identical to TypeScript ES modules:

```atlas
// Named exports
export fn foo(): void { ... }
export const BAR = 42;
export struct Baz { ... }

// Named imports
import { foo, BAR } from "./module";

// Namespace import
import * as utils from "./utils";

// Stdlib imports use atlas: prefix
import { fs } from "atlas:fs";
import { http } from "atlas:http";
```

No default exports. No re-exports (`export { x } from "./y"` not supported). No dynamic imports.

## Traits vs Classes

Atlas has traits, not classes. Traits are the unit of polymorphism:

```atlas
trait Drawable {
  fn draw(self): void;
  fn color(self): string { "black" }  // default impl
}

trait Shape extends Drawable {  // comma for multiple: extends A, B
  fn area(self): number;
}

struct Circle { radius: number; }

impl Shape for Circle {
  fn area(self): number { Math.PI * self.radius * self.radius }
}

impl Drawable for Circle {
  fn draw(self): void { console.log(`drawing circle r=${self.radius}`); }
}
```

## Async Model

Based on Tokio. Async functions return `Future<T>`:

```atlas
async fn fetch(url: string): Result<string, string> {
  let response = await http.get(url);
  if response.status() == 200 {
    Ok(response.body())
  } else {
    Err(`HTTP ${response.status()}`)
  }
}

// Spawn concurrent work
let handle = task.spawn(async fn(): number {
  expensiveComputation()
});
let result = await task.join(handle);
```

Workers are isolated — each worker has its own VM instance. Values cross worker boundaries via channels or join results, not shared memory.
