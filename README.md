<div align="center">

# Atlas

### The AI-First Systems Language

**TypeScript's surface. Rust's runtime. Built by AI, for AI — and humans.**

[![CI](https://github.com/atl-lang/atlas/actions/workflows/ci.yml/badge.svg)](https://github.com/atl-lang/atlas/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Status](https://img.shields.io/badge/Status-v0.3%20Active-blue.svg)](docs/README.md)

</div>

---

## Table of Contents

- [Why Atlas?](#why-atlas)
- [Quick Start](#quick-start)
- [Language Tour](#language-tour)
  - [Variables](#variables)
  - [Functions](#functions)
  - [Types](#types)
  - [Structs](#structs)
  - [Enums](#enums)
  - [Traits](#traits)
  - [Generics](#generics)
  - [Pattern Matching](#pattern-matching)
  - [Error Handling](#error-handling)
  - [Async / Await](#async--await)
  - [Ownership](#ownership)
  - [Modules](#modules)
- [Standard Library](#standard-library)
- [Package System](#package-system)
- [Web Development](#web-development)
- [CLI Reference](#cli-reference)
- [Project Configuration](#project-configuration)
- [Testing](#testing)
- [Building & Distribution](#building--distribution)
- [Design Philosophy](#design-philosophy)

---

## Why Atlas?

Other languages were designed before AI existed. Atlas was designed **for** AI code generation from day one — and that changes everything.

> **"TypeScript's module system and type annotations wrapped around Rust's runtime model."**

**The rule:** If TypeScript has an answer, Atlas uses it exactly. If TypeScript has no answer (systems-level concerns), Atlas designs its own minimal syntax. Rust/Go surface syntax is never copied.

**The result:** AI agents generate correct Atlas code with zero special training, because it looks like TypeScript. And it runs with Rust-grade safety because the runtime model is borrowed directly from Rust.

### What Atlas delivers

| Concern | How Atlas handles it |
|---------|---------------------|
| AI code generation | TypeScript-familiar surface — zero friction |
| Memory safety | Copy-on-write value semantics, no GC |
| Systems-level work | Ownership annotations, async, channels |
| Error handling | `Result<T,E>` and `Option<T>` — no exceptions |
| Packages | Git-native — no central registry |
| Concurrency | Tokio-based async, task spawning, channels |
| Execution | Single path: compiler + VM only |

---

## Quick Start

```bash
# Build from source
cargo build --release -p atlas-cli
export PATH="$PATH:$HOME/dev/projects/atlas/target/release"

# Create a new project
atlas new my-project
cd my-project

# Run
atlas run src/main.atl
```

```javascript
// src/main.atl
fn main(): void {
    console.log("Hello, Atlas!");
}
```

---

## Language Tour

### Variables

```javascript
let x = 10;             // immutable
let mut y = 20;         // mutable
const PI = 3.14159;     // compile-time constant, inlined everywhere
```

### Functions

Named functions require an explicit return type. Anonymous functions (closures) do not.

```javascript
fn add(x: number, y: number): number {
    return x + y;
}

// Default parameters
fn greet(name: string = "World"): void {
    console.log("Hello, " + name);
}

// Rest parameters
fn sum(...values: number[]): number {
    return values.reduce(fn(acc, v) { return acc + v; }, 0);
}

// Tail expression — no return keyword needed
fn double(x: number): number {
    x * 2
}

// Closures
let multiply = fn(x: number, y: number): number { return x * y; };

// Async functions
async fn fetch(url: string): Future<string> {
    let resp = await http.get(url);
    return resp.body;
}

// defer — cleanup runs on scope exit (LIFO)
fn open_file(path: string): void {
    let f = file.open(path);
    defer file.close(f);
    // ... use f
}
```

### Types

```javascript
// Primitives
let n: number = 42;       // unified — no int/float split
let s: string = "hello";
let b: bool = true;
let nothing: null = null;

// Arrays — TypeScript postfix syntax
let nums: number[] = [1, 2, 3];
let nested: string[][] = [["a", "b"]];

// Tuples — fixed-arity, heterogeneous
let pair: (number, string) = (1, "one");
let single: (number,) = (42,);  // trailing comma for single-element

// Maps and Sets
let map = new Map<string, number>();
let set = new Set<string>();

// Option and Result
let maybe: Option<number> = Some(42);
let none: Option<number> = None;
let ok: Result<number, string> = Ok(100);
let err: Result<number, string> = Err("failed");

// Union types
let id: number | string = "abc123";

// Type aliases
type UserId = string;
type Callback = (number) => void;

// Structural types
type Point2D = { x: number, y: number };
```

### Structs

```javascript
struct User {
    id: number,
    name: string,
    email: string,
}

// Instantiate — all fields required
let user = User { id: 1, name: "Alice", email: "alice@example.com" };

// Methods via impl
impl User {
    fn display(self): string {
        return `${self.name} <${self.email}>`;
    }

    static fn guest(): User {
        return User { id: 0, name: "Guest", email: "" };
    }
}

console.log(user.display());        // "Alice <alice@example.com>"
console.log(User.guest().name);     // "Guest"

// Generic structs
struct Pair<T, U> {
    first: T,
    second: U,
}

let p = Pair<number, string> { first: 1, second: "one" };
```

### Enums

```javascript
// Unit variants
enum Direction {
    North,
    South,
    East,
    West,
}

// Tuple variants — with data
enum Shape {
    Circle(number),            // radius
    Rectangle(number, number), // width, height
}

// Struct variants — named fields
enum Message {
    Quit,
    Move { x: number, y: number },
    Text(string),
}

let shape = Shape::Circle(5.0);
let msg = Message::Move { x: 10, y: 20 };
```

### Traits

```javascript
trait Animal {
    fn name(self): string;
    fn sound(self): string;

    // Default implementation
    fn describe(self): string {
        return self.name() + " says " + self.sound();
    }
}

struct Dog { name: string }

impl Animal for Dog {
    fn name(self): string { return self.name; }
    fn sound(self): string { return "woof"; }
}

// Trait inheritance — TypeScript comma style
trait Pet extends Animal, Trainable {
    fn owner(self): string;
}
```

### Generics

```javascript
// Generic function with trait bound
fn largest<T extends Comparable>(list: T[]): T {
    let mut max = list[0];
    for item in list {
        if item > max { max = item; }
    }
    return max;
}

// Multiple bounds — & style (TypeScript)
fn process<T extends Readable & Writable>(item: T): void {
    // ...
}

// Generic struct with impl
struct Stack<T> {
    items: T[],
}

impl<T> Stack<T> {
    fn push(self, item: T): Stack<T> {
        return Stack { items: self.items.push(item) };
    }

    fn pop(self): Option<T> {
        if self.items.length() == 0 { return None; }
        return Some(self.items[self.items.length() - 1]);
    }
}
```

### Pattern Matching

Enum variants are matched by bare name — the typechecker resolves them from the scrutinee type. The qualified `EnumName::Variant` form also works but is not required.

```javascript
// Match on enum — bare variant names
match shape {
    Circle(r) => console.log(`Circle with radius ${r}`),
    Rectangle(w, h) => console.log(`${w}x${h} rectangle`),
}

// Guards
match score {
    n if n >= 90 => "A",
    n if n >= 80 => "B",
    n if n >= 70 => "C",
    _ => "F",
}

// OR patterns
match direction {
    North | South => "vertical",
    East | West => "horizontal",
}

// Struct patterns
match user {
    User { name: "admin", .. } => console.log("admin user"),
    User { id, name, .. } => console.log(`user ${id}: ${name}`),
}

// Option / Result
match result {
    Ok(value) => console.log(`Got: ${value}`),
    Err(e) => console.log(`Error: ${e}`),
}

match maybe {
    Some(v) => v * 2,
    None => 0,
}
```

### Error Handling

```javascript
fn divide(a: number, b: number): Result<number, string> {
    if b == 0 {
        return Err("division by zero");
    }
    return Ok(a / b);
}

// ? operator — propagates error up the call stack
fn compute(x: number): Result<number, string> {
    let half = divide(x, 2)?;
    let quarter = divide(half, 2)?;
    return Ok(quarter);
}

// Chaining
let result = divide(10, 2)
    .map(fn(v) { return v * 100; })
    .unwrapOr(0);

// Option methods
let name: Option<string> = Some("Atlas");
let upper = name.map(fn(s) { return s.toUpperCase(); });
let value = name.unwrapOr("unknown");
```

### Async / Await

```javascript
// Basic async function
async fn fetch_user(id: number): Future<string> {
    let url = `https://api.example.com/users/${id}`;
    let response = await http.get(url);
    return response.body;
}

// Parallel execution
async fn load_all(): Future<void> {
    let [users, posts] = await futureAll([
        fetch_users(),
        fetch_posts(),
    ]);
    console.log(`Loaded ${users.length()} users, ${posts.length()} posts`);
}

// Task spawning
let handle = task.spawn(fn() {
    return expensive_computation();
});
let result = await task.join(handle);

// Channels — safe message passing between tasks
let ch = channelUnbounded<string>();
task.spawn(fn() {
    channelSend(ch, "hello from task");
});
let msg = await channelReceive(ch);

// Timers
await sleep(1000);                   // pause 1 second
let t = timer(5000);                 // fire once after 5s
let iv = interval(1000);             // fire every 1s

// Racing futures
let first = await futureRace([slowOp(), fastOp()]);
```

### Ownership

Atlas uses Rust's ownership model, but it's mostly **invisible** for everyday code. The default is `borrow` — annotate only when you need something different.

```javascript
// borrow — default, caller keeps ownership (no annotation needed)
fn read_name(user: User): string {
    return user.name;
}

// own — move semantics, caller loses binding after call
fn consume(own data: string): void {
    // data is moved in
}

// share — Arc, concurrent read access
fn share_config(share cfg: Config): void {
    task.spawn(fn() { console.log(cfg.host); });
}

// Copy-on-write arrays — value semantics, zero annotation needed
let a = [1, 2, 3];
let b = a;            // O(1) — shared backing store
b = b.push(4);        // copy-on-write triggers here
console.log(a);       // [1, 2, 3] — unchanged
console.log(b);       // [1, 2, 3, 4]
```

### Modules

```javascript
// Named imports
import { add, subtract } from "./math";
import { User, createUser } from "./models/user";

// Namespace import
import * as utils from "./utils";
utils.format("hello");

// Exports
export fn add(a: number, b: number): number { return a + b; }
export struct Config { host: string, port: number }
export const VERSION = "0.3.0";
export type Handler = (Request) => Response;

// Package import (after atlas install)
import { new_router, chain, logger } from "web";
```

---

## Standard Library

### Console & I/O

```javascript
console.log("hello");
console.error("oops");
console.warn("warning");
console.debug("debug info");

let line = io.readLine();
let prompted = io.readLinePrompt("Enter name: ");
```

### File System

```javascript
let content = file.read("./data.txt");    // Option<string>
file.write("./out.txt", "content");
file.append("./log.txt", "new line\n");
let exists = file.exists("./data.txt");
file.remove("./tmp.txt");
file.rename("./old.txt", "./new.txt");
file.copy("./src.txt", "./dst.txt");

// Async variants
let content = await file.readAsync("./data.txt");
await file.writeAsync("./out.txt", "content");
```

### Math

```javascript
Math.sqrt(16)       // Some(4.0)
Math.abs(-5)        // 5
Math.floor(3.7)     // 3
Math.ceil(3.2)      // 4
Math.round(3.5)     // 4
Math.pow(2, 10)     // 1024
Math.log(Math.E)    // Some(1.0)
Math.sin(0)         // 0.0
Math.cos(0)         // 1.0
Math.random()       // number in [0, 1)
Math.min(1, 2, 3)   // 1
Math.max(1, 2, 3)   // 3
```

### String Methods

```javascript
let s = "Hello, Atlas!";
s.length()                     // 13
s.toUpperCase()                // "HELLO, ATLAS!"
s.toLowerCase()                // "hello, atlas!"
s.trim()                       // strips whitespace
s.startsWith("Hello")          // true
s.endsWith("!")                // true
s.includes("Atlas")            // true
s.indexOf("Atlas")             // Some(7)
s.slice(7, 12)                 // "Atlas"
s.substring(7, 12)             // "Atlas"
s.split(", ")                  // ["Hello", "Atlas!"]
s.replace("Atlas", "World")    // "Hello, World!"
s.repeat(2)
s.padStart(20, "*")
s.padEnd(20, "-")
s.charAt(0)                    // "H"
```

### Array Methods

All array operations are **copy-on-write** — originals are never mutated.

```javascript
let arr = [1, 2, 3];
arr.length()                              // 3
arr.push(4)                               // [1, 2, 3, 4]
arr.pop()                                 // (3, [1, 2])
arr.shift()                               // (1, [2, 3])
arr.unshift(0)                            // [0, 1, 2, 3]
arr.reverse()                             // [3, 2, 1]
arr.slice(1, 3)                           // [2, 3]
arr.concat([4, 5])                        // [1, 2, 3, 4, 5]
arr.indexOf(2)                            // Some(1)
arr.includes(2)                           // true
arr.join(", ")                            // "1, 2, 3"
arr.map(fn(x) { return x * 2; })         // [2, 4, 6]
arr.filter(fn(x) { return x > 1; })      // [2, 3]
arr.reduce(fn(acc, x) { return acc + x; }, 0)  // 6
arr.find(fn(x) { return x > 1; })        // Some(2)
arr.some(fn(x) { return x > 2; })        // true
arr.every(fn(x) { return x > 0; })       // true
arr.sort(fn(a, b) { return a - b; })
arr.flat()
arr.flatMap(fn(x) { return [x, x * 2]; })
```

### Map & Set

```javascript
// Map — CoW, set() returns a new Map
let map = new Map<string, number>();
map = map.set("a", 1);
map = map.set("b", 2);
map.get("a")            // Some(1)
map.has("b")            // true
map = map.delete("a");
map.size                // 1
map.keys()              // string[]
map.values()            // number[]
map.entries()           // (string, number)[]

// Set
let set = new Set<string>();
set = set.add("x");
set.has("x")            // true
set = set.delete("x");
set.size                // 0
```

### JSON

```javascript
let obj = Json.parse(`{"name":"Atlas","version":3}`);
let str = Json.stringify(obj);
let pretty = Json.stringify(obj, 2);   // indented
let keys = Json.keys(obj);             // string[]
```

### HTTP Client

```javascript
// Simple GET
let resp = httpSend(httpRequestGet("https://api.example.com/data"));
match resp {
    Ok(r) => console.log(httpBody(r)),
    Err(e) => console.log("Error: " + e),
}

// POST with headers
let req = httpRequest("POST", "https://api.example.com/users");
req = httpSetHeader(req, "Content-Type", "application/json");
req = httpSetBody(req, `{"name":"Alice"}`);
let result = httpSend(req);

// Async
let resp = await httpGetAsync("https://api.example.com/data");
```

### Process

```javascript
let out = process.exec("ls", ["-la"]);
console.log(out.stdout);
console.log(out.exitCode);

let result = process.shell("echo hello && echo world");
let cwd = process.cwd();
process.exit(0);
```

### Crypto

```javascript
let hash = Crypto.sha256("hello world");
let mac = Crypto.hmac("secret", "message");
let h512 = Crypto.sha512("data");
```

### DateTime

```javascript
let now = DateTime.now();
console.log(now.year, now.month, now.day);
console.log(now.format("YYYY-MM-DD"));
let tomorrow = now.addDays(1);
let ts = now.timestamp();
let parsed = DateTime.parse("2026-01-01", "YYYY-MM-DD");
```

### SQLite

```javascript
let db = sqlite.open("./data.db");
sqlite.execute(db, "CREATE TABLE IF NOT EXISTS users (id INTEGER, name TEXT)");
sqlite.execute(db, "INSERT INTO users VALUES (1, 'Alice')");
let rows = sqlite.query(db, "SELECT * FROM users");
sqlite.close(db);
```

---

## Package System

Atlas uses a **git-native package system** — no central registry. Git tags are versions.

### Declare dependencies in `atlas.toml`

```toml
[dependencies]
web = { git = "https://github.com/atl-pkg/web", tag = "v0.1.0" }
```

### Commands

```bash
atlas install          # fetch all deps → ~/atlas/pkg/, write atlas.lock
atlas update           # bump to latest git tags, update lockfile
atlas add web          # add a dependency interactively
atlas remove web       # remove a dependency
atlas publish          # validate package, create annotated git tag
```

### Lockfile

`atlas.lock` pins each dependency to an exact commit SHA + SHA-256 checksum — always reproducible.

### Using a package

```javascript
import { new_router, chain, logger } from "web";
```

### Official packages

| Package | Repo | Description |
|---------|------|-------------|
| `web` | `atl-pkg/web` | Router, middleware, templates, HTMX |

---

## Web Development

The `atl-pkg/web` package provides routing, middleware, templating, and HTMX integration.

```toml
[dependencies]
web = { git = "https://github.com/atl-pkg/web", tag = "v0.1.0" }
```

### Routing

```javascript
import { new_router } from "web";

const router = new_router()
    .get("/", fn(req, res) {
        return res.html("<h1>Hello from Atlas</h1>");
    })
    .get("/users/:id", fn(req, res) {
        let id = req.params.get("id").unwrapOr("0");
        return res.json(`{"id": ${id}}`);
    })
    .post("/users", fn(req, res) {
        return res.status(201).json(`{"created": true}`);
    });

http.serve(3000, router.match_route);
```

### Middleware

```javascript
import { chain, logger, cors, json_body, rate_limit, basic_auth } from "web";

const app = chain([
    logger(),
    cors("*"),
    json_body(),
    rate_limit(100),
    basic_auth("user", "pass"),
], router.match_route);

http.serve(3000, app);
```

### Templating

```javascript
import { render, render_file } from "web";

let ctx = new Map<string, any>();
ctx = ctx.set("title", "Atlas");
ctx = ctx.set("items", ["one", "two", "three"]);

let html = render(`
<h1>{{ title }}</h1>
<ul>
  {% for item in items %}
  <li>{{ item }}</li>
  {% end %}
</ul>
`, ctx);

// From file
let page = render_file("views/index.html", ctx);
```

| Syntax | Description |
|--------|-------------|
| `{{ expr }}` | Variable interpolation |
| `{% for x in list %}...{% end %}` | Loop |
| `{% if cond %}...{% else %}...{% end %}` | Conditional |
| `{% include "partial" %}` | Include partial |

### HTMX

```javascript
import { htmx } from "web";

router.get("/counter", fn(req, res) {
    if htmx.is_htmx(req) {
        return htmx.partial(res, "<span>42</span>");
    }
    return res.html(render_file("views/counter.html", ctx));
});
```

### Static Files

```javascript
import { static_files } from "web";

const app = chain([static_files("./public"), logger()], router.match_route);
```

---

## CLI Reference

| Command | Description |
|---------|-------------|
| `atlas run <file>` | Compile and run a file (`--watch` to re-run on save) |
| `atlas build` | Build project from `atlas.toml` (`--release` for production) |
| `atlas test [pattern]` | Run `*.test.atl` files (`--sequential`, `--verbose`) |
| `atlas fmt [files]` | Format source files (`--check` for CI) |
| `atlas check <file>` | Type-check without running |
| `atlas repl` | Interactive REPL (`--tui` for TUI mode) |
| `atlas debug <file>` | Interactive debugger |
| `atlas new <name>` | Create a new project from template |
| `atlas init` | Initialize in current directory |
| `atlas install` | Fetch dependencies, write lockfile |
| `atlas update` | Update all deps to latest versions |
| `atlas add <pkg>` | Add a dependency |
| `atlas remove <pkg>` | Remove a dependency |
| `atlas publish` | Publish package (creates annotated git tag) |
| `atlas lsp` | Start Language Server on stdio |
| `atlas explain <code>` | Explain error/warning code (`AT`/`AW` prefix) |
| `atlas profile <file>` | Profile VM execution, output flamegraph data |
| `atlas ast <file>` | Dump AST as JSON |
| `atlas typecheck <file>` | Dump type info as JSON |
| `atlas completions <shell>` | Generate shell completions |

**Global flags:** `--json` / `ATLAS_JSON=1` (machine-readable output), `--no-color` / `NO_COLOR`

---

## Project Configuration

```toml
[package]
name = "my-project"
version = "1.0.0"
edition = "2026"
description = "An Atlas project"
authors = ["Your Name <you@example.com>"]
license = "MIT"
repository = "https://github.com/you/my-project"

[build]
output = "target"
source = "src"
entry = "src/main.atl"

[compiler]
optimize = false    # true for production
debug = true

[formatting]
indent = 4
max_line_length = 100
use_tabs = false

[security]
mode = "standard"   # "none" | "standard" | "strict"

[security.filesystem]
read = ["./data"]
write = ["./output"]
deny = ["/etc", "/usr"]

[security.network]
allow = ["*.example.com"]

[dependencies]
web = { git = "https://github.com/atl-pkg/web", tag = "v0.1.0" }
```

---

## Testing

Tests live in `*.test.atl` files, auto-discovered by `atlas test`.

```javascript
// tests/math.test.atl
import { test, assert, assertEqual } from "test";

test("divide — happy path", fn() {
    let result = divide(10, 2);
    assert(result.isOk());
    assertEqual(result.unwrap(), 5);
});

test("divide — by zero returns Err", fn() {
    let result = divide(10, 0);
    assert(result.isErr());
    assertEqual(result.unwrapErr(), "division by zero");
});

test("array CoW semantics", fn() {
    let a = [1, 2, 3];
    let b = a;
    b = b.push(4);
    assertEqual(a.length(), 3);   // a unchanged
    assertEqual(b.length(), 4);
});
```

```bash
atlas test               # all tests, parallel
atlas test divide        # filter by name
atlas test --sequential  # one at a time
atlas test --verbose     # show each test name
```

---

## Building & Distribution

```bash
# Development
atlas run src/main.atl

# Production — self-contained binary, VM embedded, no runtime dep
atlas build --release
# → target/release/<name>
```

---

## Design Philosophy

### The AI-First Principle

> *If it's hard for AI to generate, the language is wrong — not the AI.*

Every syntax decision runs through this filter:

1. **TypeScript has it?** → Use TypeScript's exact form
2. **TypeScript has no answer?** → Design Atlas-native, minimal tokens
3. **Only Rust/Go has it?** → Never copy surface syntax (exceptions: `match`, `impl`, `trait` — AI already knows them)

### Key Design Decisions

| Decision | Why |
|----------|-----|
| `number` — unified numeric type | AI never picks the wrong int/float variant |
| `T[]` arrays — TypeScript postfix | Familiar to every model trained on TypeScript |
| `let` / `let mut` | Immutable by default, same as Rust |
| `fn` keyword | Fewer tokens than `function` |
| `match` expression | AI knows it from Rust and FP languages |
| `impl Foo { }` | AI knows it from Rust |
| `trait extends A, B` | TypeScript comma style — not Rust `+` |
| `<T extends A & B>` | TypeScript `&` for generic bounds |
| `Result<T,E>` / `Option<T>` | Rust names — AI knows both |
| CoW memory — invisible | No annotation needed for everyday code |
| `borrow` is default | Zero annotation for the common case |
| Compiler + VM only | Single execution path — no interpreter complexity |

### The Runtime Model

- **No garbage collector** — deterministic allocation, value semantics by default
- **Copy-on-write** — arrays, maps, sets share backing store until mutation; transparent to the programmer
- **Ownership at boundaries** — annotate `own` or `share` only when you need move/Arc semantics
- **Async via Tokio** — cooperative tasks (~100 bytes overhead each), work-stealing thread pool
- **Bytecode compiler → VM** — single execution path, no interpreter fallback

---

<div align="center">

Built by AI. For AI. And humans who want both.

</div>
