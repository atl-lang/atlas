# Atlas Gotchas — Things AI Agents Get Wrong

These are the most common mistakes. Read this before generating any Atlas code.

## 1. Wrong Return Type Syntax — Use `:` Not `->`

```atlas
// WRONG — Rust-style arrow
fn add(x: number, y: number) -> number { x + y }

// RIGHT — TypeScript-style colon
fn add(x: number, y: number): number { x + y }
```

Method docs also use `:`:
```
// WRONG:  .push(value: T) -> T[]
// RIGHT:  .push(value: T): T[]
```

## 2. Using `print()` Instead of `console.log()`

```atlas
// WRONG — print/println don't exist
print("hello");
println!("hello");

// RIGHT
console.log("hello");
console.log(`value: ${x}`);
```

## 3. Using `arrayPush(arr, x)` Instead of `arr.push(x)`

```atlas
// WRONG — bare function forms were removed
arrayPush(arr, 4);
arrayPop(arr);

// RIGHT — method syntax
arr.push(4);
arr.pop();
```

## 4. Using `fs.readFile()` Instead of `file.readText()`

```atlas
// WRONG — fs namespace doesn't exist
let text = fs.readFile("file.txt");
fs.writeFile("out.txt", content);
fs.readdir("./src");

// RIGHT — file namespace, different method names
let text = file.readText("file.txt");
file.writeText("out.txt", content);
file.readdir("./src");
```

Read/write lives in `io` and `file` namespaces. Basic text I/O: `file.readText`, `file.writeText`.

## 5. Using `fn main()` as Entry Point

```atlas
// WRONG — fn main does NOT auto-execute
fn main(): void {
    console.log("hello");
}

// RIGHT — top-level code runs directly
console.log("hello");
```

Atlas has no magic entry point. Code at the top level of a file runs when the file is executed.

## 6. Using `{}` in Template Strings Instead of `${}`

```atlas
// WRONG — {} does not interpolate
let msg = `Hello {name}`;

// RIGHT — ${} interpolates
let msg = `Hello ${name}`;
```

## 7. Using `JSON.parse()` Instead of `json.parse()`

```atlas
// WRONG — capitalized JSON from JavaScript
let data = JSON.parse(text);
let text = JSON.stringify(value);

// RIGHT — lowercase j in Atlas
let data = json.parse(text);
let text = json.stringify(value);
```

## 8. No Interpreter

**Wrong:** "The Atlas interpreter evaluates..."
**Right:** Atlas compiles to bytecode and runs on the VM. There is no interpreter. Use "compiler," "VM," "runtime" — never "interpreter."

## 9. Semicolons Are Required

```atlas
// WRONG — will fail to parse
fn process(x: number): number {
    let y = x * 2
    y + 1
}

// RIGHT
fn process(x: number): number {
    let y = x * 2;
    y + 1  // last expression — no semicolon (implicit return)
}
```

Every statement needs `;`. The last expression in a block is the implicit return — it does NOT need `;`.

## 10. CoW Collections — Always Capture Return Values

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

This applies to ALL collection types: Map, Set, Queue, Stack, and Array.

## 11. Map/Set Construction

```atlas
// WRONG
let m = {};              // creates a struct record, not a Map
let m = hashMapNew();    // old name, removed
let m = HashMap<string, number>();  // not valid

// RIGHT
let m = new Map<string, number>();
let s = new Set<string>();
```

## 12. `fn` Parameters Default to `borrow` — No Annotation Needed

```atlas
// These are equivalent for everyday code
fn process(data: string): string { data.toUpperCase() }
fn process(borrow data: string): string { data.toUpperCase() }

// Only annotate explicitly when you need own or share semantics:
fn consume(own data: string): string { ... }
fn share(share data: string): string { ... }
```

## 13. `bool` Not `boolean`

```atlas
// WRONG
let flag: boolean = true;

// RIGHT
let flag: bool = true;
```

## 14. Tuple Syntax — Parens, Not Brackets

```atlas
// WRONG
let pair: [string, number] = ["hello", 42];

// RIGHT
let pair: (string, number) = ("hello", 42);
```

## 15. Trait Inheritance — Comma Style

```atlas
// WRONG (Rust style)
trait MyTrait: TraitA + TraitB {}

// RIGHT (TypeScript comma style, D-026)
trait MyTrait extends TraitA, TraitB {}
```

## 16. Generic Bounds — `&` Not `+` At Type Level

```atlas
// WRONG
fn process<T: Foo + Bar>(x: T): void {}

// RIGHT (D-039 — TypeScript & style)
fn generic<T extends Foo & Bar>(x: T): void {}
```

## 17. No `loop` Keyword — Use `while true`

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

## 18. No Arrow Function Shorthand

```atlas
// WRONG — arrow functions don't exist as standalone expressions
let double = (x: number) => x * 2;

// RIGHT — use fn keyword
let double = fn(x: number): number { x * 2 };
```

## 19. `Future<T>` Not `Promise<T>`

```atlas
// WRONG
async fn fetch(): Promise<string> { ... }

// RIGHT
async fn fetch(): Future<string> { ... }
// or let Atlas infer it
async fn fetch(): string { ... }
```

## 20. `if` Has No Parens

```atlas
// WRONG
if (x > 0) { ... }

// RIGHT
if x > 0 { ... }
```
