# Atlas Syntax Quick Reference

## Variables

```atlas
let x = 10;              // immutable
let mut y = 20;          // mutable
var z = 30;              // mutable (alias for let mut)
```

## Types

```atlas
// Primitives
let n: number = 42;
let s: string = "hello";
let b: bool = true;
let v: void;
let nu: null = null;

// Arrays (D-041: prefix syntax)
let arr: []number = [1, 2, 3];
let nested: [][]string = [["a"], ["b"]];

// Tuples (D-042)
let pair: (number, string) = (1, "one");
let (x, y) = pair;  // destructuring

// Option/Result
let opt: Option<number> = Some(42);
let res: Result<number, string> = Ok(42);

// HashMap/HashSet
let map: HashMap<string, number> = hashMapNew();
let set: HashSet<string> = hashSetNew();
```

## Functions

```atlas
// Named functions require return type (D-023, D-046)
fn add(borrow x: number, borrow y: number): number {
    return x + y;
}

// Void return
fn greet(borrow name: string): void {
    console.log("Hello " + name);
}

// Anonymous functions (closures)
let double = fn(borrow x: number): number { return x * 2; };
```

## Ownership (D-040)

```atlas
fn example(borrow x: number): void { }  // borrow (default if omitted)
fn example(own x: number): void { }      // take ownership
fn example(share x: number): void { }    // shared reference
```

## Control Flow

```atlas
// If/else
if condition {
    // ...
} else if other {
    // ...
} else {
    // ...
}

// Match
match value {
    1 => console.log("one"),
    2 | 3 => console.log("two or three"),
    n if n > 10 => console.log("big"),
    _ => console.log("other"),
}

// Loops (D-007, D-012)
for item in collection {
    // ...
}

while condition {
    // ...
}

// Loop control
break;
continue;
```

## Structs (D-022)

```atlas
struct Point {
    x: number,
    y: number,
}

let p = Point { x: 10, y: 20 };
console.log(p.x);
```

## Enums

```atlas
enum Color {
    Red,
    Green,
    Blue,
    Rgb(number, number, number),
}

let c = Color.Red;
let custom = Color.Rgb(255, 128, 0);

match c {
    Color.Red => console.log("red"),
    Color.Rgb(r, g, b) => console.log("rgb"),
    _ => console.log("other"),
}
```

## Traits (D-026)

```atlas
trait Printable {
    fn print(borrow self): void;
}

impl Printable for Point {
    fn print(borrow self): void {
        console.log("(" + self.x.toString() + ", " + self.y.toString() + ")");
    }
}
```

## Inherent Impl (D-036)

```atlas
impl Point {
    fn new(borrow x: number, borrow y: number): Point {
        return Point { x: x, y: y };
    }

    fn distance(borrow self): number {
        return Math.sqrt(self.x * self.x + self.y * self.y);
    }
}

let p = Point.new(3, 4);
console.log(p.distance());  // 5
```

## Option/Result

```atlas
// Option
let opt: Option<number> = Some(42);
let none: Option<number> = None;

if opt.isSome() {
    console.log(opt.unwrap());
}

let value = opt.unwrapOr(0);

// Result
let ok: Result<number, string> = Ok(42);
let err: Result<number, string> = Err("failed");

match result {
    Ok(v) => console.log(v),
    Err(e) => console.log(e),
}
```

## Modules

```atlas
// math.atl
export fn add(borrow x: number, borrow y: number): number {
    return x + y;
}

// main.atl
import { add } from "./math";
console.log(add(1, 2));
```

## File Extensions (D-047)

- `.atl` - Atlas source files
- `.atlas` - Also valid (legacy)
