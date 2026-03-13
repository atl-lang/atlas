# Common Atlas Patterns

Correct, idiomatic ways to do common things. Copy these.

## Error Handling

### Propagate with `?`
```atlas
fn read_config(path: string): Result<Config, string> {
  let text = fs.readFile(path)?;
  let json = Json.parse(text)?;
  let config = build_config(json)?;
  Ok(config)
}
```

### Match on Result
```atlas
match fs.readFile("config.json") {
  Ok(content) => process(content),
  Err(e) => {
    console.error(`Failed: ${e}`);
  },
}
```

### Default on None
```atlas
let name = user.get("name").unwrapOr("Anonymous");
let count = cache.get(key).unwrapOrElse(fn(): number { computeDefault() });
```

### Chain Option transformations
```atlas
let upper = maybeStr
  .map(fn(s: string): string { s.toUpperCase() })
  .filter(fn(s: string): bool { s.length > 0 })
  .unwrapOr("DEFAULT");
```

## Collections

### Build a Map from data
```atlas
let scores = new Map<string, number>();
for player in players {
  let scores = scores.set(player.name, player.score);
}
```

### Iterate a Map
```atlas
// forEach with callback
myMap.forEach(fn(key: string, value: number): void {
  console.log(`${key}: ${value}`);
});

// Get all keys/values
let keys = myMap.keys();
let values = myMap.values();
let entries = myMap.entries();

for entry in entries {
  let (key, value) = entry;
  console.log(`${key} = ${value}`);
}
```

### Set operations
```atlas
let a = new Set<string>().add("x").add("y");
let b = new Set<string>().add("y").add("z");

// Set algebra (returns new Set)
let union = a.union(b);              // {x, y, z}
let intersection = a.intersection(b); // {y}
let diff = a.difference(b);          // {x}
```

### Queue as work queue
```atlas
let queue = new Queue<Task>();
let queue = queue.enqueue(task1).enqueue(task2);

while !queue.isEmpty() {
  let [maybe_task, queue] = queue.dequeue();
  match maybe_task {
    Some(task) => process(task),
    None => break,
  }
}
```

## Async Patterns

### Sequential async
```atlas
async fn pipeline(url: string): Result<string, string> {
  let response = await http.get(url);
  let body = response.body();
  let parsed = Json.parse(body)?;
  Ok(formatResult(parsed))
}
```

### Parallel async with joinAll
```atlas
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
```

### Timeout pattern
```atlas
async fn withTimeout<T>(work: Future<T>, ms: number): Result<T, string> {
  match await task.timeout(work, ms) {
    Ok(result) => Ok(result),
    Err(_) => Err(`timed out after ${ms}ms`),
  }
}
```

### Channel producer/consumer
```atlas
async fn producer(ch: Channel<number>): void {
  for i in 0..100 {
    await channel.send(ch, i);
  }
}

async fn consumer(ch: Channel<number>): void {
  while true {
    match await channel.receive(ch) {
      Some(val) => process(val),
      None => break,
    }
  }
}
```

## Structs and Impl

### Builder pattern
```atlas
struct HttpRequest {
  url: string;
  method: string;
  headers: Map<string, string>;
  timeout: number;
}

impl HttpRequest {
  fn new(url: string): HttpRequest {
    HttpRequest {
      url,
      method: "GET",
      headers: new Map<string, string>(),
      timeout: 30000,
    }
  }

  fn withMethod(self, method: string): HttpRequest {
    HttpRequest { method, ..self }  // struct update syntax
  }

  fn withHeader(self, key: string, value: string): HttpRequest {
    HttpRequest { headers: self.headers.set(key, value), ..self }
  }
}

let req = HttpRequest.new("https://example.com")
  .withMethod("POST")
  .withHeader("Content-Type", "application/json");
```

### Trait-based polymorphism
```atlas
trait Serialize {
  fn toJson(self): string;
}

fn saveAll<T extends Serialize>(items: T[]): void {
  for item in items {
    let json = item.toJson();
    fs.writeFile(`item.json`, json);
  }
}
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

### Nested pattern destructuring
```atlas
match result {
  Ok(Some(value)) => console.log(`got: ${value}`),
  Ok(None) => console.log("got nothing"),
  Err(e) => console.error(`failed: ${e}`),
}
```

### Guard clauses
```atlas
match event {
  Event::Key(key) if key == "Enter" => submit(),
  Event::Key(key) if key == "Escape" => cancel(),
  Event::Key(key) => handleKey(key),
  Event::Mouse(x, y) if x > 0 && y > 0 => handleClick(x, y),
  _ => {},
}
```

## File I/O

```atlas
import { fs } from "atlas:fs";

// Read text file
match fs.readFile("input.txt") {
  Ok(content) => process(content),
  Err(e) => console.error(e),
}

// Write file
fs.writeFile("output.txt", content)?;

// Async I/O
async fn loadFile(path: string): Result<string, string> {
  await fs.readFileAsync(path)
}

// Directory listing
let entries = fs.readdir("./src")?;
for entry in entries {
  console.log(entry);
}
```

## HTTP

```atlas
import { http } from "atlas:http";

// GET
let response = await http.get("https://api.example.com/data");
if response.isSuccess() {
  let body = Json.parse(response.body());
}

// POST with JSON
let req = http.request("https://api.example.com/create")
  |> http.setMethod("POST")
  |> http.setHeader("Content-Type", "application/json")
  |> http.setBody(Json.stringify(payload));
let response = await http.sendAsync(req);
```

## JSON

```atlas
import { Json } from "atlas:json";

// Parse
let value = Json.parse(jsonString)?;

// Access fields
let name = Json.getString(value, "name")?;
let age = Json.getNumber(value, "age")?;
let tags = Json.getArray(value, "tags")?;

// Stringify
let json = Json.stringify(myObject);
let pretty = Json.prettify(myObject);
```

## String Operations

```atlas
let s = "Hello, World!";
s.toUpperCase()           // "HELLO, WORLD!"
s.toLowerCase()           // "hello, world!"
s.trim()                  // remove whitespace
s.split(", ")             // ["Hello", "World!"]
s.includes("World")       // true
s.startsWith("Hello")     // true
s.replace("World", "Atlas")  // "Hello, Atlas!"
s.substring(0, 5)         // "Hello"
s.length                  // 13

// Template literals
let name = "Atlas";
let greeting = `Hello, ${name}!`;
```
