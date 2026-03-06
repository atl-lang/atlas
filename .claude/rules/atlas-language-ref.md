---
paths:
  - "**/*.atlas"
  - "**/*.atl"
  - "crates/atlas-runtime/src/parser/**"
  - "crates/atlas-runtime/src/lexer/**"
---

# Atlas Language Quick Reference

Auto-loaded when editing `.atlas`/`.atl` files or parser/lexer code.

```atlas
// Variables
let x = 42;
let mut y = 0;

// Types
let n: number = 42;
let s: string = "hello";
let b: bool = true;
let arr: number[] = [1, 2, 3];

// Structs
struct Point { x: number, y: number }
let p = Point { x: 1, y: 2 };
print(p.x);

// Functions
fn add(a: number, b: number) -> number { a + b }

// Entry point (optional — top-level code also runs)
fn main() {
    print("Hello from main!");
}

// Traits
trait Greetable {
    fn greet(self: Greetable) -> string;
}
impl Greetable for Point {
    fn greet(self: Point) -> string { return "I am a point"; }
}
let greeting = p.greet();

// Traits
trait Greetable {
    fn greet(self: Greetable) -> string;
    fn default_method(self: Greetable) -> string { return "default"; }  // default impl OK
}
impl Greetable for Point {
    fn greet(self: Point) -> string { return "I am a point"; }
    // default_method inherited automatically
}
let greeting = p.greet();          // method dispatch via trait

// Modules
import { foo, bar } from "module"; // named imports
import * as mod from "module";     // namespace import
export fn public_fn() { }          // explicit export

// Stdlib (camelCase, global)
let arr2 = arrayPush(arr, 4);     // NOT push()
let length = len(arr);             // NOT arr.length()
let m: HashMap<string, number> = hashMapNew();
hashMapPut(m, "key", 42);          // NOT m.put()

// Template strings
let msg = `Hello {name}!`;         // {x} not ${x}

// File extension — both .atlas and .atl work
```
