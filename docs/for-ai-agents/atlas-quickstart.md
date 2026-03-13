# Atlas Quickstart — Write Atlas Code in 5 Minutes

## Hello World

```atlas
fn main(): void {
  console.log("Hello, Atlas!");
}
```

Run it: `atlas run hello.atlas`

## Variables

```atlas
let name: string = "Atlas";      // immutable binding
var count: number = 0;            // mutable binding
let inferred = "type inferred";   // type inference works
```

## Functions

```atlas
fn add(x: number, y: number): number {
  x + y  // last expression is implicit return
}

fn greet(name: string, greeting: string = "Hello"): string {
  `${greeting}, ${name}!`
}

// Async function
async fn fetchData(url: string): string {
  let response = await http.get(url);
  response.body
}
```

## Types

```atlas
// Primitives
let n: number = 42;
let s: string = "hello";
let b: bool = true;
let nothing: null = null;

// Arrays
let nums: number[] = [1, 2, 3];

// Tuples
let pair: (string, number) = ("age", 30);

// Option and Result
let maybe: Option<string> = Some("value");
let ok: Result<number, string> = Ok(42);
let err: Result<number, string> = Err("failed");

// Union types
let flexible: string | number = "hello";

// Generics
fn identity<T>(x: T): T { x }
fn first<T extends Comparable>(items: T[]): T { items[0] }
```

## Structs

```atlas
struct Point {
  x: number;
  y: number;
}

struct User {
  name: string;
  age: number;
  email: string;
}

let p = Point { x: 1.0, y: 2.0 };
console.log(p.x);  // 1

impl Point {
  fn distance(self, other: Point): number {
    Math.sqrt(Math.pow(self.x - other.x, 2.0) + Math.pow(self.y - other.y, 2.0))
  }
}
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
  Circle(number),          // tuple variant
  Rectangle(number, number),
  Named { width: number, height: number },  // struct variant
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

// Match as expression
let label = match shape {
  Shape::Circle(r) => `circle r=${r}`,
  Shape::Rectangle(w, h) => `rect ${w}x${h}`,
  Shape::Named { width, height } => `named ${width}x${height}`,
};

// Match on Result
match result {
  Ok(value) => console.log(`got: ${value}`),
  Err(e) => console.log(`error: ${e}`),
}

// Guard clauses
match n {
  x if x < 0 => "negative",
  0 => "zero",
  x if x > 100 => "large",
  _ => "normal",
}
```

## Control Flow

```atlas
// if/else (no parens)
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

// Error propagation
fn read_file(path: string): Result<string, string> {
  let content = fs.readFile(path)?;  // ? propagates Err
  Ok(content)
}
```

## Traits

```atlas
trait Animal {
  fn speak(self): string;
  fn name(self): string;

  fn describe(self): string {  // default implementation
    `${self.name()} says ${self.speak()}`
  }
}

trait Domestic extends Animal {  // trait inheritance
  fn owner(self): string;
}

impl Animal for Dog {
  fn speak(self): string { "woof" }
  fn name(self): string { self.name }
}
```

## Modules

```atlas
// math.atlas
export fn square(x: number): number { x * x }
export const PI: number = 3.14159;

// main.atlas
import { square, PI } from "./math";
import * as math from "./math";

console.log(square(4));    // 16
console.log(math.PI);      // 3.14159
```

## Async / Concurrency

```atlas
// async/await
async fn main(): void {
  let data = await fetchData("https://example.com");
  console.log(data);
}

// Spawn concurrent tasks
async fn parallel(): void {
  let handle1 = task.spawn(async fn(): number { heavyCompute() });
  let handle2 = task.spawn(async fn(): number { otherCompute() });
  let results = await task.joinAll([handle1, handle2]);
}

// Channels
let ch = channel.unbounded<string>();
channel.send(ch, "hello");
let msg = await channel.receive(ch);

// Timeout
let result = await task.timeout(fetchData(url), 5000);
```

## Collections

```atlas
// Map
let m = new Map<string, number>();
let m2 = m.set("key", 42);    // CoW — must capture return
let val = m2.get("key");       // Option<number>

// Set
let s = new Set<string>();
let s2 = s.add("hello");

// Queue (FIFO)
let q = new Queue<number>();
let q2 = q.enqueue(1);
let [front, q3] = q2.dequeue();  // returns [Option<T>, Queue<T>]

// Stack (LIFO)
let stack = new Stack<number>();
let stack2 = stack.push(1);
let [top, stack3] = stack2.pop();  // returns [Option<T>, Stack<T>]
```

## Error Handling

```atlas
// Result chaining
fn process(input: string): Result<number, string> {
  let parsed = parseInt(input).ok_or("not a number")?;
  let validated = if parsed > 0 { Ok(parsed) } else { Err("must be positive") }?;
  Ok(validated * 2)
}

// Option chaining
let value = maybeString
  .map(fn(s: string): string { s.toUpperCase() })
  .unwrapOr("default");
```

## Stdlib Namespaces

```atlas
import { fs } from "atlas:fs";
import { http } from "atlas:http";
import { path } from "atlas:path";
import { Json } from "atlas:json";
import { Regex } from "atlas:regex";
import { DateTime } from "atlas:datetime";
import { Encoding } from "atlas:encoding";
import { crypto } from "atlas:crypto";
import { sqlite } from "atlas:sqlite";
```
