# Atlas Gotchas — Things AI Agents Get Wrong

These are the most common mistakes. Read this before generating any Atlas code.

## 1. No Interpreter

**Wrong:** "The Atlas interpreter evaluates..."
**Right:** Atlas compiles to bytecode and runs on the VM. There is no interpreter. Use "compiler," "VM," "runtime" — never "interpreter."

## 2. Semicolons Are Required

```atlas
// WRONG — will fail to parse
fn process(x: number): number {
  let y = x * 2
  y + 1
}

// RIGHT
fn process(x: number): number {
  let y = x * 2;
  y + 1  // last expression — no semicolon needed (implicit return)
}
```

Every statement needs `;`. The last expression in a block is the implicit return — it does NOT need `;`.

## 3. CoW Collections — Always Capture Return Values

```atlas
// WRONG — mutation is discarded
let m = new Map<string, number>();
m.set("a", 1);           // m unchanged!
m.get("a");              // null

// RIGHT
let m = new Map<string, number>();
let m = m.set("a", 1);   // rebind (shadowing OK)
m.get("a");              // Some(1)
```

This applies to ALL collection types: Map, Set, Queue, Stack.

## 4. Map/Set Construction

```atlas
// WRONG
let m = {};              // creates a struct record, not a Map
let m = hashMapNew();    // old name, removed
let m = HashMap<string, number>();  // not valid

// RIGHT
let m = new Map<string, number>();
let s = new Set<string>();
let q = new Queue<number>();
let k = new Stack<number>();
```

## 5. No `undefined` — Use Option

```atlas
// WRONG — undefined doesn't exist in Atlas
let x: string | undefined = undefined;

// RIGHT
let x: Option<string> = None;
let x: Option<string> = Some("value");
```

## 6. `bool` Not `boolean`

```atlas
// WRONG
let flag: boolean = true;

// RIGHT
let flag: bool = true;
```

## 7. Tuple Syntax — Parens, Not Brackets

```atlas
// WRONG
let pair: [string, number] = ["hello", 42];

// RIGHT
let pair: (string, number) = ("hello", 42);
```

## 8. Trait Inheritance — Comma Style

```atlas
// WRONG (Rust style)
trait MyTrait: TraitA + TraitB {}

// RIGHT (TypeScript comma style, D-026)
trait MyTrait extends TraitA, TraitB {}
```

## 9. Generic Bounds — `&` Not `+` At Type Level

```atlas
// WRONG
fn process<T: Foo + Bar>(x: T): void {}

// RIGHT (D-039 — TypeScript & style)
fn generic<T extends Foo & Bar>(x: T): void {}
```

## 10. No `loop` Keyword — Use `while true`

```atlas
// WRONG — loop keyword doesn't exist
loop {
  if done { break; }
}

// RIGHT
while true {
  if done { break; }
}
```

## 11. No Arrow Function Shorthand

```atlas
// WRONG — arrow functions don't exist as standalone expressions
let double = (x: number) => x * 2;

// RIGHT — use fn keyword
let double = fn(x: number): number { x * 2 };
```

## 12. Queue/Stack Pop Returns a Tuple

```atlas
// WRONG — pop doesn't return the value directly
let value = queue.dequeue();

// RIGHT — returns [Option<T>, Queue<T>]
let [value, queue] = queue.dequeue();
let [top, stack] = stack.pop();
```

## 13. No Re-exports

```atlas
// WRONG — not supported
export { foo } from "./other";

// RIGHT — import then export
import { foo } from "./other";
export fn wrapFoo(): void { foo(); }
```

## 14. HashMap/HashSet Type Names Are Gone

```atlas
// WRONG — old names
let m: HashMap<string, number> = ...;
let s: HashSet<string> = ...;

// RIGHT
let m: Map<string, number> = new Map<string, number>();
let s: Set<string> = new Set<string>();
```

## 15. `Future<T>` Not `Promise<T>`

```atlas
// WRONG
async fn fetch(): Promise<string> { ... }

// RIGHT
async fn fetch(): Future<string> { ... }
// or just let Atlas infer it
async fn fetch(): string { ... }
```

## 16. No Default Exports

```atlas
// WRONG — Atlas has no default exports
export default fn main(): void { ... }

// RIGHT
export fn main(): void { ... }
```

## 17. Enum Variant Access Is Qualified

```atlas
// WRONG
let d = North;       // bare variant

// RIGHT
let d = Direction::North;   // qualified
```

## 18. `null` Is a Type and Value

```atlas
// In Atlas, null is both a type keyword and a value
let x: string | null = null;   // union with null
let y: Option<string> = None;  // prefer Option over | null
```

## 19. Method Calls on stdlib Use Namespace Syntax

```atlas
// WRONG — bare globals were removed (B20-B35)
let parsed = parseJSON(text);
let encoded = base64Encode(data);

// RIGHT — namespace.method() syntax
let parsed = Json.parse(text);
let encoded = Encoding.base64Encode(data);
```

## 20. `if` Has No Parens

```atlas
// WRONG
if (x > 0) { ... }

// RIGHT
if x > 0 { ... }
```
