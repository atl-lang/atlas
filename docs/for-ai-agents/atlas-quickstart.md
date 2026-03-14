# Atlas Quickstart — Write Atlas Code in 5 Minutes

## Hello World

```atlas
console.log("Hello, Atlas!");
```

Top-level code runs directly. `fn main()` does NOT auto-execute — Atlas has no magic entry point. Just write code at the top level.

File extension: `.atlas` (also `.atl`)

Run it: `atlas run hello.atlas`

## Variables

```atlas
let x = 10;            // immutable binding, type inferred
let mut y = 20;        // mutable binding (let mut)
var z = 30;            // also mutable (var = let mut shorthand)
const PI = 3.14159;    // constant, must be initialized
```

With explicit types:

```atlas
let name: string = "Atlas";
let count: number = 0;
let active: bool = true;
```

## Functions

```atlas
fn add(x: number, y: number): number {
    return x + y;
}

// Last expression is implicit return (no semicolon)
fn multiply(x: number, y: number): number {
    x * y
}

// Default parameters
fn greet(name: string, greeting: string = "Hello"): string {
    `${greeting}, ${name}!`
}

// Ownership annotations (default is borrow — omit for everyday code)
fn process(borrow data: string): string {
    data.toUpperCase()
}
```

## Output

```atlas
console.log("hello");            // print to stdout
console.log("value: " + x);     // string concatenation
console.log(`value: ${x}`);     // template literal (preferred)
```

Do NOT use `print()` or `println()` — those don't exist. Always use `console.log()`.

## Arrays

```atlas
let arr: number[] = [1, 2, 3];

arr.push(4);           // add to end
arr.pop();             // remove from end
len(arr);              // length (builtin function)
arr[0];                // index access (zero-based)

// Higher-order
let doubled = arr.map(fn(x: number): number { x * 2 });
let evens = arr.filter(fn(x: number): bool { x % 2 == 0 });
```

## Structs

```atlas
struct Point {
    x: number;
    y: number;
}

let p = Point { x: 1.0, y: 2.0 };
console.log(p.x);  // 1

impl Point {
    fn dist(borrow self): number {
        math.sqrt(self.x * self.x + self.y * self.y).unwrap()
    }
}

console.log(p.dist());
```

## Enums

```atlas
enum Direction {
    North,
    South,
    East,
    West,
}

enum Shape {
    Circle(number),
    Rectangle(number, number),
}

let d = Direction::North;
let s = Shape::Circle(5.0);
```

## Pattern Matching

```atlas
match direction {
    Direction::North => console.log("going north"),
    Direction::South => console.log("going south"),
    _ => console.log("going somewhere"),
}

// Match on Result
match result {
    Ok(value) => console.log(`got: ${value}`),
    Err(e) => console.log(`error: ${e}`),
}

// Guard clauses
match n {
    x if x < 0 => "negative",
    0 => "zero",
    _ => "positive",
}
```

## Control Flow

```atlas
// if/else — no parens around condition
if x > 0 {
    console.log("positive");
} else if x < 0 {
    console.log("negative");
} else {
    console.log("zero");
}

// if as expression
let label = if score >= 90 { "A" } else { "B" };

// for/in
for item in items {
    console.log(item);
}

for i in 0..10 {
    console.log(i);
}

// while
while count < 10 {
    count = count + 1;
}
```

## Option and Result

```atlas
// Option — value might not exist
let maybe: Option<string> = Some("hello");
let nothing: Option<string> = None;

maybe.isSome();              // true
maybe.unwrapOr("default");   // "hello"
nothing.unwrapOr("default"); // "default"

// Result — operation might fail
let ok: Result<number, string> = Ok(42);
let fail: Result<number, string> = Err("something went wrong");

match ok {
    Ok(v) => console.log(v),
    Err(e) => console.log(`error: ${e}`),
}

// ? operator propagates errors
fn parse(input: string): Result<number, string> {
    let n = parseInt(input).okOr("not a number")?;
    Ok(n * 2)
}
```

## Traits

```atlas
trait Animal {
    fn speak(borrow self): string;
    fn name(borrow self): string;

    fn describe(borrow self): string {  // default implementation
        `${self.name()} says ${self.speak()}`
    }
}

trait Domestic extends Animal {  // trait inheritance
    fn owner(borrow self): string;
}

struct Dog { name: string; }

impl Animal for Dog {
    fn speak(borrow self): string { "woof" }
    fn name(borrow self): string { self.name }
}
```

## Modules

```atlas
// math.atlas
export fn square(x: number): number { x * x }
export const PI: number = 3.14159;

// main.atlas
import { square, PI } from "./math";
import * as utils from "./utils";

console.log(square(4));    // 16
console.log(PI);           // 3.14159
```

## String Interpolation

```atlas
let name = "Atlas";
let version = 3;

// Use backticks and ${expr} — NOT {expr}
let msg = `Hello from ${name} v${version}!`;

// WRONG:
// let msg = `Hello from {name}`;    // {} does not interpolate
// let msg = "Hello from " + name;   // works but less readable
```
