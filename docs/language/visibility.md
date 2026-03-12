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

## Examples

### Module Export Pattern

```atlas
// math.atl
pub fn add(borrow x: number, borrow y: number): number {
    return helper(x, y);
}

fn helper(borrow x: number, borrow y: number): number {
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
