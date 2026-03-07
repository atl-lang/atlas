# Atlas Language Briefing — Haiku Generation Reference

You are generating Atlas programs for a battle test. Atlas is a statically
typed, AI-first language. Every program you generate will be run through the
Atlas runtime and validated. Document any friction you encounter.

## Critical Rules (violations cause parse errors)

1. **Return type is REQUIRED on every named fn:** `fn foo() -> number { ... }`
2. **No-paren conditions:** `if x > 0 { }` NOT `if (x > 0) { }`
3. **Mutable variables:** `let mut x = 0;` not `var x = 0;`
4. **No template strings:** use `"hello " + name` not `` `hello ${name}` ``
5. **Enum construction:** `Status::Ok` or `Status::Err("msg")` — NO struct variants
6. **for-in requires arrays:** `for x in [1,2,3]` — range `0..n` doesn't work in for-in
7. **Match arms use `=>`:** `match x { 1 => "one", _ => "other" }`
8. **Stdlib is global fns** (not method syntax): `arrayPush(arr, x)` not `arr.push(x)`
9. **CoW collections:** always rebind after mutation: `arr = arrayPush(arr, 1);`
10. **Struct names uppercase:** only uppercase identifiers are struct constructors

## Type Syntax

```atlas
let x: number = 42;
let s: string = "hello";
let b: bool = true;
let n: null = null;
let arr: number[] = [1, 2, 3];
let map: HashMap<string, number> = hashMapNew();
```

## Functions

```atlas
fn add(a: number, b: number) -> number {
    return a + b;
}

// Anonymous fn
let double = fn(x: number) -> number { return x * 2; };

// Recursive
fn factorial(n: number) -> number {
    if n <= 1 { return 1; }
    return n * factorial(n - 1);
}
```

## Structs and Enums

```atlas
struct Point { x: number, y: number }
let p = Point { x: 1, y: 2 };
let px: number = p.x;

enum Color { Red, Green, Blue }
enum Result2 { Ok(number), Err(string) }
let c: Color = Color::Red;
let r: Result2 = Result2::Ok(42);
```

## Pattern Matching

```atlas
match value {
    Some(x) => x,
    None => 0,
}

match color {
    Color::Red => "red",
    Color::Green => "green",
    Color::Blue => "blue",
}
```

## Option and Result

```atlas
let opt: Option<number> = Some(42);
let result: Result<number, string> = Ok(99);

fn safe_divide(a: number, b: number) -> Result<number, string> {
    if b == 0 { return Err("division by zero"); }
    return Ok(a / b);
}
```

## Collections

```atlas
// Arrays
let mut arr: number[] = [1, 2, 3];
arr = arrayPush(arr, 4);
let n: number = len(arr);
let mapped: number[] = map(arr, fn(x: number) -> number { return x * 2; });

// HashMap
let mut scores: HashMap<string, number> = hashMapNew();
scores = hashMapPut(scores, "alice", 95);
let score: any = hashMapGet(scores, "alice");
```

## Error Handling

```atlas
fn parse_age(s: string) -> Result<number, string> {
    let n: number = toNumber(s);
    if n < 0 { return Err("negative age"); }
    return Ok(n);
}

// ? operator propagates Err
fn process(s: string) -> Result<number, string> {
    let age = parse_age(s)?;
    return Ok(age * 2);
}
```

## Async / Await

```atlas
async fn fetch_data(id: number) -> string {
    await sleep(0);
    return "data_" + id;
}

let result = await fetch_data(1);

// Parallel
let results = await futureAll([futureResolve(1), futureResolve(2)]);
```

## Traits

```atlas
trait Describable {
    fn describe(self) -> string;
}

struct Dog { name: string }
impl Describable for Dog {
    fn describe(self) -> string {
        return "Dog: " + self.name;
    }
}
```

## Known Limitations (work around these)

- No template strings: use `+` for string concatenation
- No method syntax on stdlib: use `arrayPush`, `hashMapGet` etc.
- No struct-variant enum construction: only unit and tuple variants
- Closures mutating global state may not persist (use return values instead)
- No module imports in this test suite (self-contained files only)

## Output Format

Each program ends with the value to be printed/returned. Use `print(x)` for
string output or just return the value from the last expression for number output.

## Your Task

Write the programs in the domain folder. For EACH program:
1. Write clean, idiomatic Atlas code
2. Note any friction you encounter as a comment `// FRICTION: ...`
3. If you cannot express something naturally in Atlas, note it as `// LIMITATION: ...`
4. Programs should be 20–60 lines — no god files
