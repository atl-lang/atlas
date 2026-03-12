# Visibility

Atlas supports three visibility levels for controlling access to declarations.

## Keywords

| Keyword    | Scope                                    |
|------------|------------------------------------------|
| `pub`      | Accessible from any module               |
| `private`  | Accessible only within the same file     |
| `internal` | Accessible within the same module        |

Default visibility is `private`.

## Functions

```atlas
// Public - exported and accessible everywhere
pub fn public_fn(): void {
    console.log("public");
}

// Private - only accessible in this file (default)
fn private_fn(): void {
    console.log("private");
}

// Internal - accessible in all files of this module
internal fn internal_fn(): void {
    console.log("internal");
}
```

## Structs

```atlas
pub struct PublicPoint {
    pub x: number,      // public field
    y: number,          // private field (default)
}

struct PrivatePoint {   // private struct
    x: number,
    y: number,
}
```

## Enums

```atlas
pub enum Status {
    Active,
    Inactive,
}
```

## Traits

```atlas
pub trait Printable {
    fn print(borrow self): void;
}
```

## Modules

With module structure:
```
my_module/
  mod.atl      // module root
  helper.atl   // internal module file
```

```atlas
// mod.atl
pub fn public_api(): void {
    internal_helper();  // can call internal fn
}

internal fn internal_helper(): void {
    // accessible within my_module/
}

// helper.atl
internal fn another_helper(): void {
    internal_helper();  // can call internal fn from mod.atl
}
```

## Error Codes

| Code   | Description                              |
|--------|------------------------------------------|
| AT3059 | Access to private member                 |
| AT3060 | Access to internal member from outside   |

## Cross-File Imports

To import a symbol from another file, it must be exported. Use `export` to make
functions, structs, and other declarations importable:

```atlas
// math.atl
export fn add(x: number, y: number): number {
    return helper(x, y);
}

fn helper(x: number, y: number): number {
    return x + y;
}

// main.atl
import { add } from "./math";
console.log(add(1, 2).toString());  // 3
// import { helper } from "./math";  // Error AT3059: private function
```

### Import Errors

| Code   | When                                          |
|--------|-----------------------------------------------|
| AT3059 | Importing a private (non-exported) symbol     |
| AT5006 | Importing a symbol that doesn't exist         |

### Example: Private Access Violation

```atlas
// lib.atl
fn private_fn(): number { return 42; }

// main.atl
import { private_fn } from "./lib";  // Error AT3059
```

Error message: `cannot access private function 'private_fn' from outside its defining module`

## Examples

### Module Export Pattern

```atlas
// math.atl
export fn add(x: number, y: number): number {
    return helper(x, y);
}

fn helper(x: number, y: number): number {
    return x + y;
}

// main.atl
import { add } from "./math";
console.log(add(1, 2).toString());  // 3
// helper is not visible here
```

### Internal Module Helpers

```atlas
// In module with multiple files:
// mod.atl - module root
internal fn shared_helper(): number {
    return 42;
}

// other.atl - can call internal functions
fn use_helper(): number {
    return shared_helper();
}
```
