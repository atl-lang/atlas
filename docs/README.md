# Atlas

**"TypeScript's module system and type annotations wrapped around Rust's runtime model."**

Atlas is a systems-level language built for humans and AI agents. It uses TypeScript's surface syntax — so AI code generation works cold, with zero Atlas-specific training — and Rust's runtime model underneath: copy-on-write value semantics, `Result`/`Option` error handling, and explicit ownership at system boundaries.

- File extension: `.atl`
- Single numeric type: `number` (no int/float split)
- TypeScript-style arrays: `number[]`
- Rust-style error handling: `Result<T, E>`, `Option<T>`, `?` operator
- Copy-on-write collections — transparent, no annotation required

---

## Quick Start

### Hello World

```atlas
fn main(): void {
    console.log("Hello, Atlas!");
}
```

```bash
atlas run main.atl
```

### Error Handling

```atlas
fn divide(a: number, b: number): Result<number, string> {
    if b == 0 {
        return Err("division by zero");
    }
    return Ok(a / b);
}

fn main(): void {
    match divide(10, 2) {
        Ok(v) => console.log(v.toString()),
        Err(e) => console.log("Error: " + e),
    }
}
```

### Structs and Modules

```atlas
// geometry.atl
export struct Point { x: number, y: number }

export impl Point {
    fn distance(borrow self): number {
        return Math.sqrt(self.x * self.x + self.y * self.y);
    }
}

// main.atl
import { Point } from "./geometry";

fn main(): void {
    let p = Point { x: 3, y: 4 };
    console.log(p.distance().toString());  // 5
}
```

---

## Installing

Build from source:

```bash
cargo build --release -p atlas-cli
# Binary: target/release/atlas
```

Add `target/release` to your `PATH`.

---

## Core Commands

| Command | Alias | Description |
|---------|-------|-------------|
| `atlas run <file>` | `r` | Compile and run |
| `atlas run <file> --watch` | | Re-run on file changes |
| `atlas build` | `b` | Build project from `atlas.toml` |
| `atlas test` | `t` | Run all `*.test.atl` files |
| `atlas fmt <files>` | `f` | Format source files |
| `atlas repl` | | Interactive REPL |
| `atlas new <name>` | `n` | Create a new project |
| `atlas debug <file>` | `d` | Interactive debugger |
| `atlas lsp` | | Language Server (stdio) |

Full reference: [cli.md](cli.md)

---

## Language at a Glance

```atlas
// Variables
let x = 42;
let mut count = 0;
const MAX = 1000;

// Functions — borrow is the implicit default for parameters
fn greet(name: string): string {
    return "Hello, " + name + "!";
}

// Generics with trait bounds (D-039)
fn print_it<T extends Printable>(item: T): string {
    return item.to_str();
}

// Copy-on-write — transparent, no annotation
let a = [1, 2, 3];
let b = a;       // O(1) clone
b.push(4);       // a is still [1, 2, 3]

// Ownership — explicit only at system boundaries
fn consume(own data: Buffer): void { }   // caller loses ownership
fn cache(share conn: Connection): void { }  // shared reference
```

---

## Project Structure

```
my-project/
  atlas.toml        # project manifest
  src/
    main.atl        # entry point
  tests/
    math.test.atl   # discovered automatically by atlas test
```

```toml
# atlas.toml
[package]
name = "my-project"
version = "0.1.0"

[[bin]]
name = "my-project"
path = "src/main.atl"
```

---

## Documentation

| Document | Contents |
|----------|----------|
| [cli.md](cli.md) | Every CLI command and flag |
| [testing.md](testing.md) | Test framework, assertions, running tests |
| [language/syntax-quickref.md](language/syntax-quickref.md) | Full syntax reference |
| [language/ownership.md](language/ownership.md) | CoW, ownership annotations, move semantics |
| [language/errors.md](language/errors.md) | Result/Option, `?` operator, propagation |
| [language/generics.md](language/generics.md) | Generic functions, trait bounds |
| [language/visibility.md](language/visibility.md) | pub/private/internal, module exports |
| [language/build.md](language/build.md) | `atlas.toml`, build profiles, distribution |
| [AI-DESIGN-PRINCIPLES.md](AI-DESIGN-PRINCIPLES.md) | Design philosophy — read before any syntax decision |
