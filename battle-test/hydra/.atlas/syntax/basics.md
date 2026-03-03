# Basic Syntax

## Variables

### Immutable by Default (Rust-style)

```atlas
let x: number = 5;          // Immutable
x = 10;                      // ERROR: Cannot assign to immutable variable
```

### Mutable Variables

```atlas
let mut x: number = 5;      // Mutable
x = 10;                      // OK
```

**Inspiration**: Rust's `let` vs `let mut`

## Functions

### Declaration

```atlas
fn add(a: number, b: number) -> number {
    return a + b;
}

// No return type (void):
fn print_hello() -> void {
    print("Hello");
}
```

### Export/Import

```atlas
// module.atl
export fn greet(name: string) -> string {
    return "Hello, " + name;
}

// main.atl
import { greet } from "./module";

fn main() -> void {
    let msg: string = greet("Atlas");
    print(msg);
}
```

**Important**:
- Imports are relative: `"./module"` (no `.atl` extension)
- Must be in same directory or use relative path
- Named imports: `import { fn1, fn2 } from "./module";`

## Type Annotations

Always required for variables and function parameters:

```atlas
let name: string = "Atlas";
let count: number = 42;
let flag: bool = true;
let items: array = [1, 2, 3];
let data: json = parseJSON("{...}");
let result: Result<string, string> = Ok("success");
```

## Comments

```atlas
// Single line comment

/* Multi-line
   comment */
```

## Semicolons

**Required** at end of statements:

```atlas
let x: number = 5;          // Required
print("Hello");             // Required
return x;                   // Required
```

## File Structure

```atlas
// Imports at top
import { helper } from "./utils";

// Function definitions
fn process() -> void {
    // ...
}

// Exports
export fn main() -> void {
    // ...
}

// Execute
main();
```

## Entry Point

Call your main function at the end:

```atlas
fn main() -> void {
    print("Program running");
}

main();  // Execute
```
