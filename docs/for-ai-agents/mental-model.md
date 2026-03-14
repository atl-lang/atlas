# Atlas Mental Model

Understanding this file will save you from 90% of mistakes when generating Atlas code.

## The Core Identity

Atlas is **two things combined:**

```
TypeScript surface  +  Rust runtime model
─────────────────────────────────────────
Module system       →  TypeScript
Type annotations    →  TypeScript (: return type, <T extends Foo & Bar>)
Generics syntax     →  TypeScript
Import/export       →  TypeScript
Class-like traits   →  TypeScript style, Rust semantics

Value semantics     →  Rust (CoW, ownership)
Error handling      →  Rust (Result/Option)
Memory model        →  Rust-inspired (no GC pauses)
Concurrency         →  Rust/Tokio (async/await, channels)
```

**Decision filter:** "TypeScript has an answer → use it exactly. TypeScript has no answer → design Atlas-native, minimal tokens."

## Syntax Surface (TypeScript Style)

Return types use `:`, not `->`:

```atlas
// RIGHT — TypeScript style
fn add(x: number, y: number): number { x + y }

// WRONG — Rust style
fn add(x: number, y: number) -> number { x + y }
```

Generics use TypeScript bounds syntax:

```atlas
fn process<T extends Serializable & Comparable>(items: T[]): T[] { ... }
```

## Execution Model (Critical)

```
Source (.atlas)
     ↓
  Lexer → tokens
     ↓
  Parser → AST
     ↓
  Binder → name resolution
     ↓
  TypeChecker → type verification
     ↓
  Compiler → bytecode
     ↓
  Optimizer → constant folding, dead code
     ↓
  VM → execution
```

**There is NO interpreter.** Every program goes through this entire pipeline. `atlas run` = compile + execute. Never say "the interpreter does X."

## Single Execution Path

The compiler and VM are the only execution path. There is no fallback interpreter, no REPL evaluator, no "fast mode." When someone says "Atlas runs X," they mean: lexer → parser → type checker → compiler → VM.

## Value Semantics (CoW)

Atlas collections use **Copy-on-Write**. Mutation methods return a new value — they do NOT mutate in place.

```atlas
// WRONG — result discarded
let m = new Map<string, number>();
m.set("key", 1);             // m is still empty!
console.log(m.get("key"));   // null

// CORRECT — capture the return value
let m = new Map<string, number>();
let m2 = m.set("key", 1);
console.log(m2.get("key")); // Some(1)
```

This applies to: `Map`, `Set`, `Queue`, `Stack`. All collection mutations return the updated collection.

## Type System Mental Model

Atlas types map almost directly from TypeScript:

| TypeScript | Atlas | Notes |
|-----------|-------|-------|
| `string` | `string` | identical |
| `number` | `number` | identical (f64) |
| `boolean` | `bool` | different name |
| `null` | `null` | identical |
| `undefined` | — | doesn't exist in Atlas |
| `void` | `void` | for functions with no return |
| `never` | `never` | unreachable code |
| `T[]` | `T[]` | identical |
| `[T, U]` | `(T, U)` | tuple: parens not brackets |
| `T \| U` | `T \| U` | identical |
| `<T extends Foo>` | `<T extends Foo>` | identical |
| `Map<K, V>` | `Map<K, V>` | construct with `new Map<K,V>()` |
| `Promise<T>` | `Future<T>` | Atlas name for async values |
| `null \| T` | `Option<T>` | prefer Option over union |
| — | `Result<T, E>` | explicit error handling |

## Ownership Annotations

Atlas has ownership annotations but they're **invisible in everyday code**. Parameters default to `borrow` — write nothing for the common case.

```atlas
// Everyday — write nothing (defaults to borrow)
fn process(data: string): string { data.toUpperCase() }

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
let result: Result<string, string> = file.readText("path");
match result {
    Ok(content) => process(content),
    Err(e) => console.log(e),
}

// ? operator — propagate errors up
fn parseConfig(path: string): Result<string, string> {
    let text = file.readText(path)?;   // returns Err if failed
    let data = json.parse(text)?;      // returns Err if failed
    Ok(data)
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
import { file } from "atlas:file";
import { http } from "atlas:http";
```

No default exports. No re-exports (`export { x } from "./y"` not supported).

## Traits vs Classes

Atlas has traits, not classes. Traits are the unit of polymorphism:

```atlas
trait Drawable {
    fn draw(borrow self): void;
    fn color(borrow self): string { "black" }  // default impl
}

trait Shape extends Drawable {  // comma for multiple: extends A, B
    fn area(borrow self): number;
}

struct Circle { radius: number; }

impl Shape for Circle {
    fn area(borrow self): number { math.PI * self.radius * self.radius }
}

impl Drawable for Circle {
    fn draw(borrow self): void { console.log(`drawing circle r=${self.radius}`); }
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
