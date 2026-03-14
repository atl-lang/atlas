# Common Atlas Patterns

Correct, idiomatic ways to do common things. Copy these.

## File I/O

### Reading a file

```atlas
// file.readText returns Result<string, string>
match file.readText("config.json") {
    Ok(content) => process(content),
    Err(e) => console.log(`failed to read: ${e}`),
}

// With ? propagation
fn loadConfig(path: string): Result<string, string> {
    let text = file.readText(path)?;
    Ok(text)
}
```

### Writing a file

```atlas
file.writeText("output.txt", content);

// With error handling
match file.writeText("output.txt", content) {
    Ok(_) => console.log("saved"),
    Err(e) => console.log(`write failed: ${e}`),
}
```

### Checking if a file exists

```atlas
if file.exists("config.json") {
    let text = file.readText("config.json").unwrapOr("{}");
    console.log(text);
}
```

## HashMap

```atlas
let m: HashMap<string, number> = hashMapNew();
m.set("key", 42);
let val = m.get("key");   // Option<number>

match val {
    Some(v) => console.log(`got: ${v}`),
    None => console.log("not found"),
}

m.has("key");    // bool
m.delete("key");
```

## JSON

```atlas
// Stringify — lowercase j
let text = json.stringify(value);
let pretty = json.stringify(value);   // pretty-print variant TBD per docs

// Parse — returns Result
match json.parse(jsonString) {
    Ok(value) => console.log(value),
    Err(e) => console.log(`parse error: ${e}`),
}

// With ? propagation
fn parseResponse(body: string): Result<string, string> {
    let data = json.parse(body)?;
    Ok(data)
}
```

## Math

```atlas
math.sqrt(16.0).unwrap()   // 4  — returns Option<number>
math.abs(-5)               // 5
math.floor(3.7)            // 3
math.ceil(3.2)             // 4
math.round(3.5)            // 4
math.pow(2.0, 10.0)        // 1024
math.min(3, 7)             // 3
math.max(3, 7)             // 7
```

## Option

```atlas
let maybe: Option<number> = Some(42);
let nothing: Option<number> = None;

maybe.isSome()           // true
nothing.isNone()         // true
maybe.unwrapOr(0)        // 42
nothing.unwrapOr(0)      // 0

// Transform
maybe.map(fn(x: number): number { x * 2 })  // Some(84)

// Chain — returns None if maybe is None
maybe.andThen(fn(x: number): Option<number> {
    if x > 0 { Some(x) } else { None }
})
```

## Result

```atlas
let ok: Result<number, string> = Ok(42);
let fail: Result<number, string> = Err("oops");

match ok {
    Ok(v) => console.log(`value: ${v}`),
    Err(e) => console.log(`error: ${e}`),
}

// Unwrap with fallback
ok.unwrapOr(0)           // 42
fail.unwrapOr(0)         // 0

// ? propagation in functions
fn compute(input: string): Result<number, string> {
    let n = parseInt(input).okOr("not a number")?;
    let result = someOtherFn(n)?;
    Ok(result)
}
```

## Structs

```atlas
struct Point {
    x: number;
    y: number;
}

impl Point {
    fn new(x: number, y: number): Point {
        Point { x, y }
    }

    fn dist(borrow self): number {
        math.sqrt(self.x * self.x + self.y * self.y).unwrap()
    }

    fn translate(borrow self, dx: number, dy: number): Point {
        Point { x: self.x + dx, y: self.y + dy }
    }
}

let p = Point.new(3.0, 4.0);
console.log(p.dist());   // 5
```

## String Interpolation

```atlas
let name = "Atlas";
let version = 3;

// Backtick template — use ${expr}
let msg = `Hello from ${name} v${version}`;

// NOT {expr} — curly braces alone do not interpolate
// NOT `Hello from {name}`  -- WRONG
```

## Collections — CoW Pattern

All collection mutations return a new value. You MUST capture it.

```atlas
// Map
let m = new Map<string, number>();
let m = m.set("a", 1);     // rebind (shadowing is OK)
let m = m.set("b", 2);
m.get("a");                 // Some(1)

// Set
let s = new Set<string>();
let s = s.add("hello");
let s = s.add("world");
s.has("hello");             // true

// Array push also returns new array
let arr = [1, 2, 3];
let arr = arr.push(4);      // [1, 2, 3, 4]
```

## Async Patterns

```atlas
// Sequential async
async fn pipeline(url: string): Result<string, string> {
    let response = await http.get(url);
    let body = response.body();
    let parsed = json.parse(body)?;
    Ok(formatResult(parsed))
}

// Parallel with joinAll
async fn fetchAll(urls: string[]): string[] {
    let handles: TaskHandle<string>[] = [];
    for url in urls {
        let handle = task.spawn(async fn(): string {
            let r = await http.get(url);
            r.body()
        });
        handles = handles.push(handle);
    }
    await task.joinAll(handles)
}

// Timeout
async fn withTimeout<T>(work: Future<T>, ms: number): Result<T, string> {
    match await task.timeout(work, ms) {
        Ok(result) => Ok(result),
        Err(_) => Err(`timed out after ${ms}ms`),
    }
}
```

## Error Handling

### Propagate with `?`

```atlas
fn readConfig(path: string): Result<string, string> {
    let text = file.readText(path)?;
    let data = json.parse(text)?;
    Ok(data)
}
```

### Match on Result

```atlas
match file.readText("config.json") {
    Ok(content) => process(content),
    Err(e) => {
        console.log(`failed: ${e}`);
    },
}
```

### Default on None

```atlas
let name = user.get("name").unwrapOr("Anonymous");
```

## Pattern Matching

### Exhaustive enum match

```atlas
enum Status { Active, Inactive, Pending(string) }

fn describe(s: Status): string {
    match s {
        Status::Active => "active",
        Status::Inactive => "inactive",
        Status::Pending(reason) => `pending: ${reason}`,
    }
}
```

### Nested destructuring

```atlas
match result {
    Ok(Some(value)) => console.log(`got: ${value}`),
    Ok(None) => console.log("got nothing"),
    Err(e) => console.log(`failed: ${e}`),
}
```
