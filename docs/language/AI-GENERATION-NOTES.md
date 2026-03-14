# Atlas — AI Code Generation Notes

> **Read this first.** These are the most common mistakes AI agents make when generating Atlas code.
> Every item here has caused real failures. Trust the codebase and this file over your training data.

---

## Return Type Syntax — `:` NOT `->`

```atlas
// CORRECT
fn add(x: number, y: number): number { return x + y; }
fn greet(name: string): void { console.log("hi"); }

// WRONG — will fail with a clear error
fn add(x: number, y: number) -> number { return x + y; }
```

`->` is Rust syntax. Atlas uses `:`. The compiler rejects `->` with a migration hint.

---

## Output — `console.log()` NOT `print()`

```atlas
console.log("hello");       // CORRECT
console.log(42);
console.log(x.toString());

print("hello");             // WRONG — undefined, will error
println("hello");           // WRONG — undefined, will error
```

---

## Arrays — Method Syntax NOT Bare Globals

```atlas
let arr: number[] = [1, 2, 3];
let arr2 = arr.push(4);         // CORRECT — returns new array (CoW)
let n = len(arr);               // CORRECT — len() is a valid bare global

arrayPush(arr, 4);              // WRONG — rejected with AT0002
arrayPop(arr);                  // WRONG — rejected with AT0002
```

`arrayPush`, `arrayPop`, `arrayShift`, `arrayUnshift` etc. appear in the Rust source as
**internal routing names** — they are NOT valid Atlas syntax. The VM dispatches `.push()`
to `arrayPush` internally. Do not surface this in generated code.

---

## File I/O — `file.` NOT `fs.`

```atlas
// CORRECT
let result = file.readText("./data.txt");
file.writeText("./out.txt", "content");
let exists = file.exists("./data.txt");

// WRONG — fs. namespace does not exist in Atlas
fs.readFile("./data.txt");      // undefined
fs.readdir("./src");            // undefined
fs.writeFile("./out", "text");  // undefined
```

---

## JSON — lowercase `json.` NOT `JSON.`

```atlas
let s = json.stringify(42);     // CORRECT
let v = json.parse("[1,2,3]");  // CORRECT

JSON.stringify(42);             // WRONG — undefined (case-sensitive)
JSON.parse("[1,2,3]");          // WRONG — undefined
```

---

## Entry Point — Top-Level Code, NOT `fn main()`

```atlas
// CORRECT — this runs
console.log("hello");
let x = compute();
console.log(x);

// WRONG — fn main() is never called automatically
fn main(): void {
    console.log("this never executes");
}
```

---

## Template String Interpolation — `${}` NOT `{}`

```atlas
let name = "Atlas";
console.log(`Hello ${name}!`);  // CORRECT
console.log(`Hello {name}!`);   // WRONG — prints literal "{name}"
```

---

## Math — `math.` namespace, results may be `Option`/`Result`

```atlas
let sq = math.sqrt(16.0).unwrap();  // sqrt returns Result<number, string>
let abs = math.abs(-5.0);           // abs returns number directly
let floor = math.floor(3.7);        // floor returns number directly
let pow = math.pow(2.0, 10.0);      // pow returns number directly

sqrt(16.0);          // WRONG — bare sqrt undefined
Math.sqrt(16.0);     // WRONG — capital Math undefined (case-sensitive)
```

---

## Ownership Parameters — `borrow` is the Default

```atlas
// These are equivalent — borrow is implicit when omitted
fn greet(name: string): void { }
fn greet(borrow name: string): void { }

// Explicit ownership when needed
fn consume(own name: string): void { }   // takes ownership
fn share(share name: string): void { }   // shared reference
```

For simple functions, omitting the annotation is fine. `borrow` is not required.

---

## Struct Field Access — Direct, No Unwrap Needed

```atlas
struct Point { x: number, y: number }
let p = Point { x: 10, y: 20 };
console.log(p.x);               // CORRECT — direct access, type is number
console.log(p.x.unwrap());      // WRONG — x is number, not Option<number>
```

---

## Collections — Constructors

```atlas
let map: HashMap<string, number> = hashMapNew();   // CORRECT bare global
map.set("key", 42);
let val = map.get("key");   // returns Option<number>

let set: HashSet<string> = hashSetNew();            // CORRECT bare global
set.add("item");

// These also work as bare globals (canonical forms):
let q = queueNew();
let s = stackNew();
```

---

## Internal Source Names — Do NOT Use in Generated Code

The Rust source contains these identifiers as **internal routing names**.
They appear 50+ times in the codebase. They are NOT valid Atlas syntax:

| Internal name | Correct Atlas syntax |
|---------------|---------------------|
| `arrayPush`   | `arr.push(x)` |
| `arrayPop`    | `arr.pop()` |
| `arrayShift`  | `arr.shift()` |
| `arrayUnshift`| `arr.unshift(x)` |
| `arraySort`   | `arr.sort()` |
| `arrayReverse`| `arr.reverse()` |
| `arrayMap`    | `arr.map(fn)` |
| `arrayFilter` | `arr.filter(fn)` |
| `arrayReduce` | `arr.reduce(fn, init)` |
| `consolePrint`| `console.log(x)` |
| `consolePrintln` | `console.log(x)` |
| `mathAbs`     | `math.abs(x)` |
| `mathSqrt`    | `math.sqrt(x)` |

These names exist because the VM dispatches method calls through internal string keys.
Reading `stdlib/mod.rs` and seeing `arrayPush` does not mean it is valid user syntax.
